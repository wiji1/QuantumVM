use std::sync::LazyLock;
use regex::Regex;
use crate::lexer::{match_keyword, TokenTrait};

static FLOAT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9]*\.[0-9]+([eE][+-]?[0-9]+)?|^[0-9]+[eE][+-]?[0-9]+").unwrap()
});

static IMAGINARY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([0-9]*\.[0-9]+([eE][+-]?[0-9]+)?|[0-9]+([eE][+-]?[0-9]+)?|[0-9]+)im").unwrap()
});

static INTEGER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9]+").unwrap()
});

static STRING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^"([^"\\]|\\.)*""#).unwrap()
});

static BITSTRING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^"[01]+"b"#).unwrap()
});

static TIMING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([0-9]*\.[0-9]+|[0-9]+)(ns|us|µs|ms|s|dt)").unwrap()
});

static HARDWARE_QUBIT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\$[0-9]+").unwrap()
});

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Bitstring(String),
    Boolean(bool),
    Imaginary(f64),
    Timing(String),
    HardwareQubit(i64),
}

impl TokenTrait for Literal {
    fn try_parse(input: &str) -> Option<(Self, usize)> {
        if let Some(m) = BITSTRING.find(input) {
            let raw = m.as_str();
            let inner = &raw[1..raw.len() - 2];
            return Some((Literal::Bitstring(inner.to_string()), m.len()));
        }

        if let Some(m) = STRING.find(input) {
            let raw = m.as_str();
            let inner = &raw[1..raw.len() - 1];
            return Some((Literal::String(inner.to_string()), m.len()));
        }

        if let Some(len) = match_keyword(input, "true") {
            return Some((Literal::Boolean(true), len));
        }
        if let Some(len) = match_keyword(input, "false") {
            return Some((Literal::Boolean(false), len));
        }

        if let Some(m) = TIMING.find(input) {
            return Some((Literal::Timing(m.as_str().to_string()), m.len()));
        }

        if let Some(m) = IMAGINARY.find(input) {
            let raw = m.as_str();
            let num_str = &raw[..raw.len() - 2];
            let value: f64 = num_str.parse().ok()?;
            return Some((Literal::Imaginary(value), m.len()));
        }

        if let Some(m) = FLOAT.find(input) {
            let value: f64 = m.as_str().parse().ok()?;
            return Some((Literal::Float(value), m.len()));
        }

        if let Some(m) = HARDWARE_QUBIT.find(input) {
            let raw = m.as_str();
            let value: i64 = raw[1..].parse().ok()?;
            return Some((Literal::HardwareQubit(value), m.len()));
        }

        if let Some(m) = INTEGER.find(input) {
            let value: i64 = m.as_str().parse().ok()?;
            return Some((Literal::Integer(value), m.len()));
        }

        None
    }
}