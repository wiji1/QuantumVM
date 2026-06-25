use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use quantum_vm::{lexer::Lexer, parser::Parser, LspQuery, Span, TypeCheckConfig, TypeChecker};

struct DocumentState {
    text: String,
    version: i32,
    type_checker: Option<TypeChecker>,
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

    async fn parse_document(&self, text: &str, uri: &Url, version: i32) -> Option<TypeChecker> {
        let mut lexer = Lexer::new(text.to_string());
        let file_path = uri.to_file_path().ok().and_then(|p| p.to_str().map(|s| s.to_string()));
        lexer.set_file_path(file_path);
        lexer.start();

        let mut parser = Parser::new(lexer.tokens);
        let parse_result = parser.start(true);

        if !parse_result.errors.is_empty() {
            self.client
                .log_message(MessageType::ERROR, format!("Parse errors: {} error(s)", parse_result.errors.len()))
                .await;

            self.publish_parse_errors(uri.clone(), &parse_result.errors, version).await;
            return None;
        }

        let program = parse_result.program;

        let working_dir = uri.to_file_path().ok()
            .and_then(|p| p.parent().map(|parent| parent.to_path_buf()));
        let mut type_checker = TypeChecker::new(TypeCheckConfig {
            working_dir,
            ..Default::default()
        });
        let _result = type_checker.check_program(&program);

        Some(type_checker)
    }

    async fn publish_parse_errors(&self, uri: Url, errors: &[quantum_vm::ParseError], version: i32) {
        let diagnostics: Vec<Diagnostic> = errors
            .iter()
            .map(|error| {
                let range = if let Some(span) = error.span() {
                    self.span_to_range(span)
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

    fn span_to_range(&self, span: &Span) -> Range {
        Range {
            start: Position {
                line: span.line as u32,
                character: span.col as u32,
            },
            end: Position {
                line: span.line as u32,
                character: (span.col + span.len) as u32,
            },
        }
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
                    self.span_to_range(span)
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

        self.client
            .log_message(MessageType::INFO, format!("Document opened: {}", uri))
            .await;

        let type_checker = self.parse_document(&text, &params.text_document.uri, version).await;

        if let Some(ref checker) = type_checker {
            self.publish_diagnostics(params.text_document.uri.clone(), checker, version)
                .await;
        }

        self.documents.insert(
            uri,
            DocumentState {
                text,
                version,
                type_checker,
            },
        );
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let version = params.text_document.version;
        let text = params.content_changes[0].text.clone();

        let type_checker = self.parse_document(&text, &params.text_document.uri, version).await;

        if let Some(ref checker) = type_checker {
            self.publish_diagnostics(params.text_document.uri.clone(), checker, version)
                .await;
        }

        self.documents.insert(
            uri,
            DocumentState {
                text,
                version,
                type_checker,
            },
        );
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.documents.remove(&uri);

        self.client
            .log_message(MessageType::INFO, format!("Document closed: {}", uri))
            .await;
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
            let range = self.span_to_range(&def_span);
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
        let ref_spans = lsp_query.find_references(line, col);

        let current_uri = &params.text_document_position.text_document.uri;
        let mut locations: Vec<Location> = ref_spans
            .iter()
            .map(|span| Location {
                uri: self.span_to_uri(span, current_uri),
                range: self.span_to_range(span),
            })
            .collect();

        if !params.context.include_declaration {
            if let Some(def_span) = lsp_query.find_definition(line, col) {
                let def_range = self.span_to_range(&def_span);
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
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));

    Server::new(stdin, stdout, socket).serve(service).await;
}
