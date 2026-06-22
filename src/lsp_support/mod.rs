use crate::lexer::Span;
use crate::type_checker::reference_registry::Reference;
use crate::type_checker::TypeChecker;

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

    pub fn get_symbol_references(&self, symbol_name: &str) -> Vec<Reference> {
        self.checker
            .reference_registry()
            .get_references(symbol_name)
            .map(|refs| refs.clone())
            .unwrap_or_default()
    }

    pub fn get_symbol_at_position(&self, line: usize, col: usize) -> Option<String> {
        self.checker
            .reference_registry()
            .find_symbol_at_position(line, col)
    }
}

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
