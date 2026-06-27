use std::collections::HashMap;
use crate::lexer::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceType {
    VariableRead,
    VariableWrite,
    FunctionCall,
    FunctionDefinition,
    GateCall,
    GateDefinition,
    ParameterDefinition,
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub name: String,
    pub span: Span,
    pub reference_type: ReferenceType,
}

pub struct ReferenceRegistry {
    references: HashMap<String, Vec<Reference>>,
}

impl ReferenceRegistry {
    pub fn new() -> Self {
        Self {
            references: HashMap::new(),
        }
    }

    pub fn add_reference(&mut self, name: String, span: Span, ref_type: ReferenceType) {
        self.references
            .entry(name.clone())
            .or_insert_with(Vec::new)
            .push(Reference {
                name,
                span,
                reference_type: ref_type,
            });
    }

    pub fn get_references(&self, name: &str) -> Option<&Vec<Reference>> {
        self.references.get(name)
    }

    pub fn find_references_at_position(&self, line: usize, col: usize) -> Vec<Reference> {
        let mut results = Vec::new();

        for (_, refs) in &self.references {
            for reference in refs {
                if reference.span.line == line
                    && col >= reference.span.col
                    && col <= reference.span.col + reference.span.len {
                    results.push(reference.clone());
                }
            }
        }

        results
    }

    pub fn find_symbol_at_position(&self, line: usize, col: usize) -> Option<String> {
        for (symbol_name, refs) in &self.references {
            for reference in refs {
                if reference.span.line == line
                    && col >= reference.span.col
                    && col <= reference.span.col + reference.span.len {
                    return Some(symbol_name.clone());
                }
            }
        }
        None
    }

    pub fn get_all_references_at_position(&self, line: usize, col: usize) -> Vec<Reference> {
        if let Some(symbol_name) = self.find_symbol_at_position(line, col) {
            self.get_references(&symbol_name)
                .map(|refs| refs.clone())
                .unwrap_or_default()
        } else { Vec::new() }
    }
}

impl Default for ReferenceRegistry {
    fn default() -> Self {
        Self::new()
    }
}