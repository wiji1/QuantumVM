use crate::type_checker::static_error::StaticError;
use std::fmt;
use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub span: Option<Span>,
}

impl Diagnostic {
    pub fn error(message: String) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message,
            line: None,
            column: None,
            span: None,
        }
    }

    pub fn warning(message: String) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message,
            line: None,
            column: None,
            span: None,
        }
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span.clone());
        self.line = Some(span.line);
        self.column = Some(span.col);
        self
    }

    pub fn from_type_error(error: StaticError) -> Self {
        Self::error(error.to_string())
    }

    pub fn from_type_error_with_span(error: StaticError, span: Span) -> Self {
        Self::error(error.to_string()).with_span(span)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity_str = match self.severity {
            DiagnosticSeverity::Error => "Error",
            DiagnosticSeverity::Warning => "Warning",
            DiagnosticSeverity::Info => "Info",
            DiagnosticSeverity::Hint => "Hint",
        };

        if let (Some(line), Some(col)) = (self.line, self.column) {
            write!(f, "{severity_str} at {}:{}: {}", line, col, self.message)
        } else {
            write!(f, "{severity_str}: {}", self.message)
        }
    }
}

#[derive(Debug, Default)]
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn add_error(&mut self, error: StaticError) {
        self.diagnostics.push(Diagnostic::from_type_error(error));
    }

    pub fn add_warning(&mut self, message: String) {
        self.diagnostics.push(Diagnostic::warning(message));
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .count()
    }

    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

impl fmt::Display for DiagnosticCollector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in &self.diagnostics {
            writeln!(f, "{diagnostic}")?;
        }
        Ok(())
    }
}
