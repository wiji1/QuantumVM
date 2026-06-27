use crate::lexer::Span;
use crate::type_checker::TypeChecker;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use crate::span_to_range;
use crate::type_checker::reference_registry::ReferenceType;

pub struct LspQuery<'a> {
    checker: &'a TypeChecker,
}

impl<'a> LspQuery<'a> {
    pub fn new(checker: &'a TypeChecker) -> Self {
        Self { checker }
    }

    pub fn find_definition(&self, line: usize, col: usize) -> Option<Span> {
        let symbol_name = self.checker
            .reference_registry()
            .find_symbol_at_position(line, col)?;

        if let Some(binding) = self.checker.env().lookup(&symbol_name) {
            return binding.definition_span.clone();
        }
        if let Some(func_sig) = self.checker.env().get_function(&symbol_name) {
            return func_sig.definition_span.clone();
        }

        if let Some(gate_sig) = self.checker.env().get_gate(&symbol_name) {
            return gate_sig.definition_span.clone();
        }

        if let Some(references) = self.checker.reference_registry().get_references(&symbol_name) {
            for reference in references {
                if reference.reference_type == ReferenceType::ParameterDefinition {
                    return Some(reference.span.clone());
                }
            }
        }

        None
    }

    pub fn find_references(&self, line: usize, col: usize) -> Vec<Span> {
        self.checker
            .reference_registry()
            .get_all_references_at_position(line, col)
            .iter()
            .map(|r| r.span.clone())
            .collect()
    }


    pub fn get_hover_info(&self, line: usize, col: usize) -> Option<String> {
        let symbol_name = self.checker
            .reference_registry()
            .find_symbol_at_position(line, col)?;

        if let Some(binding) = self.checker.env().lookup(&symbol_name) {
            let const_str = if binding.is_const { "const " } else { "" };
            return Some(format!(
                "{const_str}{}: {}",
                symbol_name,
                format_type(&binding.ty)
            ));
        }

        if let Some(func_sig) = self.checker.env().get_function(&symbol_name) {
            let params_str = func_sig
                .params
                .iter()
                .map(format_type)
                .collect::<Vec<_>>()
                .join(", ");
            let return_str = format_type(&func_sig.return_type);
            return Some(format!(
                "function {}({}) -> {}",
                symbol_name, params_str, return_str
            ));
        }

        if let Some(gate_sig) = self.checker.env().get_gate(&symbol_name) {
            let params_str = gate_sig.params.join(", ");
            let qubits_str = gate_sig.qubits.join(", ");
            return Some(format!(
                "gate {}({}) {}",
                symbol_name, params_str, qubits_str
            ));
        }

        None
    }

    pub fn get_symbol_at_position(&self, line: usize, col: usize) -> Option<String> {
        self.checker
            .reference_registry()
            .find_symbol_at_position(line, col)
    }

    pub fn get_document_symbols(&self) -> Vec<SymbolInformation> {
        let mut symbols = Vec::new();

        for (name, binding) in self.checker.env().get_all_bindings() {
            if let Some(ref span) = binding.definition_span {
                let file_path = match &span.file {
                    Some(path) => path,
                    None => continue,
                };

                let uri = match Url::from_file_path(file_path) {
                    Ok(uri) => uri,
                    Err(_) => continue,
                };

                let kind = if binding.is_const { SymbolKind::CONSTANT }
                else { SymbolKind::VARIABLE };

                #[allow(deprecated)]
                symbols.push(SymbolInformation {
                    name: name.clone(),
                    kind,
                    tags: None,
                    deprecated: None,
                    location: Location { uri, range: span_to_range(span), },
                    container_name: None,
                });
            }
        }

        for (name, func_sig) in self.checker.env().get_all_functions() {
            if let Some(ref span) = func_sig.definition_span {
                let file_path = match &span.file {
                    Some(path) => path,
                    None => continue,
                };

                let uri = match Url::from_file_path(file_path) {
                    Ok(uri) => uri,
                    Err(_) => continue,
                };

                #[allow(deprecated)]
                symbols.push(SymbolInformation {
                    name: name.clone(),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    deprecated: None,
                    location: Location { uri, range: span_to_range(span) },
                    container_name: None,
                });
            }
        }

        for (name, gate_sig) in self.checker.env().get_all_gates() {
            if let Some(ref span) = gate_sig.definition_span {
                let file_path = match &span.file {
                    Some(path) => path,
                    None => continue,
                };

                let uri = match Url::from_file_path(file_path) {
                    Ok(uri) => uri,
                    Err(_) => continue,
                };

                #[allow(deprecated)]
                symbols.push(SymbolInformation {
                    name: name.clone(),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    deprecated: None,
                    location: Location { uri, range: span_to_range(span) },
                    container_name: None,
                });
            }
        }

        symbols
    }

    pub fn get_completions(&self, _line: usize, _col: usize) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        for (name, binding) in self.checker.env().get_all_bindings() {
            let detail = format!("{}: {}", name, format_type(&binding.ty));
            completions.push(CompletionItem {
                label: name.clone(),
                kind: Some(if binding.is_const { CompletionItemKind::CONSTANT }
                else { CompletionItemKind::VARIABLE }),
                detail: Some(detail),
                ..Default::default()
            });
        }

        for (name, func_sig) in self.checker.env().get_all_functions() {
            let params_str = func_sig.params.iter().map(format_type).collect::<Vec<_>>().join(", ");
            let return_str = format_type(&func_sig.return_type);
            let detail = format!("({}) -> {}", params_str, return_str);

            completions.push(CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(detail.clone()),
                insert_text: Some(format!("{}(${{1}})", name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                documentation: Some(Documentation::String(detail)),
                ..Default::default()
            });
        }

        for (name, gate_sig) in self.checker.env().get_all_gates() {
            let params_str = gate_sig.params.join(", ");
            let qubits_str = gate_sig.qubits.join(", ");
            let detail = if !params_str.is_empty() {
                format!("gate {}({}) {}", name, params_str, qubits_str)
            } else { format!("gate {} {}", name, qubits_str) };

            completions.push(CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(detail.clone()),
                insert_text: Some(format!("{}(${{1}})", name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                documentation: Some(Documentation::String(detail)),
                ..Default::default()
            });
        }

        //TODO: Pull this from keyword enum
        let keywords = vec![
            "include", "def", "gate", "qubit", "bit", "int", "uint", "float",
            "angle", "bool", "const", "let", "if", "else", "for", "while",
            "break", "continue", "return", "measure", "reset", "barrier",
            "delay", "stretch", "duration", "array", "void", "complex",
        ];

        for keyword in keywords {
            completions.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            });
        }

        completions
    }

    pub fn get_signature_help(&self, text: &str, line: usize, col: usize) -> Result<Option<SignatureHelp>> {
        let (func_name, param_index) = match self.find_function_call_context(text, line, col) {
            Some(ctx) => ctx,
            None => return Ok(None),
        };

        if let Some(func_sig) = self.checker.env().get_function(&func_name) {
            let params: Vec<ParameterInformation> = func_sig
                .params.iter().enumerate().map(|(i, ty)| {
                    let label = format!("param{}: {}", i, format_type(ty));
                    ParameterInformation {
                        label: ParameterLabel::Simple(label),
                        documentation: None,
                    }
                })
                .collect();

            let params_str = func_sig.params.iter().enumerate()
                .map(|(i, ty)| format!("param{}: {}", i, format_type(ty))).collect::<Vec<_>>()
                .join(", ");
            let return_str = format_type(&func_sig.return_type);
            let label = format!("{}({}) -> {}", func_name, params_str, return_str);

            return Ok(Some(SignatureHelp {
                signatures: vec![SignatureInformation {
                    label,
                    documentation: None,
                    parameters: Some(params),
                    active_parameter: Some(param_index as u32),
                }],
                active_signature: Some(0),
                active_parameter: Some(param_index as u32),
            }));
        }

        if let Some(gate_sig) = self.checker.env().get_gate(&func_name) {
            let mut params: Vec<ParameterInformation> = gate_sig.params.iter()
                .map(|p| {
                    ParameterInformation {
                        label: ParameterLabel::Simple(p.clone()),
                        documentation: None,
                    }
                })
                .collect();

            params.extend(gate_sig.qubits.iter().map(|q| {
                ParameterInformation {
                    label: ParameterLabel::Simple(q.clone()),
                    documentation: None,
                }
            }));

            let label = if !gate_sig.params.is_empty() {
                format!("gate {}({}) {}", func_name, gate_sig.params.join(", "), gate_sig.qubits.join(", "))
            } else { format!("gate {} {}", func_name, gate_sig.qubits.join(", ")) };

            return Ok(Some(SignatureHelp {
                signatures: vec![SignatureInformation {
                    label,
                    documentation: None,
                    parameters: Some(params),
                    active_parameter: Some(param_index as u32),
                }],
                active_signature: Some(0),
                active_parameter: Some(param_index as u32),
            }));
        }

        Ok(None)
    }

    fn find_function_call_context(&self, text: &str, line: usize, col: usize) -> Option<(String, usize)> {
        let lines: Vec<&str> = text.lines().collect();
        if line >= lines.len() { return None; }

        let current_line = lines[line];
        if col > current_line.len() { return None; }

        let text_before_cursor = &current_line[..col];

        let mut paren_depth = 0;
        let mut function_start = None;

        for (i, c) in text_before_cursor.char_indices().rev() {
            match c {
                ')' => paren_depth += 1,
                '(' => {
                    if paren_depth == 0 {
                        function_start = Some(i);
                        break;
                    }
                    paren_depth -= 1;
                }
                _ => {}
            }
        }

        let func_start_idx = function_start?;

        let text_before_paren = &text_before_cursor[..func_start_idx].trim_end();
        let func_name = text_before_paren
            .split(|c: char| !c.is_alphanumeric() && c != '_').last()?.to_string();

        if func_name.is_empty() { return None; }

        let text_in_call = &text_before_cursor[func_start_idx + 1..];
        let param_index = text_in_call.matches(',').count();

        Some((func_name, param_index))
    }
}

//TODO: Incorporate this into the enum
fn format_type(ty: &crate::type_checker::type_repr::Type) -> String {
    use crate::type_checker::type_repr::Type::*;
    match ty {
        Int(Some(size)) => format!("int[{size}]"),
        Int(None) => "int".to_string(),
        UInt(Some(size)) => format!("uint[{size}]"),
        UInt(None) => "uint".to_string(),
        Float(Some(size)) => format!("float[{size}]"),
        Float(None) => "float".to_string(),
        Bool => "bool".to_string(),
        Bit(Some(size)) => format!("bit[{size}]"),
        Bit(None) => "bit".to_string(),
        Qubit(Some(size)) => format!("qubit[{size}]"),
        Qubit(None) => "qubit".to_string(),
        Angle(Some(size)) => format!("angle[{size}]"),
        Angle(None) => "angle".to_string(),
        Complex(inner) => format!("complex[{}]", format_type(inner)),
        Array { element_type, dimensions } => {
            let dims_str = dimensions
                .iter()
                .map(|d| d.map_or("?".to_string(), |n| n.to_string()))
                .collect::<Vec<_>>()
                .join("][");
            format!("array[{}][{dims_str}]", format_type(element_type))
        }
        Duration => "duration".to_string(),
        Stretch => "stretch".to_string(),
        Range => "range".to_string(),
        Void => "void".to_string(),
        Unknown => "unknown".to_string(),
        Function { .. } => "function".to_string(),
        Unspecified => "unspecified".to_string(),
    }
}
