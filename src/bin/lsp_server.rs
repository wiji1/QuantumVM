use std::path::PathBuf;
use std::sync::OnceLock;

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use quantum_vm::parser::{self, Parser};
use quantum_vm::{lexer::Lexer, span_to_range, type_checker::reference_registry::Reference, LspQuery, Span, TypeCheckConfig, TypeChecker};

static STDLIB_PATH: OnceLock<PathBuf> = OnceLock::new();

fn ensure_stdlib_extracted() -> PathBuf {
    let path = STDLIB_PATH.get_or_init(|| {
        let cache_dir = dirs::cache_dir().expect("valid OS cache directory");
        let target = cache_dir.join("qasm-lsp").join(parser::STDGATES_FILENAME);
        if !target.exists() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).expect("failed to create stdlib cache directory");
            }
            std::fs::write(&target, parser::STDGATES_CONTENT).expect("failed to write stdlib to cache");
        }
        target
    });
    path.clone()
}

struct DocumentState {
    text: String,
    version: i32,
    type_checker: Option<TypeChecker>,
    includes: Vec<String>,
}

struct Backend {
    client: Client,
    documents: DashMap<String, DocumentState>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
        }
    }

    async fn parse_document(&self, text: &str, uri: &Url, version: i32) -> Option<(TypeChecker, Vec<String>)> {
        let mut lexer = Lexer::new(text.to_string());
        let file_path = uri.to_file_path().ok().and_then(|p| p.to_str().map(|s| s.to_string()));
        lexer.set_file_path(file_path);
        lexer.start();

        let mut parser = Parser::new(lexer.tokens);
        let parse_result = parser.start(true);

        if !parse_result.errors.is_empty() {
            self.publish_parse_errors(uri.clone(), &parse_result.errors, version).await;
            return None;
        }

        let mut program = parse_result.program;

        let includes = self.extract_includes(&program, uri);

        self.inject_include_contents(&mut program, uri);

        let working_dir = uri.to_file_path().ok()
            .and_then(|p| p.parent().map(|parent| parent.to_path_buf()));
        let mut type_checker = TypeChecker::new(TypeCheckConfig {
            working_dir,
            ..Default::default()
        });
        let _result = type_checker.check_program(&program);

        Some((type_checker, includes))
    }

    fn inject_include_contents(&self, program: &mut parser::Program, current_uri: &Url) {
        use quantum_vm::parser::statement::{Stmt, StmtKind};

        let working_dir = current_uri.to_file_path().ok()
            .and_then(|p| p.parent().map(|parent| parent.to_path_buf()));

        for stmt in &mut program.statements {
            if let StmtKind::Include(path) = &stmt.kind {
                let (full_path, content) = if path == parser::STDGATES_FILENAME {
                    (ensure_stdlib_extracted(), Some(parser::STDGATES_CONTENT.to_string()))
                } else if let Some(ref dir) = working_dir {
                    let full_path = dir.join(path);
                    if let Ok(file_url) = Url::from_file_path(&full_path) {
                        let uri_string = file_url.to_string();
                        if let Some(doc) = self.documents.get(&uri_string) {
                            (full_path, Some(doc.text.clone()))
                        } else { continue; }
                    } else { continue; }
                } else { continue; };

                if let Some(content) = content {
                    let full_path_str = full_path.to_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| path.clone());
                    *stmt = Stmt::new(
                        StmtKind::IncludeFromSrc(full_path_str, content),
                        stmt.span.clone()
                    );
                }
            }
        }
    }

    fn extract_includes(&self, program: &quantum_vm::parser::Program, uri: &Url) -> Vec<String> {
        use quantum_vm::parser::statement::StmtKind;
        let working_dir = uri.to_file_path().ok()
            .and_then(|p| p.parent().map(|parent| parent.to_path_buf()));

        program.statements.iter()
            .filter_map(|stmt| {
                if let StmtKind::Include(path) = &stmt.kind {
                    if let Some(ref dir) = working_dir {
                        let full_path = dir.join(path);
                        Url::from_file_path(&full_path).ok().map(|u| u.to_string())
                    } else { None }
                } else { None }
            })
            .collect()
    }

    fn find_including_documents(&self, included_uri: &str) -> Vec<String> {
        self.documents.iter()
            .filter_map(|entry| {
                let (doc_uri, doc_state) = entry.pair();
                if doc_state.includes.contains(&included_uri.to_string()) { Some(doc_uri.clone()) }
                else { None }
            })
            .collect()
    }

    async fn invalidate_dependent_files(&self, changed_uri: &str) {
        let dependent_uris = self.find_including_documents(changed_uri);

        for dependent_uri in dependent_uris {
            let Some(doc_state) = self.documents.get(&dependent_uri) else { continue };
            let text = doc_state.text.clone();
            let version = doc_state.version;
            drop(doc_state);

            let Ok(uri) = Url::parse(&dependent_uri) else { continue };
            let parse_result = self.parse_document(&text, &uri, version).await;

            let (type_checker, includes) = match parse_result {
                Some((tc, inc)) => {
                    self.publish_diagnostics(uri.clone(), &tc, version).await;
                    (Some(tc), inc)
                }
                None => (None, vec![]),
            };

            self.documents.insert(
                dependent_uri,
                DocumentState {
                    text,
                    version,
                    type_checker,
                    includes,
                },
            );
        }
    }

    fn find_cross_file_references(&self, current_uri: &str, symbol_name: &str) -> Vec<Location> {
        let mut related_uris = Vec::new();

        related_uris.extend(self.find_including_documents(current_uri));

        if let Some(current_doc) = self.documents.get(current_uri) {
            related_uris.extend(current_doc.includes.iter().cloned());
        }

        related_uris
            .into_iter()
            .flat_map(|related_uri| {
                let Some(doc) = self.documents.get(&related_uri) else { return vec![] };
                let Some(ref tc) = doc.type_checker else { return vec![] };
                let Some(refs) = tc.reference_registry().get_references(symbol_name) else { return vec![] };

                refs.iter()
                    .filter_map(|ref_item| {
                        let file = ref_item.span.file.as_ref()?;
                        let file_url = Url::from_file_path(file).ok()?;
                        (file_url.to_string() == related_uri).then(|| Location {
                            uri: file_url,
                            range: span_to_range(&ref_item.span),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    async fn publish_parse_errors(&self, uri: Url, errors: &[quantum_vm::ParseError], version: i32) {
        let diagnostics: Vec<Diagnostic> = errors
            .iter()
            .map(|error| {
                let range = if let Some(span) = error.span() {
                    span_to_range(span)
                } else {
                    Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 1,
                        },
                    }
                };

                Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: error.to_string(),
                    source: Some("qasm-lsp".to_string()),
                    ..Default::default()
                }
            })
            .collect();

        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }
    
    fn position_to_coords(&self, position: Position) -> (usize, usize) {
        (position.line as usize, position.character as usize)
    }

    fn span_to_uri(&self, span: &Span, current_uri: &Url) -> Url {
        if let Some(ref file_path) = span.file {
            Url::from_file_path(file_path).unwrap_or_else(|_| current_uri.clone())
        } else { current_uri.clone() }
    }

    async fn publish_diagnostics(&self, uri: Url, type_checker: &TypeChecker, version: i32) {
        let diagnostics: Vec<Diagnostic> = type_checker
            .get_diagnostics()
            .iter()
            .filter_map(|diag| {
                let range = if let Some(ref span) = diag.span {
                    span_to_range(span)
                } else {
                    Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 1,
                        },
                    }
                };

                let severity = match diag.severity {
                    quantum_vm::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
                    quantum_vm::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
                    quantum_vm::DiagnosticSeverity::Info => DiagnosticSeverity::INFORMATION,
                    quantum_vm::DiagnosticSeverity::Hint => DiagnosticSeverity::HINT,
                };

                Some(Diagnostic {
                    range,
                    severity: Some(severity),
                    message: diag.message.clone(),
                    source: Some("qasm-lsp".to_string()),
                    ..Default::default()
                })
            })
            .collect();

        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "QASM Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    ..Default::default()
                }),
                rename_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::PARAMETER,
                                    SemanticTokenType::CLASS,
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::READONLY,
                                ],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            ..Default::default()
                        }
                    )
                ),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "QASM Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;
        let version = params.text_document.version;

        let parse_result = self.parse_document(&text, &params.text_document.uri, version).await;

        let (type_checker, includes) = match parse_result {
            Some((tc, inc)) => {
                self.publish_diagnostics(params.text_document.uri.clone(), &tc, version).await;
                (Some(tc), inc)
            }
            None => (None, vec![]),
        };

        self.documents.insert(
            uri,
            DocumentState {
                text,
                version,
                type_checker,
                includes,
            },
        );
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let version = params.text_document.version;
        let text = params.content_changes[0].text.clone();

        let parse_result = self.parse_document(&text, &params.text_document.uri, version).await;

        let (type_checker, includes) = match parse_result {
            Some((tc, inc)) => {
                self.publish_diagnostics(params.text_document.uri.clone(), &tc, version).await;
                (Some(tc), inc)
            }
            None => (None, vec![]),
        };

        self.documents.insert(
            uri.clone(),
            DocumentState {
                text,
                version,
                type_checker,
                includes,
            },
        );

        self.invalidate_dependent_files(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.documents.remove(&uri);
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;
        let (line, col) = self.position_to_coords(position);

        let doc = match self.documents.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let type_checker = match &doc.type_checker {
            Some(tc) => tc,
            None => return Ok(None),
        };

        let lsp_query = LspQuery::new(type_checker);

        if let Some(def_span) = lsp_query.find_definition(line, col) {
            let range = span_to_range(&def_span);
            let target_uri = self.span_to_uri(&def_span, &params.text_document_position_params.text_document.uri);

            Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: target_uri,
                range,
            })))
        } else {
            Ok(None)
        }
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;
        let (line, col) = self.position_to_coords(position);

        let doc = match self.documents.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let type_checker = match &doc.type_checker {
            Some(tc) => tc,
            None => return Ok(None),
        };

        let lsp_query = LspQuery::new(type_checker);

        let symbol_name = match lsp_query.get_symbol_at_position(line, col) {
            Some(name) => name,
            None => return Ok(None),
        };

        let ref_spans = lsp_query.find_references(line, col);
        let current_uri = &params.text_document_position.text_document.uri;
        let mut locations: Vec<Location> = ref_spans
            .iter()
            .map(|span| {
                let uri = self.span_to_uri(span, current_uri);
                let range = span_to_range(span);
                Location { uri, range }
            })
            .collect();

        let cross_file_refs = self.find_cross_file_references(&uri, &symbol_name);
        locations.extend(cross_file_refs);

        locations.sort_by(|a, b| {
            a.uri.to_string().cmp(&b.uri.to_string())
                .then_with(|| a.range.start.line.cmp(&b.range.start.line))
                .then_with(|| a.range.start.character.cmp(&b.range.start.character))
        });
        locations.dedup_by(|a, b| {
            a.uri == b.uri && a.range == b.range
        });

        if !params.context.include_declaration {
            if let Some(def_span) = lsp_query.find_definition(line, col) {
                let def_range = span_to_range(&def_span);
                let def_uri = self.span_to_uri(&def_span, current_uri);
                locations.retain(|loc| {
                    loc.uri != def_uri || loc.range != def_range
                });
            }
        }

        Ok(Some(locations))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;
        let (line, col) = self.position_to_coords(position);

        let doc = match self.documents.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let type_checker = match &doc.type_checker {
            Some(tc) => tc,
            None => return Ok(None),
        };

        let lsp_query = LspQuery::new(type_checker);

        if let Some(hover_info) = lsp_query.get_hover_info(line, col) {
            Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```qasm\n{}\n```", hover_info),
                }),
                range: None,
            }))
        } else { Ok(None) }
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;
        let (line, col) = self.position_to_coords(position);

        let doc = match self.documents.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let type_checker = match &doc.type_checker {
            Some(tc) => tc,
            None => return Ok(None),
        };

        let lsp_query = LspQuery::new(type_checker);
        let ref_spans = lsp_query.find_references(line, col);

        let highlights: Vec<DocumentHighlight> = ref_spans
            .iter()
            .filter(|span| {
                span.file.as_ref().map_or(true, |f| {
                    params.text_document_position_params.text_document.uri
                        .to_file_path()
                        .ok()
                        .and_then(|p| p.to_str().map(|s| s == f))
                        .unwrap_or(false)
                }) || span.file.is_none()
            })
            .map(|span| {
                let kind = if let Some(def_span) = lsp_query.find_definition(line, col) {
                    if span.line == def_span.line
                        && span.col == def_span.col
                        && span.len == def_span.len {
                        Some(DocumentHighlightKind::WRITE)
                    } else { Some(DocumentHighlightKind::READ) }
                } else { Some(DocumentHighlightKind::TEXT) };

                DocumentHighlight {
                    range: span_to_range(span),
                    kind,
                }
            })
            .collect();

        if highlights.is_empty() { Ok(None) }
        else { Ok(Some(highlights)) }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.to_string();

        let doc = match self.documents.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let type_checker = match &doc.type_checker {
            Some(tc) => tc,
            None => return Ok(None),
        };

        let lsp_query = LspQuery::new(type_checker);
        let symbols = lsp_query.get_document_symbols();

        if symbols.is_empty() { Ok(None) }
        else { Ok(Some(DocumentSymbolResponse::Flat(symbols))) }
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;
        let (line, col) = self.position_to_coords(position);

        let doc = match self.documents.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let type_checker = match &doc.type_checker {
            Some(tc) => tc,
            None => return Ok(None),
        };

        let lsp_query = LspQuery::new(type_checker);
        let completions = lsp_query.get_completions(line, col);

        if completions.is_empty() { Ok(None) }
        else { Ok(Some(CompletionResponse::Array(completions))) }
    }

    async fn signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;
        let (line, col) = self.position_to_coords(position);

        let doc = match self.documents.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let text = &doc.text;
        let type_checker = match &doc.type_checker {
            Some(tc) => tc,
            None => return Ok(None),
        };

        let lsp_query = LspQuery::new(type_checker);
        lsp_query.get_signature_help(text, line, col)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;
        let (line, col) = self.position_to_coords(position);
        let new_name = params.new_name;

        let doc = match self.documents.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let type_checker = match &doc.type_checker {
            Some(tc) => tc,
            None => return Ok(None),
        };

        let lsp_query = LspQuery::new(type_checker);

        let symbol_name = match lsp_query.get_symbol_at_position(line, col) {
            Some(name) => name,
            None => return Ok(None),
        };

        let ref_spans = lsp_query.find_references(line, col);
        let current_uri = &params.text_document_position.text_document.uri;

        let mut locations: Vec<Location> = ref_spans
            .iter()
            .map(|span| Location {
                uri: self.span_to_uri(span, current_uri),
                range: span_to_range(span),
            })
            .collect();

        let cross_file_refs = self.find_cross_file_references(&uri, &symbol_name);
        locations.extend(cross_file_refs);

        if locations.is_empty() { return Ok(None); }

            let mut changes = std::collections::HashMap::new();
        for location in locations {
            let text_edit = TextEdit {
                range: location.range,
                new_text: new_name.clone(),
            };
            changes.entry(location.uri).or_insert_with(Vec::new).push(text_edit);
        }

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.to_string();

        let doc = match self.documents.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let type_checker = match &doc.type_checker {
            Some(tc) => tc,
            None => return Ok(None),
        };

        let mut tokens: Vec<u32> = Vec::new();
        let mut symbol_tokens: Vec<(usize, usize, usize, u32, u32)> = Vec::new();

        const TOKEN_VARIABLE: u32 = 0;
        const TOKEN_FUNCTION: u32 = 1;
        const TOKEN_PARAMETER: u32 = 2;
        const TOKEN_GATE: u32 = 3;

        const MODIFIER_READONLY: u32 = 1 << 0;

        let env = type_checker.env();

        for (name, binding) in env.get_all_bindings() {
            if let Some(refs) = type_checker.reference_registry().get_references(&name) {
                let modifiers = if binding.is_const { MODIFIER_READONLY } else { 0 };
                for reference in refs {
                    if !in_current_file(reference, &uri) { continue; }

                    symbol_tokens.push((
                        reference.span.line,
                        reference.span.col,
                        reference.span.len,
                        TOKEN_VARIABLE,
                        modifiers,
                    ));
                }
            }
        }

        for (name, _) in env.get_all_functions() {
            if let Some(refs) = type_checker.reference_registry().get_references(&name) {
                for reference in refs {
                    if !in_current_file(reference, &uri) { continue; }

                    symbol_tokens.push((
                        reference.span.line,
                        reference.span.col,
                        reference.span.len,
                        TOKEN_FUNCTION,
                        0,
                    ));
                }
            }
        }

        for (name, _) in env.get_all_gates() {
            if let Some(refs) = type_checker.reference_registry().get_references(&name) {
                let token_type = TOKEN_GATE;

                for reference in refs {
                    if !in_current_file(reference, &uri) { continue; }

                    symbol_tokens.push((
                        reference.span.line,
                        reference.span.col,
                        reference.span.len,
                        token_type,
                        0,
                    ));
                }
            }
        }

        symbol_tokens.sort_by_key(|t| (t.0, t.1));

        let mut prev_line = 0;
        let mut prev_col = 0;

        for (line, col, len, token_type, modifiers) in symbol_tokens {
            let delta_line = line - prev_line;
            let delta_col = if delta_line == 0 { col - prev_col } else { col };

            tokens.push(delta_line as u32);
            tokens.push(delta_col as u32);
            tokens.push(len as u32);
            tokens.push(token_type);
            tokens.push(modifiers);

            prev_line = line;
            prev_col = col;
        }

        let semantic_tokens: Vec<SemanticToken> = tokens
            .chunks_exact(5)
            .map(|chunk| SemanticToken {
                delta_line: chunk[0],
                delta_start: chunk[1],
                length: chunk[2],
                token_type: chunk[3],
                token_modifiers_bitset: chunk[4],
            })
            .collect();

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: semantic_tokens,
        })))
    }
}

fn in_current_file(reference: &Reference, uri: &String) -> bool {
    match &reference.span.file {
        None => true,
        Some(file) => {
            Url::from_file_path(file)
                .ok()
                .map(|u| u.to_string() == *uri)
                .unwrap_or(false)
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));

    Server::new(stdin, stdout, socket).serve(service).await;
}
