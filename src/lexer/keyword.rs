use crate::lexer::{match_keyword, TokenTrait};

#[derive(Debug, Clone)]
pub enum Keyword {
    OpenQASM,
    Measure,
    Gate,
    If,
    Else,
    Reset,
    Barrier,
    For,
    While,
    In,
    Const,
    Continue,
    Break,
    Return,
    Def,
    ReadOnly,
    Mutable,
    Dim,
    Switch,
    Case,
    Default,
    Input,
    Output,
    Include,
    Inv,
    Pow,
    Ctrl,
    NegCtrl
}

impl TokenTrait for Keyword {
    //TODO: Rewrite this; its nasty code
    fn try_parse(input: &str) -> Option<(Self, usize)> {
        if let Some(len) = match_keyword(input, "OPENQASM") {
            return Some((Self::OpenQASM, len));
        }
        if let Some(len) = match_keyword(input, "measure") {
            return Some((Self::Measure, len));
        }
        if let Some(len) = match_keyword(input, "gate") {
            return Some((Self::Gate, len));
        }
        if let Some(len) = match_keyword(input, "if") {
            return Some((Self::If, len));
        }
        if let Some(len) = match_keyword(input, "else") {
            return Some((Self::Else, len));
        }
        if let Some(len) = match_keyword(input, "reset") {
            return Some((Self::Reset, len));
        }
        if let Some(len) = match_keyword(input, "barrier") {
            return Some((Self::Barrier, len));
        }
        if let Some(len) = match_keyword(input, "for") {
            return Some((Self::For, len));
        }
        if let Some(len) = match_keyword(input, "while") {
            return Some((Self::While, len));
        }
        if let Some(len) = match_keyword(input, "in") {
            return Some((Self::In, len));
        }
        if let Some(len) = match_keyword(input, "const") {
            return Some((Self::Const, len));
        }
        if let Some(len) = match_keyword(input, "continue") {
            return Some((Self::Continue, len));
        }
        if let Some(len) = match_keyword(input, "break") {
            return Some((Self::Break, len));
        }
        if let Some(len) = match_keyword(input, "return") {
            return Some((Self::Return, len));
        }
        if let Some(len) = match_keyword(input, "def") {
            return Some((Self::Def, len));
        }
        if let Some(len) = match_keyword(input, "readonly") {
            return Some((Self::ReadOnly, len));
        }
        if let Some(len) = match_keyword(input, "mutable") {
            return Some((Self::Mutable, len));
        }
        if let Some(len) = match_keyword(input, "dim") {
            return Some((Self::Dim, len));
        }
        if let Some(len) = match_keyword(input, "switch") {
            return Some((Self::Switch, len));
        }
        if let Some(len) = match_keyword(input, "case") {
            return Some((Self::Case, len));
        }
        if let Some(len) = match_keyword(input, "default") {
            return Some((Self::Default, len));
        }
        if let Some(len) = match_keyword(input, "input") {
            return Some((Self::Input, len));
        }
        if let Some(len) = match_keyword(input, "output") {
            return Some((Self::Output, len));
        }
        if let Some(len) = match_keyword(input, "include") {
            return Some((Self::Include, len));
        }
        if let Some(len) = match_keyword(input, "inv") {
            return Some((Self::Inv, len));
        }
        if let Some(len) = match_keyword(input, "pow") {
            return Some((Self::Pow, len));
        }
        if let Some(len) = match_keyword(input, "ctrl") {
            return Some((Self::Ctrl, len));
        }
        if let Some(len) = match_keyword(input, "negctrl") {
            return Some((Self::NegCtrl, len));
        }

        None
    }
}
