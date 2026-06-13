pub mod type_repr;
pub mod type_env;
pub mod static_error;
pub mod diagnostics;
pub mod coercion;
pub mod expr_checker;
pub mod stmt_checker;

use crate::parser::Program;
use crate::type_checker::diagnostics::{Diagnostic, DiagnosticCollector};
use crate::type_checker::type_env::TypeEnv;
use crate::type_checker::static_error::StaticError;

#[derive(Debug, Clone)]
pub struct TypeCheckConfig {
    pub strict_mode: bool,
    pub allow_implicit_casts: bool,
    pub collect_all_errors: bool,
}

impl Default for TypeCheckConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            allow_implicit_casts: true,
            collect_all_errors: true,
        }
    }
}

#[derive(Debug)]
pub struct TypeCheckResult {
    pub success: bool,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct TypeChecker {
    env: TypeEnv,
    diagnostics: DiagnosticCollector,
    config: TypeCheckConfig,
    current_function: Option<FunctionContext>,
}

#[derive(Debug, Clone)]
struct FunctionContext {
    name: String,
    return_type: type_repr::Type,
}

impl TypeChecker {
    pub fn new(config: TypeCheckConfig) -> Self {
        let mut checker = Self {
            env: TypeEnv::new(),
            diagnostics: DiagnosticCollector::new(),
            config,
            current_function: None,
        };

        checker
    }

    pub fn check_program(&mut self, program: &Program) -> TypeCheckResult {
        for stmt in &program.statements {
            if let Err(e) = self.check_statement(stmt) {
                self.diagnostics.add_error(e);

                if self.config.strict_mode && !self.config.collect_all_errors { break; }
            }
        }

        TypeCheckResult {
            success: !self.diagnostics.has_errors(),
            diagnostics: self.diagnostics.get_diagnostics().to_vec(),
        }
    }

    fn check_statement(&mut self, stmt: &crate::parser::statement::Stmt) -> Result<(), StaticError> {
        stmt_checker::check_statement(self, stmt)
    }

    pub fn env(&self) -> &TypeEnv {
        &self.env
    }
    pub fn env_mut(&mut self) -> &mut TypeEnv {
        &mut self.env
    }
    pub fn config(&self) -> &TypeCheckConfig {
        &self.config
    }
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.add(diagnostic);
    }
    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        self.diagnostics.get_diagnostics()
    }
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    pub(crate) fn current_function(&self) -> Option<&FunctionContext> {
        self.current_function.as_ref()
    }

    pub(crate) fn set_function_context(&mut self, name: String, return_type: type_repr::Type) {
        self.current_function = Some(FunctionContext {
            name,
            return_type,
        });
    }

    pub(crate) fn clear_function_context(&mut self) {
        self.current_function = None;
    }
}
