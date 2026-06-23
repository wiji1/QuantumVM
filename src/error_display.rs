use crate::lexer::keyword::Keyword;
use crate::lexer::literal::Literal;
use crate::lexer::symbol::{CompoundSymbol, Symbol};
use crate::lexer::type_def::TypeDefinition;
use crate::lexer::{Span, TokenType};
use crate::parser::parse_error::ParseError;
use crate::source_cache::SourceCache;
use crate::type_checker::diagnostics::{Diagnostic, DiagnosticSeverity};
use crate::interpreter::runtime_error::RuntimeError;
use ariadne::{Color, Label, Report, ReportKind, Source};
use std::io;

pub struct ErrorReporter {
    cache: SourceCache,
}

impl ErrorReporter {
    pub fn new(cache: SourceCache) -> Self {
        Self { cache }
    }

    pub fn report_parse_errors(&self, filename: &str, errors: &[ParseError]) {
        for error in errors {
            self.report_parse_error(filename, error);
        }
    }

    pub fn report_parse_error(&self, filename: &str, error: &ParseError) {
        let source = match self.cache.get_source(filename) {
            Some(src) => src,
            None => {
                eprintln!("Parse error: {:?}", error);
                return;
            }
        };

        let mut report_builder = Report::build(ReportKind::Error, filename, 0);

        match error {
            ParseError::UnexpectedToken { expected, found, span } => {
                let offset = self.span_to_offset(&source, span);
                let len = span.len.max(1);
                report_builder = report_builder
                    .with_message("Unexpected token")
                    .with_label(
                        Label::new((filename, offset..offset + len))
                            .with_message(format!(
                                "expected {}, found {}",
                                token_display_name(expected),
                                token_display_name(found)
                            ))
                            .with_color(Color::Red)
                    );

                if let Some(hint) = get_unexpected_token_hint(expected, found) {
                    report_builder = report_builder.with_help(hint);
                }
            }
            ParseError::UnexpectedEof { expected } => {
                report_builder = report_builder
                    .with_message("Unexpected end of file")
                    .with_help(format!("Expected {}", token_display_name(expected)));
            }
            ParseError::InvalidLiteral { message, span } => {
                let offset = self.span_to_offset(&source, span);
                let len = span.len.max(1);
                report_builder = report_builder
                    .with_message("Invalid literal")
                    .with_label(
                        Label::new((filename, offset..offset + len))
                            .with_message(message.clone())
                            .with_color(Color::Red)
                    );
            }
            ParseError::InvalidVersion { found, span } => {
                let offset = self.span_to_offset(&source, span);
                let len = span.len.max(1);
                report_builder = report_builder
                    .with_message("Invalid version specifier")
                    .with_label(
                        Label::new((filename, offset..offset + len))
                            .with_message(format!("found '{}'", found))
                            .with_color(Color::Red)
                    )
                    .with_help("Valid versions: 3.0+");
            }
            ParseError::InvalidStatement { found, span } => {
                let offset = self.span_to_offset(&source, span);
                let len = span.len.max(1);
                report_builder = report_builder
                    .with_message("Invalid statement")
                    .with_label(
                        Label::new((filename, offset..offset + len))
                            .with_message(format!("unexpected {}", token_display_name(found)))
                            .with_color(Color::Red)
                    );
            }
            ParseError::TypeError { message, span } => {
                let offset = self.span_to_offset(&source, span);
                let len = span.len.max(1);
                report_builder = report_builder
                    .with_message("Type error during parsing")
                    .with_label(
                        Label::new((filename, offset..offset + len))
                            .with_message(message.clone())
                            .with_color(Color::Red)
                    );
            }
        }

        let report = report_builder.finish();
        let mut writer = io::stderr();
        report.write((filename, Source::from(&**source)), &mut writer).unwrap();
    }

    pub fn report_type_errors(&self, filename: &str, diagnostics: &[Diagnostic]) {
        let source = match self.cache.get_source(filename) {
            Some(src) => src,
            None => {
                for diag in diagnostics {
                    eprintln!("{}", diag);
                }
                return;
            }
        };

        for diag in diagnostics {
            let kind = match diag.severity {
                DiagnosticSeverity::Error => ReportKind::Error,
                DiagnosticSeverity::Warning => ReportKind::Warning,
                DiagnosticSeverity::Info => ReportKind::Advice,
                DiagnosticSeverity::Hint => ReportKind::Advice,
            };

            let color = match diag.severity {
                DiagnosticSeverity::Error => Color::Red,
                DiagnosticSeverity::Warning => Color::Yellow,
                DiagnosticSeverity::Info => Color::Cyan,
                DiagnosticSeverity::Hint => Color::Blue,
            };

            let mut report_builder = Report::build(kind, filename, 0)
                .with_message(&diag.message);

            if let Some(span) = &diag.span {
                let offset = self.span_to_offset(&source, span);
                report_builder = report_builder.with_label(
                    Label::new((filename, offset..offset + span.len))
                        .with_color(color)
                );
            }

            let report = report_builder.finish();
            let mut writer = io::stderr();
            report.write((filename, Source::from(&**source)), &mut writer).unwrap();
        }
    }

    pub fn report_runtime_error(&self, filename: &str, error: &RuntimeError) {
        let source = match self.cache.get_source(filename) {
            Some(src) => src,
            None => {
                eprintln!("Runtime error: {}", error);
                return;
            }
        };

        let mut report_builder = Report::build(ReportKind::Error, filename, 0)
            .with_message(format!("Runtime error: {}", error));

        if let Some(span) = &error.span {
            let offset = self.span_to_offset(&source, span);
            let len = span.len.max(1);
            report_builder = report_builder.with_label(
                Label::new((filename, offset..offset + len))
                    .with_color(Color::Red)
            );
        }

        let report = report_builder.finish();
        let mut writer = io::stderr();
        report.write((filename, Source::from(&**source)), &mut writer).unwrap();
    }

    fn span_to_offset(&self, source: &str, span: &Span) -> usize {
        let mut offset = 0;
        let mut current_line = 0;
        let mut current_col = 0;

        for ch in source.chars() {
            if current_line == span.line && current_col == span.col {
                return offset;
            }

            if ch == '\n' {
                current_line += 1;
                current_col = 0;
            } else { current_col += 1; }

            offset += ch.len_utf8();
        }

        offset.min(source.len())
    }
}

//TODO: Move this into the individual enums
fn token_display_name(token: &TokenType) -> String {
    match token {
        TokenType::Symbol(s) => match s {
            Symbol::LBracket => "'['".to_string(),
            Symbol::RBracket => "']'".to_string(),
            Symbol::Colon => "':'".to_string(),
            Symbol::Semicolon => "';'".to_string(),
            Symbol::Dot => "'.'".to_string(),
            Symbol::Comma => "','".to_string(),
            Symbol::Equals => "'='".to_string(),
            Symbol::Slash => "'/'".to_string(),
            Symbol::NewLine => "newline".to_string(),
            Symbol::Star => "'*'".to_string(),
            Symbol::Plus => "'+'".to_string(),
            Symbol::Minus => "'-'".to_string(),
            Symbol::Tilde => "'~'".to_string(),
            Symbol::Bang => "'!'".to_string(),
            Symbol::Percent => "'%'".to_string(),
            Symbol::LessThan => "'<'".to_string(),
            Symbol::GreaterThan => "'>'".to_string(),
            Symbol::Ampersand => "'&'".to_string(),
            Symbol::Caret => "'^'".to_string(),
            Symbol::Pipe => "'|'".to_string(),
            Symbol::LParen => "'('".to_string(),
            Symbol::RParen => "')'".to_string(),
            Symbol::LBrace => "'{'".to_string(),
            Symbol::RBrace => "'}'".to_string(),
            Symbol::At => "'@'".to_string(),
        },
        TokenType::CompoundSymbol(cs) => match cs {
            CompoundSymbol::LineComment => "line comment".to_string(),
            CompoundSymbol::BlockComment => "block comment".to_string(),
            CompoundSymbol::DoubleAsterisk => "'**'".to_string(),
            CompoundSymbol::BitShiftLeft => "'<<'".to_string(),
            CompoundSymbol::BitShiftRight => "'>>'".to_string(),
            CompoundSymbol::LessThanOrEqual => "'<='".to_string(),
            CompoundSymbol::GreaterThanOrEqual => "'>='".to_string(),
            CompoundSymbol::Equals => "'=='".to_string(),
            CompoundSymbol::NotEquals => "'!='".to_string(),
            CompoundSymbol::And => "'&&'".to_string(),
            CompoundSymbol::Or => "'||'".to_string(),
            CompoundSymbol::Concat => "'++'".to_string(),
            CompoundSymbol::Arrow => "'->'".to_string(),
        },
        TokenType::Keyword(k) => match k {
            Keyword::OpenQASM => "keyword 'OPENQASM'".to_string(),
            Keyword::Measure => "keyword 'measure'".to_string(),
            Keyword::Gate => "keyword 'gate'".to_string(),
            Keyword::If => "keyword 'if'".to_string(),
            Keyword::Else => "keyword 'else'".to_string(),
            Keyword::Reset => "keyword 'reset'".to_string(),
            Keyword::Barrier => "keyword 'barrier'".to_string(),
            Keyword::For => "keyword 'for'".to_string(),
            Keyword::While => "keyword 'while'".to_string(),
            Keyword::In => "keyword 'in'".to_string(),
            Keyword::Const => "keyword 'const'".to_string(),
            Keyword::Continue => "keyword 'continue'".to_string(),
            Keyword::Break => "keyword 'break'".to_string(),
            Keyword::Return => "keyword 'return'".to_string(),
            Keyword::Def => "keyword 'def'".to_string(),
            Keyword::Let => "keyword 'let'".to_string(),
            Keyword::Input => "keyword 'input'".to_string(),
            Keyword::Output => "keyword 'output'".to_string(),
            Keyword::Include => "keyword 'include'".to_string(),
            _ => format!("keyword '{:?}'", k).to_lowercase(),
        },
        TokenType::TypeDef(t) => match t {
            TypeDefinition::Qubit => "type 'qubit'".to_string(),
            TypeDefinition::Bool => "type 'bool'".to_string(),
            TypeDefinition::Bit => "type 'bit'".to_string(),
            TypeDefinition::Int => "type 'int'".to_string(),
            TypeDefinition::UInt => "type 'uint'".to_string(),
            TypeDefinition::Float => "type 'float'".to_string(),
            TypeDefinition::Angle => "type 'angle'".to_string(),
            TypeDefinition::Complex => "type 'complex'".to_string(),
            TypeDefinition::Duration => "type 'duration'".to_string(),
            TypeDefinition::Array => "type 'array'".to_string(),
            TypeDefinition::Void => "type 'void'".to_string(),
            TypeDefinition::Stretch => "type 'stretch'".to_string(),
        },
        TokenType::Identifier(_) => "identifier".to_string(),
        TokenType::Literal(l) => match l {
            Literal::Integer(_) => "integer literal".to_string(),
            Literal::Float(_) => "float literal".to_string(),
            Literal::String(_) => "string literal".to_string(),
            Literal::Bitstring(_) => "bitstring literal".to_string(),
            Literal::Boolean(_) => "boolean literal".to_string(),
            Literal::Imaginary(_) => "imaginary literal".to_string(),
            Literal::Timing(_) => "timing literal".to_string(),
            Literal::HardwareQubit(_) => "hardware qubit".to_string(),
        },
        TokenType::CompoundAssignment(_) => "compound assignment operator".to_string(),
    }
}

fn get_unexpected_token_hint(_expected: &TokenType, _found: &TokenType) -> Option<String> {
    // TODO: Add helpful hints based on common error patterns
    None
}
