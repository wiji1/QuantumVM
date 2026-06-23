pub mod lexer;
pub mod parser;
mod interpreter;
mod type_checker;
mod coercion;
mod input_resolver;
pub mod source_cache;
pub mod error_display;

pub mod lsp_support;

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::path::PathBuf;
use crate::input_resolver::InputResolver;
use crate::parser::supporting_types::{IoDirection, ClassicalType};
use crate::parser::statement::StmtKind;
use crate::parser::Program;
use std::collections::HashMap;

pub use crate::interpreter::value::Value;
pub use crate::interpreter::runtime_error::RuntimeError;
pub use crate::parser::parse_error::ParseError;
pub use crate::parser::ParseResult;
pub use crate::lsp_support::LspQuery;
pub use crate::type_checker::{TypeChecker, TypeCheckConfig, TypeCheckResult};
pub use crate::type_checker::diagnostics::{DiagnosticSeverity, Diagnostic};
pub use crate::lexer::Span;
pub use crate::source_cache::SourceCache;
pub use crate::error_display::ErrorReporter;
use crate::interpreter::runtime_error::RuntimeErrorKind;

#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Success {
        outputs: HashMap<String, Value>,
    },
    ParseError(Vec<ParseError>),
    TypeCheckError(Vec<Diagnostic>),
    RuntimeError(RuntimeError),
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionResult::Success { .. })
    }

    pub fn is_parse_error(&self) -> bool {
        matches!(self, ExecutionResult::ParseError(_))
    }

    pub fn is_type_error(&self) -> bool {
        matches!(self, ExecutionResult::TypeCheckError(_))
    }

    pub fn is_runtime_error(&self) -> bool {
        matches!(self, ExecutionResult::RuntimeError(_))
    }

    pub fn get_outputs(&self) -> Option<&HashMap<String, Value>> {
        match self {
            ExecutionResult::Success { outputs } => Some(outputs),
            _ => None,
        }
    }

    pub fn get_output(&self, name: &str) -> Option<&Value> {
        self.get_outputs()?.get(name)
    }
}

#[derive(Debug, Clone, Default)]
pub struct RunConfig {
    pub inputs: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
}

pub fn run_program(source: &str, config: RunConfig) -> ExecutionResult {
    let mut lexer = Lexer::new(source.to_string());
    lexer.start();

    let lexer_output = lexer.tokens;
    let mut parser = Parser::new(lexer_output);
    let parse_result = parser.start(true);

    if !parse_result.errors.is_empty() {
        return ExecutionResult::ParseError(parse_result.errors);
    }

    let program = parse_result.program;

    let mut type_checker = TypeChecker::new(TypeCheckConfig::default());
    let result = type_checker.check_program(&program);

    if !result.success {
        let errors: Vec<Diagnostic> = result.diagnostics
            .iter()
            .filter(|d| matches!(d.severity, DiagnosticSeverity::Error))
            .cloned()
            .collect();
        return ExecutionResult::TypeCheckError(errors);
    }

    let input_declarations = extract_input_declarations(&program);
    let mut resolver = InputResolver::new();

    for (key, value) in config.inputs {
        resolver.add_cli_input(key, value);
    }

    let (resolved_inputs, _warnings) = match resolver.resolve(&input_declarations) {
        Ok(result) => result,
        Err(e) => {
            return ExecutionResult::RuntimeError(
                RuntimeError::new(RuntimeErrorKind::InvalidCall(format!("Input resolution error: {}", e)))
            );
        }
    };

    let working_dir = config.working_dir.unwrap_or_else(|| PathBuf::from("."));
    let mut interpreter = Interpreter::new(program, working_dir);

    for (name, value) in resolved_inputs {
        interpreter.set_input(&name, value);
    }

    match interpreter.start() {
        Ok(()) => ExecutionResult::Success {
            outputs: interpreter.get_outputs().clone()
        },
        Err(e) => ExecutionResult::RuntimeError(e)
    }
}

pub fn run_file(path: &str) -> std::io::Result<ExecutionResult> {
    let source = std::fs::read_to_string(path)?;
    let script_path = PathBuf::from(path);
    let working_dir = script_path.parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    Ok(run_program(&source, RunConfig {
        working_dir: Some(working_dir),
        ..Default::default()
    }))
}

fn extract_input_declarations(program: &Program) -> HashMap<String, ClassicalType> {
    let mut declarations = HashMap::new();

    for stmt in &program.statements {
        if let StmtKind::IoDecl { direction, ty, name } = &stmt.kind {
            if matches!(direction, IoDirection::Input) {
                declarations.insert(name.clone(), ty.clone());
            }
        }
    }

    declarations
}
