pub mod expression;
pub mod statement;
pub mod parse_error;
pub mod supporting_types;

use crate::lexer::identifier::Identifier;
use crate::lexer::keyword::Keyword;
use crate::lexer::literal::Literal;
use crate::lexer::symbol::{CompoundSymbol, Symbol};
use crate::lexer::type_def::TypeDefinition;
use crate::lexer::{Token, TokenType};
use crate::parser::expression::Expr;
use crate::parser::parse_error::ParseError;
use crate::parser::statement::Stmt;
use crate::parser::supporting_types::{ArrayDimensions, AssignOp, ClassicalType, ForIter, GateModifier, GateOperand, IndexedIdent, IoDirection, Param, ParamType, SwitchCase, UnaryOp};

fn matches_token_type(actual: &TokenType, expected: &TokenType) -> bool {
    match (actual, expected) {
        (TokenType::Symbol(a), TokenType::Symbol(b)) => std::mem::discriminant(a) == std::mem::discriminant(b),
        (TokenType::CompoundSymbol(a), TokenType::CompoundSymbol(b)) => std::mem::discriminant(a) == std::mem::discriminant(b),
        (TokenType::Keyword(a), TokenType::Keyword(b)) => std::mem::discriminant(a) == std::mem::discriminant(b),
        (TokenType::TypeDef(a), TokenType::TypeDef(b)) => std::mem::discriminant(a) == std::mem::discriminant(b),
        (TokenType::Identifier(a), TokenType::Identifier(b)) => std::mem::discriminant(a) == std::mem::discriminant(b),
        (TokenType::Literal(a), TokenType::Literal(b)) => std::mem::discriminant(a) == std::mem::discriminant(b),
        _ => false,
    }
}

fn infix_binding_power(op: &TokenType) -> Option<(u8, u8)> {
    let power = match op {
        TokenType::Symbol(s) => { Symbol::infix_binding_power(s) }
        TokenType::CompoundSymbol(s) => { CompoundSymbol::infix_binding_power(s) }
        _ => return None
    };

    power
}

macro_rules! expect_token {
    ($self:expr, $expected:expr) => {{
        let tok = $self.peek().clone();
        if matches_token_type(&tok.kind, &$expected) {
            $self.advance();
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: $expected,
                found: tok.kind,
                span: tok.span,
            });
        }
    }};
}

macro_rules! extract_token {
    ($self:expr, $pat:pat => $val:expr, $expected:expr) => {{
        let tok = $self.peek().clone();
        match tok.kind {
            $pat => {
                $self.advance();
                $val
            }
            _ => return Err(ParseError::UnexpectedToken {
                expected: $expected,
                found: tok.kind,
                span: tok.span,
            }),
        }
    }};
}

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

pub struct Program {
    pub version: Option<f64>,
    pub statements: Vec<Stmt>,
}

impl Parser {
    pub(crate) fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, cursor: 0 }
    }

    pub fn start(&mut self, include_lib: bool) -> Result<Program, ParseError> {
        let version = self.parse_version()?;
        let mut statements = vec![];
        while !self.is_at_end() {
            self.skip_trivia();
            if self.is_at_end() { break; }
            statements.push(self.parse_statement()?);
        }

        if include_lib {
            let lib_name = "stdgates.inc";
            let stdgates_source = include_str!("../lib/stdgates.inc");
            statements.insert(0, Stmt::IncludeFromSrc(lib_name.to_string(), stdgates_source.to_string()));
        }

        Ok(Program { version, statements })
    }

    fn peek(&self) -> &Token { &self.tokens[self.cursor.min(self.tokens.len() - 1)] }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.cursor];
        self.cursor += 1;
        token
    }

    fn at(&self, kind: TokenType) -> bool {
        matches_token_type(&self.peek().kind, &kind)
    }

    fn is_at_end(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn skip_trivia(&mut self) {
        while !self.is_at_end() {
            match self.peek().kind {
                TokenType::Symbol(Symbol::NewLine) => { self.advance(); }
                TokenType::CompoundSymbol(CompoundSymbol::LineComment) => { self.advance(); }
                _ => break,
            }
        }
    }

    fn parse_version(&mut self) -> Result<Option<f64>, ParseError> {
        self.skip_trivia();
        if !self.at(TokenType::Keyword(Keyword::OpenQASM)) {
            return Ok(None);
        }
        self.advance();

        let version = extract_token!(
            self,
            TokenType::Literal(Literal::Float(v)) => v,
            TokenType::Literal(Literal::Float(0.0))
        );

        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));
        Ok(Some(version))
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek().clone();

        let parsed = match token.kind {
            TokenType::Literal(l) => {
                self.advance();
                let literal = match l {
                    Literal::Integer(i) => { Expr::Int(i) }
                    Literal::Float(f) => { Expr::Float(f) }
                    Literal::Boolean(b) => { Expr::Bool(b) }
                    Literal::Imaginary(i) => { Expr::Imaginary(i) }
                    Literal::Timing(t) => { Expr::Timing(t) }
                    Literal::Bitstring(s) => {
                        let width = s.len();
                        let value = u64::from_str_radix(&s, 2).map_err(|_| ParseError::InvalidLiteral {
                            message: format!("invalid bitstring: {}", s),
                            span: token.span.clone(),
                        })?;
                        Expr::Bits(value, width)
                    }
                    Literal::String(s) => {
                        if s.chars().all(|c| c == '0' || c == '1') {
                            let width = s.len();
                            let value = u64::from_str_radix(&s, 2).map_err(|_| ParseError::InvalidLiteral {
                                message: format!("invalid bitstring: {}", s),
                                span: token.span.clone(),
                            })?;
                            Expr::Bits(value, width)
                        } else {
                            return Err(ParseError::InvalidLiteral {
                                message: format!("invalid bitstring: {}", s),
                                span: token.span.clone(),
                            });
                        }
                    }
                    _ => {
                        return Err(ParseError::InvalidLiteral {
                            message: "not valid in expression context".to_string(),
                            span: token.span.clone(),
                        })
                    }
                };

                literal
            }
            TokenType::Identifier(Identifier::Identifier(s)) => {
                let s = s.clone();
                self.advance();

                if self.at(TokenType::Symbol(Symbol::LParen)) {
                    self.advance();
                    let mut args = vec![];
                    while !self.at(TokenType::Symbol(Symbol::RParen)) && !self.is_at_end() {
                        args.push(self.parse_expr(0)?);
                        if self.at(TokenType::Symbol(Symbol::Comma)) { self.advance(); }
                    }
                    expect_token!(self, TokenType::Symbol(Symbol::RParen));
                    Expr::Call { name: s, args }
                } else if self.at(TokenType::Symbol(Symbol::LBracket)) {
                    let indices = self.parse_index_operands()?;

                    if indices.is_empty() { Expr::Ident(s) }
                    else { Expr::IndexedIdent(IndexedIdent { name: s, indices }) }
                } else { Expr::Ident(s) }
            }
            TokenType::Symbol(Symbol::Minus)
            | TokenType::Symbol(Symbol::Tilde)
            | TokenType::Symbol(Symbol::Bang) => {
                let op = match token.kind {
                    TokenType::Symbol(Symbol::Minus) => UnaryOp::Neg,
                    TokenType::Symbol(Symbol::Tilde) => UnaryOp::BitNot,
                    TokenType::Symbol(Symbol::Bang)  => UnaryOp::LogicNot,
                    _ => unreachable!()
                };
                self.advance();

                let bp = match &token.kind {
                    TokenType::Symbol(s) => s.prefix_binding_power().unwrap(),
                    _ => unreachable!()
                };
                let expr = self.parse_expr(bp)?;
                Expr::Unary { op, expr: Box::new(expr) }
            }
            TokenType::Symbol(Symbol::LParen) => {
                self.advance();
                self.skip_trivia();
                let expr = self.parse_expr(0)?;
                expect_token!(self, TokenType::Symbol(Symbol::RParen));
                expr
            }
            TokenType::Symbol(Symbol::LBrace) => {
                self.advance();

                let mut values = vec![];

                loop {
                    self.skip_trivia();

                    if self.at(TokenType::Symbol(Symbol::RBrace)) { break; }

                    if self.is_at_end() { break; }

                    values.push(self.parse_expr(0)?);

                    self.skip_trivia();

                    if self.at(TokenType::Symbol(Symbol::Comma)) { self.advance(); }
                }

                expect_token!(self, TokenType::Symbol(Symbol::RBrace));
                Expr::Array(Box::from(values))
            }
            TokenType::TypeDef(type_def) => {
                self.advance();
                let size = self.extract_index_operand()?;

                expect_token!(self, TokenType::Symbol(Symbol::LParen));
                let expr = self.parse_expr(0)?;
                expect_token!(self, TokenType::Symbol(Symbol::RParen));

                let classical_type = type_def.get_classical_type(size);

                match classical_type {
                    Some(classical_type) => {
                        Expr::Cast { ty: Box::new(classical_type), expr: Box::new(expr) }
                    }
                    None => return Err(ParseError::TypeError {
                        message: "type cannot be cast to".to_string(),
                        span: token.span
                    })
                }
            }
            _ => return Err(ParseError::UnexpectedToken {
                expected: TokenType::Literal(Literal::Integer(0)),
                found: token.kind.clone(),
                span: token.span.clone(),
            })
        };

        Ok(parsed)
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_atom()?;

        loop {
            let operator = self.peek().clone();
            let binary_operator = match &operator.kind {
                TokenType::Symbol(s) => s.to_binary_operator(),
                TokenType::CompoundSymbol(s) => s.to_binary_operator(),
                _ => None
            };

            let power = infix_binding_power(&operator.kind);

            let (left_bp, right_bp) = match power {
                None => { break; }
                Some(bp) => { bp }
            };

            if left_bp < min_bp { break; }

            self.advance();
            self.skip_trivia();

            let right = self.parse_expr(right_bp)?;
            left = Expr::Binary { op: binary_operator.unwrap(), lhs: Box::from(left), rhs: Box::from(right) }
        }

        Ok(left)
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        expect_token!(self, TokenType::Symbol(Symbol::LBrace));
        let mut stmts = vec![];

        while !self.at(TokenType::Symbol(Symbol::RBrace)) && !self.is_at_end() {
            self.skip_trivia();
            if self.at(TokenType::Symbol(Symbol::RBrace)) { break; }
            stmts.push(self.parse_statement()?);
        }

        expect_token!(self, TokenType::Symbol(Symbol::RBrace));
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        self.skip_trivia();

        match self.peek().kind {
            TokenType::TypeDef(TypeDefinition::Qubit) => self.parse_quantum_decl(),
            TokenType::TypeDef(TypeDefinition::Array) => self.parse_array_decl(),
            TokenType::TypeDef(_) => self.parse_classic_decl(),
            TokenType::CompoundAssignment(_) => self.parse_assign(),
            TokenType::Keyword(Keyword::Const) => self.parse_const_decl(),
            TokenType::Keyword(Keyword::Gate) => self.parse_gate_def(),
            TokenType::Keyword(Keyword::If) => self.parse_if(),
            TokenType::Keyword(Keyword::Reset) => self.parse_reset(),
            TokenType::Keyword(Keyword::Barrier) => self.parse_barrier(),
            TokenType::Keyword(Keyword::While) => self.parse_while(),
            TokenType::Keyword(Keyword::For) => self.parse_for(),
            TokenType::Keyword(Keyword::Continue) => self.parse_continue(),
            TokenType::Keyword(Keyword::Break) => self.parse_break(),
            TokenType::Keyword(Keyword::Return) => self.parse_return(),
            TokenType::Keyword(Keyword::Def) => self.parse_def(),
            TokenType::Keyword(Keyword::Switch) => self.parse_switch(),
            TokenType::Keyword(Keyword::Include) => self.parse_include(),
            TokenType::Keyword(Keyword::Measure) => self.parse_arrow_measure(),
            TokenType::Keyword(Keyword::Let) => self.parse_let(),
            TokenType::Keyword(Keyword::GPhase) => self.parse_gphase(),
            TokenType::Keyword(Keyword::CReg) => self.parse_creg(),
            TokenType::Keyword(Keyword::QReg) => self.parse_qreg(),
            TokenType::Keyword(Keyword::Extern) => self.parse_extern(),
            TokenType::Keyword(Keyword::Pragma) => self.parse_pragma(),
            TokenType::Keyword(Keyword::Delay) => self.parse_delay(),
            TokenType::Keyword(Keyword::Cal) => self.parse_cal_block(false),
            TokenType::Keyword(Keyword::DefCal) => self.parse_cal_block(true),
            TokenType::Keyword(Keyword::Input) | TokenType::Keyword(Keyword::Output) => self.parse_io_decl(),
            TokenType::Keyword(Keyword::Inv) | TokenType::Keyword(Keyword::Pow) |
            TokenType::Keyword(Keyword::Ctrl) | TokenType::Keyword(Keyword::NegCtrl) => self.parse_gate_call(),
            TokenType::Keyword(Keyword::Box) => {
                self.advance();
                let stmts = self.parse_block()?;
                Ok(Stmt::Block(stmts))
            }
            TokenType::Symbol(Symbol::LBrace) => {
                let stmts = self.parse_block()?;
                Ok(Stmt::Block(stmts))
            },
            TokenType::Identifier(_) => {
                if self.cursor + 1 >= self.tokens.len() { return self.parse_assign() }
                match &self.tokens[self.cursor + 1].kind {
                    TokenType::Symbol(Symbol::Equals) => self.parse_assign(),
                    TokenType::Symbol(Symbol::LBracket) => self.parse_assign(),
                    TokenType::CompoundAssignment(_) => self.parse_assign(),
                    TokenType::Symbol(Symbol::LParen) => {
                        if self.has_qubit_operands_after_params() { self.parse_gate_call() }
                        else {self.parse_expression_statement() }
                    },
                    _ => self.parse_gate_call()
                }
            },
            _ => {
                Err(ParseError::InvalidStatement {
                    found: self.peek().kind.clone(),
                    span: self.peek().span.clone(),
                })
            }
        }
    }

    fn parse_statement_or_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.skip_trivia();
        if self.at(TokenType::Symbol(Symbol::LBrace)) { self.parse_block() }
        else { Ok(vec![self.parse_statement()?]) }
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.skip_trivia();
        expect_token!(self, TokenType::Keyword(Keyword::If));

        expect_token!(self, TokenType::Symbol(Symbol::LParen));
        let condition = self.parse_expr(0)?;
        expect_token!(self, TokenType::Symbol(Symbol::RParen));

        let then = self.parse_statement_or_block()?;

        let mut else_: Option<Vec<Stmt>> = None;

        if self.at(TokenType::Keyword(Keyword::Else)) {
            self.advance();

            else_ = Some(self.parse_statement_or_block()?);
        }

        Ok(Stmt::If {
            cond: condition,
            then: Box::from(then),
            else_,
        })
    }

    fn parse_quantum_decl(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::TypeDef(TypeDefinition::Qubit));

        let size = self.extract_index_operand()?;

        let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );

        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));
        Ok(Stmt::QuantumDecl { name, size })
    }

    fn parse_classical_type(&mut self) -> Result<ClassicalType, ParseError> {
        let type_def = extract_token!(
            self,
            TokenType::TypeDef(t) => t,
            TokenType::TypeDef(TypeDefinition::Int)
        );

        match type_def {
            TypeDefinition::Complex => {
                expect_token!(self, TokenType::Symbol(Symbol::LBracket));
                let inner = self.parse_classical_type()?;
                expect_token!(self, TokenType::Symbol(Symbol::RBracket));
                Ok(ClassicalType::Complex(Some(Box::new(inner))))
            }
            TypeDefinition::Duration => {
                let _ = self.extract_index_operand()?;
                Ok(ClassicalType::Duration(None))
            }
            TypeDefinition::Stretch => {
                let _ = self.extract_index_operand()?;
                Ok(ClassicalType::Stretch(None))
            }
            _ => {
                let size = self.extract_index_operand()?;
                let classical_type = type_def.get_classical_type(size);

                match classical_type {
                    Some(classical_type) => Ok(classical_type),
                    None => Err(ParseError::TypeError {
                        message: format!("cannot convert type: {:?} to classical type", type_def),
                        span: self.peek().span.clone(),
                    })
                }

            }
        }
    }

    fn parse_classic_decl(&mut self) -> Result<Stmt, ParseError> {
        let classical_type = self.parse_classical_type()?;

        let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );

        let init = if self.at(TokenType::Symbol(Symbol::Equals)) {
            self.advance();
            self.skip_trivia();
            Some(self.parse_expr(0)?)
        } else { None };

        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));
        Ok(Stmt::ClassicalDecl { ty: classical_type, name, init })
    }

    fn parse_const_decl(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        let classical_type = self.parse_classical_type()?;

        let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );

        expect_token!(self, TokenType::Symbol(Symbol::Equals));
        self.skip_trivia();

        let init = self.parse_expr(0)?;

        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));
        Ok(Stmt::ConstDecl { ty: classical_type, name, init })
    }

    fn parse_gate_def(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::Keyword(Keyword::Gate));

        let (name, params, qubits) = self.parse_gate_header()?;
        let body = self.parse_block()?;

        Ok(Stmt::GateDef { name, params, qubits, body })
    }

    fn parse_gate_header(&mut self) -> Result<(String, Vec<String>, Vec<String>), ParseError> {
        let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );

        let mut params = vec![];
        if self.at(TokenType::Symbol(Symbol::LParen)) {
            self.advance();
            params = self.parse_identifier_list(TokenType::Symbol(Symbol::RParen))?;
            expect_token!(self, TokenType::Symbol(Symbol::RParen));
        }

        let qubits = self.parse_identifier_list(TokenType::Symbol(Symbol::LBrace))?;

        Ok((name, params, qubits))
    }

    fn parse_identifier_list(&mut self, terminator: TokenType) -> Result<Vec<String>, ParseError> {
        let mut items = vec![];
        while !self.at(terminator.clone()) && !self.is_at_end() {
            self.skip_trivia();
            if self.at(terminator.clone()) { break; }

            let s = match self.peek().kind.clone() {
                TokenType::Identifier(Identifier::Identifier(s)) => {
                    self.advance();
                    s
                }
                TokenType::Keyword(k) => {
                    let s = format!("{:?}", k);
                    self.advance();
                    s
                }
                _ => return Err(ParseError::UnexpectedToken {
                    expected: TokenType::Identifier(Identifier::Identifier(String::new())),
                    found: self.peek().kind.clone(),
                    span: self.peek().span.clone(),
                })
            };
            items.push(s);
            if self.at(TokenType::Symbol(Symbol::Comma)) { self.advance(); }
        }
        Ok(items)
    }

    fn parse_gate_operand_list(&mut self, terminator: TokenType) -> Result<Vec<GateOperand>, ParseError> {
        let mut qubits: Vec<GateOperand> = vec!();

        while !self.at(terminator.clone()) && !self.is_at_end() {
            self.skip_trivia();
            let operand = self.parse_gate_operand()?;

            qubits.push(operand);

            if !self.at(TokenType::Symbol(Symbol::Comma)) && !self.at(terminator.clone()) {
                return Err(ParseError::InvalidStatement {
                    found: self.peek().kind.clone(),
                    span: self.peek().span.clone(),
                })
            }

            if self.at(TokenType::Symbol(Symbol::Comma)) { self.advance(); }
        }

        Ok(qubits)
    }

    fn parse_expression_list(&mut self, terminator: TokenType) -> Result<Vec<Expr>, ParseError> {
        let mut exprs: Vec<Expr> = vec![];

        while !self.at(terminator.clone()) && !self.is_at_end() {
            exprs.push(self.parse_expr(0)?);

            if self.at(TokenType::Symbol(Symbol::Comma)) { self.advance(); }
        }

        Ok(exprs)
    }

    fn parse_gate_operand(&mut self) -> Result<GateOperand, ParseError> {
        if let TokenType::Literal(Literal::HardwareQubit(n)) = self.peek().kind.clone() {
            self.advance();
            return Ok(GateOperand::HardwareQubit(n as u32));
        }

        let name = match self.peek().kind.clone() {
            TokenType::Identifier(Identifier::Identifier(s)) => {
                self.advance();
                s
            }
            TokenType::Keyword(k) => {
                let s = format!("{:?}", k);
                self.advance();
                s
            }
            _ => return Err(ParseError::UnexpectedToken {
                expected: TokenType::Identifier(Identifier::Identifier(String::new())),
                found: self.peek().kind.clone(),
                span: self.peek().span.clone(),
            })
        };

        let indices = self.parse_index_operands()?;

        Ok(GateOperand::Ident(IndexedIdent { name, indices }))
    }

    fn parse_gate_call(&mut self) -> Result<Stmt, ParseError> {
        let mut modifiers = vec![];
        while self.is_gate_modifier() {
            modifiers.push(self.parse_gate_modifier()?);
        }

        let identifier = match self.peek().kind.clone() {
            TokenType::Identifier(Identifier::Identifier(s)) => {
                self.advance();
                s
            }
            TokenType::Keyword(Keyword::GPhase) => {
                self.advance();
                "gphase".to_string()
            }
            _ => return Err(ParseError::UnexpectedToken {
                expected: TokenType::Identifier(Identifier::Identifier(String::new())),
                found: self.peek().kind.clone(),
                span: self.peek().span.clone(),
            })
        };

        let mut params: Vec<Expr> = vec![];

        if self.at(TokenType::Symbol(Symbol::LParen)) {
            self.advance();
            params = self.parse_expression_list(TokenType::Symbol(Symbol::RParen))?;
            expect_token!(self, TokenType::Symbol(Symbol::RParen));
        }

        let qubits: Vec<GateOperand> = self.parse_gate_operand_list(TokenType::Symbol(Symbol::Semicolon))?;
        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

        Ok(Stmt::GateCall { name: identifier, modifiers, params, qubits})
    }

    fn is_gate_modifier(&self) -> bool {
        matches!(self.peek().kind,
            TokenType::Keyword(Keyword::Inv)
            | TokenType::Keyword(Keyword::Pow)
            | TokenType::Keyword(Keyword::Ctrl)
            | TokenType::Keyword(Keyword::NegCtrl)
        )
    }

    fn parse_gate_modifier(&mut self) -> Result<GateModifier, ParseError> {
        let modifier = match self.peek().kind.clone() {
            TokenType::Keyword(Keyword::Inv) => {
                self.advance();
                GateModifier::Inv
            }
            TokenType::Keyword(Keyword::Pow) => {
                self.advance();
                expect_token!(self, TokenType::Symbol(Symbol::LParen));
                let expr = self.parse_expr(0)?;
                expect_token!(self, TokenType::Symbol(Symbol::RParen));
                GateModifier::Pow(expr)
            }
            TokenType::Keyword(Keyword::Ctrl) | TokenType::Keyword(Keyword::NegCtrl) => {
                let token = self.advance().kind.clone();
                let count = if self.at(TokenType::Symbol(Symbol::LParen)) {
                    self.advance();
                    let expr = self.parse_expr(0)?;
                    expect_token!(self, TokenType::Symbol(Symbol::RParen));
                    Some(expr)
                } else {
                    None
                };

                match token {
                    TokenType::Keyword(Keyword::Ctrl) => GateModifier::Ctrl(count),
                    TokenType::Keyword(Keyword::NegCtrl) => GateModifier::NegCtrl(count),
                    _ => unreachable!(),
                }
            }
            _ => return Err(ParseError::InvalidStatement {
                found: self.peek().kind.clone(),
                span: self.peek().span.clone(),
            })
        };

        expect_token!(self, TokenType::Symbol(Symbol::At));
        Ok(modifier)
    }

    fn has_qubit_operands_after_params(&self) -> bool {
        let mut i = self.cursor + 2;
        let mut depth = 0;

        while i < self.tokens.len() {
            let kind = &self.tokens[i].kind;
            i += 1;

            if matches!(kind, TokenType::Symbol(Symbol::LParen)) {
                depth += 1;
                continue;
            }
            if !matches!(kind, TokenType::Symbol(Symbol::RParen)) { continue; }
            if depth > 0 {
                depth -= 1;
                continue;
            }

            return matches!(
                self.tokens.get(i).map(|t| &t.kind),
                Some(TokenType::Identifier(_) | TokenType::Symbol(Symbol::LBracket))
            );
        }
        false
    }

    fn extract_index_operand(&mut self) -> Result<Option<Expr>, ParseError> {
        if self.at(TokenType::Symbol(Symbol::LBracket)) {
            self.advance();
            self.skip_trivia();
            let expr = self.parse_expr(0)?;
                expect_token!(self, TokenType::Symbol(Symbol::RBracket));
            Ok(Some(expr))
        } else {
            Ok(None)
        }
    }

    fn parse_assign(&mut self) -> Result<Stmt, ParseError> {
        let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );

        let indices = self.parse_index_operands()?;

        let op = match &self.peek().kind {
            TokenType::Symbol(Symbol::Equals) => { self.advance(); AssignOp::Eq }
            TokenType::CompoundAssignment(ca) => {
                let bin_op = ca.get_binary_op();
                self.advance();
                AssignOp::Compound(bin_op)
            }
            kind => return Err(ParseError::UnexpectedToken {
                expected: TokenType::Symbol(Symbol::Equals),
                found: kind.clone(),
                span: self.peek().span.clone(),
            })
        };

        let value = if self.at(TokenType::Keyword(Keyword::Measure)) {
            self.advance();

            let qubit = self.parse_gate_operand()?;
            Expr::Measure(Box::new(qubit))
        } else {
            self.parse_expr(0)?
        };

        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

        Ok(Stmt::Assign {
            target: IndexedIdent { name, indices },
            op,
            value
        })
    }

    fn parse_reset(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::Keyword(Keyword::Reset));

        let operand = self.parse_gate_operand()?;
        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

        Ok(Stmt::Reset { qubit: operand })
    }

    fn parse_barrier(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::Keyword(Keyword::Barrier));

        let identifier_list = self.parse_gate_operand_list(TokenType::Symbol(Symbol::Semicolon))?;
        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

        Ok(Stmt::Barrier { qubits: identifier_list })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::Keyword(Keyword::While));

        expect_token!(self, TokenType::Symbol(Symbol::LParen));
        let condition = self.parse_expr(0)?;
        expect_token!(self, TokenType::Symbol(Symbol::RParen));

        let body = self.parse_statement_or_block()?;

        Ok(Stmt::While { cond: condition, body: Box::from(body) })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::Keyword(Keyword::For));

        let type_def = extract_token!(
            self,
            TokenType::TypeDef(t) => t,
            TokenType::TypeDef(TypeDefinition::Int)
        );

        let classical_type = match type_def.get_classical_type(self.extract_index_operand()?) {
            Some(classical_type) => classical_type,
            None => return Err(ParseError::InvalidStatement {
                found: self.peek().kind.clone(),
                span: self.peek().span.clone(),
            })
        };

        let identifier = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );

        expect_token!(self, TokenType::Keyword(Keyword::In));

        let iter: Result<ForIter, ParseError> = match self.peek().kind {
            TokenType::Symbol(Symbol::LBrace) => {
                expect_token!(self, TokenType::Symbol(Symbol::LBrace));
                let exprs = self.parse_expression_list(TokenType::Symbol(Symbol::RBrace))?;
                expect_token!(self, TokenType::Symbol(Symbol::RBrace));

                Ok(ForIter::Set(exprs))
            }
            TokenType::Symbol(Symbol::LBracket) => {
                expect_token!(self, TokenType::Symbol(Symbol::LBracket));
                let range = self.parse_range_expression(None)?;
                expect_token!(self, TokenType::Symbol(Symbol::RBracket));
                Ok(ForIter::Range {
                    start: if let Expr::Range { start, .. } = &range { start.clone() } else { None },
                    stop: if let Expr::Range { stop, .. } = &range { stop.clone() } else { None },
                    step: if let Expr::Range { step, .. } = &range { step.clone() } else { None },
                })
            }
            TokenType::Identifier(_) => {
                let expr = self.parse_expr(0)?;

                Ok(ForIter::Expr(expr))
            }
            _ => {
                Err(ParseError::UnexpectedToken {
                    expected: TokenType::Symbol(Symbol::LBracket),
                    found: self.peek().kind.clone(),
                    span: self.peek().span.clone(),
                })
            }
        };

        let body = self.parse_statement_or_block()?;

        Ok(Stmt::For {
           var: identifier,
           ty: classical_type,
           iter: iter?,
           body: Box::new(body),
        })
    }

    fn parse_continue(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::Keyword(Keyword::Continue));
        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));
        Ok(Stmt::Continue)
    }

    fn parse_break(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::Keyword(Keyword::Break));
        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));
        Ok(Stmt::Break)
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::Keyword(Keyword::Return));

        let expr = match self.peek().kind {
            TokenType::Symbol(Symbol::Semicolon) => { None }
            TokenType::Keyword(Keyword::Measure) => {
                self.advance();
                let qubit = self.parse_gate_operand()?;
                Some(Expr::Measure(Box::new(qubit)))
            }
            _ => { Some(self.parse_expr(0)?) }
        };

        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

        Ok(Stmt::Return(expr))
    }

    fn parse_array_decl(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::TypeDef(TypeDefinition::Array));
        expect_token!(self, TokenType::Symbol(Symbol::LBracket));

        let classical_type = self.parse_classical_type()?;

        expect_token!(self, TokenType::Symbol(Symbol::Comma));

        let expressions = self.parse_expression_list(TokenType::Symbol(Symbol::RBracket))?;
        expect_token!(self, TokenType::Symbol(Symbol::RBracket));

        let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );


        let init = if self.at(TokenType::Symbol(Symbol::Equals)) {
            self.advance();
            self.skip_trivia();
            Some(self.parse_expr(0)?)
        } else { None };

        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

        let stmt = Stmt::ArrayDecl {
            ty: classical_type,
            name: name,
            size: expressions,
            init: init,
        };

        Ok(stmt)
    }

    fn parse_expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );

        expect_token!(self, TokenType::Symbol(Symbol::LParen));
        let args = self.parse_expression_list(TokenType::Symbol(Symbol::RParen))?;
        expect_token!(self, TokenType::Symbol(Symbol::RParen));
        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

        let call = Expr::Call {
            name,
            args,
        };

        Ok(Stmt::ExpressionStatement(call))
    }

    fn parse_def(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::Keyword(Keyword::Def));

        let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );

        expect_token!(self, TokenType::Symbol(Symbol::LParen));

        let mut params = vec![];
        while !self.at(TokenType::Symbol(Symbol::RParen)) && !self.is_at_end() {
            params.push(self.parse_param()?);
            if self.at(TokenType::Symbol(Symbol::Comma)) { self.advance(); }
        }

        expect_token!(self, TokenType::Symbol(Symbol::RParen));

        let return_type = if self.at(TokenType::CompoundSymbol(CompoundSymbol::Arrow)) {
            self.advance();
            Some(self.parse_classical_type()?)
        } else { None };

        let body = self.parse_block()?;

        Ok(Stmt::Def { name, params, return_type, body })
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        match self.peek().kind.clone() {
            TokenType::TypeDef(TypeDefinition::Qubit) => {
                self.advance();
                let size = self.extract_index_operand()?;
                let name = extract_token!(
                    self,
                    TokenType::Identifier(Identifier::Identifier(s)) => s,
                    TokenType::Identifier(Identifier::Identifier(String::new()))
                );
                Ok(Param { ty: ParamType::Qubit(size), name })
            }

            TokenType::TypeDef(_) => {
                let classical_type = self.parse_classical_type()?;

                let name = extract_token!(
                    self,
                    TokenType::Identifier(Identifier::Identifier(s)) => s,
                    TokenType::Identifier(Identifier::Identifier(String::new()))
                );
                Ok(Param { ty: ParamType::Scalar(classical_type), name })
            }

            TokenType::Keyword(Keyword::ReadOnly) | TokenType::Keyword(Keyword::Mutable) => {
                let mutable = matches!(self.peek().kind, TokenType::Keyword(Keyword::Mutable));
                self.advance();
                expect_token!(self, TokenType::TypeDef(TypeDefinition::Array));
                expect_token!(self, TokenType::Symbol(Symbol::LBracket));
                let element_ty = self.parse_classical_type()?;

                let dimensions = if self.at(TokenType::Keyword(Keyword::Dim)) {
                    self.advance();
                    expect_token!(self, TokenType::Symbol(Symbol::Equals));
                    let expr = self.parse_expr(0)?;
                    ArrayDimensions::Dim(expr)
                } else {
                    expect_token!(self, TokenType::Symbol(Symbol::Comma));
                    let exprs = self.parse_expression_list(TokenType::Symbol(Symbol::RBracket))?;
                    ArrayDimensions::Explicit(exprs)
                };

                expect_token!(self, TokenType::Symbol(Symbol::RBracket));

                let name = extract_token!(
                    self,
                    TokenType::Identifier(Identifier::Identifier(s)) => s,
                    TokenType::Identifier(Identifier::Identifier(String::new()))
                );

                Ok(Param {
                    ty: ParamType::ArrayRef { mutable, element_ty, dimensions },
                    name,
                })
            }

            _ => Err(ParseError::UnexpectedToken {
                expected: TokenType::TypeDef(TypeDefinition::Int),
                found: self.peek().kind.clone(),
                span: self.peek().span.clone(),
            })
        }
    }

    fn parse_switch(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        expect_token!(self, TokenType::Symbol(Symbol::LParen));
        let expr = self.parse_expr(0)?;
        expect_token!(self, TokenType::Symbol(Symbol::RParen));
        expect_token!(self, TokenType::Symbol(Symbol::LBrace));

        let mut cases: Vec<SwitchCase> = vec![];
        while !self.at(TokenType::Symbol(Symbol::RBrace)) && !self.is_at_end() {
            self.skip_trivia();
            if self.at(TokenType::Symbol(Symbol::RBrace)) { break; }
            cases.push(self.parse_switch_case()?)
        }

        expect_token!(self, TokenType::Symbol(Symbol::RBrace));
        Ok(Stmt::Switch { expr, cases })
    }

    fn parse_switch_case(&mut self) -> Result<SwitchCase, ParseError> {
        let values: Vec<Expr> = match self.peek().kind {
            TokenType::Keyword(Keyword::Case) => {
                self.advance();
                self.parse_expression_list(TokenType::Symbol(Symbol::LBrace))?
            },
            TokenType::Keyword(Keyword::Default) => {
                self.advance();
                vec![]
            }
            _ => {
                return Err(ParseError::InvalidStatement {
                    found: self.peek().kind.clone(),
                    span: self.peek().span.clone(),
                });
            }
        };

        let body = self.parse_block()?;
        Ok(SwitchCase { values, body })
    }

    fn parse_io_decl(&mut self) -> Result<Stmt, ParseError> {
        let direction = match self.peek().kind {
            TokenType::Keyword(Keyword::Input) => IoDirection::Input,
            TokenType::Keyword(Keyword::Output) => IoDirection::Output,
            _ => unreachable!()
        };

        self.advance();

        let classical_type = self.parse_classical_type()?;

        let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );

        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

        Ok(Stmt::IoDecl { direction, ty: classical_type, name })
    }

    fn parse_include(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let path = extract_token!(
            self,
            TokenType::Literal(Literal::String(s)) => s,
            TokenType::Literal(Literal::String(String::new()))
        );

        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

        Ok(Stmt::Include(path))
    }

    fn parse_arrow_measure(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::Keyword(Keyword::Measure));

        let qubit = self.parse_gate_operand()?;

        if self.at(TokenType::CompoundSymbol(CompoundSymbol::Arrow)) {
            self.advance();
            let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );

            let indices = self.parse_index_operands()?;

            expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

            Ok(Stmt::Assign {
                target: IndexedIdent { name, indices },
                op: AssignOp::Eq,
                value: Expr::Measure(Box::new(qubit)),
            })
        } else {
            expect_token!(self, TokenType::Symbol(Symbol::Semicolon));
            Ok(Stmt::ExpressionStatement(Expr::Measure(Box::new(qubit))))
        }
    }
    fn parse_index_operands(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut indices = vec![];
        while self.at(TokenType::Symbol(Symbol::LBracket)) {
            self.advance();

            loop {
                let index_expr = if self.at(TokenType::Symbol(Symbol::Colon)) {
                    self.parse_range_expression(None)?
                } else {
                    let first = self.parse_expr(0)?;
                    if self.at(TokenType::Symbol(Symbol::Colon)) {
                        self.parse_range_expression(Some(first))?
                    } else { first }
                };

                indices.push(index_expr);

                if self.at(TokenType::Symbol(Symbol::Comma)) { self.advance(); }
                else { break; }
            }

            expect_token!(self, TokenType::Symbol(Symbol::RBracket));
        }
        Ok(indices)
    }

    fn parse_range_expression(&mut self, start: Option<Expr>) -> Result<Expr, ParseError> {
        let start = if start.is_some() { start.map(Box::new) }
        else if !self.at(TokenType::Symbol(Symbol::Colon)) {
            Some(Box::new(self.parse_expr(0)?))
        } else { None };

        expect_token!(self, TokenType::Symbol(Symbol::Colon));

        let stop = if !self.at(TokenType::Symbol(Symbol::Colon))
            && !self.at(TokenType::Symbol(Symbol::RBracket)) {
            Some(Box::new(self.parse_expr(0)?))
        } else { None };

        let step = if self.at(TokenType::Symbol(Symbol::Colon)) {
            self.advance();
            if !self.at(TokenType::Symbol(Symbol::RBracket)) {
                Some(Box::new(self.parse_expr(0)?))
            } else { None }
        } else { None };

        Ok(Expr::Range { start, stop, step })
    }

    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        expect_token!(self, TokenType::Keyword(Keyword::Let));

        let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );

        expect_token!(self, TokenType::Symbol(Symbol::Equals));
        self.skip_trivia();
        let value = self.parse_expr(0)?;
        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

        Ok(Stmt::Let { name, value })
    }

    fn parse_gphase(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let mut params = vec![];
        if self.at(TokenType::Symbol(Symbol::LParen)) {
            self.advance();
            params = self.parse_expression_list(TokenType::Symbol(Symbol::RParen))?;
            expect_token!(self, TokenType::Symbol(Symbol::RParen));
        }

        let qubits = if !self.at(TokenType::Symbol(Symbol::Semicolon)) {
            self.parse_gate_operand_list(TokenType::Symbol(Symbol::Semicolon))?
        } else { vec![] };

        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

        Ok(Stmt::GateCall {
            name: "gphase".to_string(),
            modifiers: vec![],
            params,
            qubits
        })
    }

    fn parse_qreg(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );
        let size = self.extract_index_operand()?;
        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));
        Ok(Stmt::QuantumDecl { name, size })
    }

    fn parse_creg(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
            let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );
        let size = self.extract_index_operand()?;
        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));
        Ok(Stmt::ClassicalDecl {
            ty: ClassicalType::Bit(size),
            name,
            init: None
        })
    }

    fn parse_extern(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        let name = extract_token!(
            self,
            TokenType::Identifier(Identifier::Identifier(s)) => s,
            TokenType::Identifier(Identifier::Identifier(String::new()))
        );

        expect_token!(self, TokenType::Symbol(Symbol::LParen));
        let mut depth = 1;
        while depth > 0 && !self.is_at_end() {
            match self.peek().kind {
                TokenType::Symbol(Symbol::LParen) => { depth += 1; self.advance(); }
                TokenType::Symbol(Symbol::RParen) => { depth -= 1; self.advance(); }
                _ => { self.advance(); }
            }
        }

        if self.at(TokenType::CompoundSymbol(CompoundSymbol::Arrow)) {
            self.advance();
            self.advance();
            let _ = self.extract_index_operand()?;
        }

        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));
        Ok(Stmt::Extern { name })
    }

    fn parse_pragma(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        while !self.is_at_end() && !self.at(TokenType::Symbol(Symbol::NewLine)) {
            self.advance();
        }
        Ok(Stmt::Pragma)
    }

    fn parse_delay(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        let _ = self.extract_index_operand()?;
        if !self.at(TokenType::Symbol(Symbol::Semicolon)) {
            self.parse_gate_operand_list(TokenType::Symbol(Symbol::Semicolon))?;
        }
        expect_token!(self, TokenType::Symbol(Symbol::Semicolon));

        Ok(Stmt::NoOp)
    }

    fn parse_cal_block(&mut self, find_brace_first: bool) -> Result<Stmt, ParseError> {
        self.advance();

        if find_brace_first {
            while !self.at(TokenType::Symbol(Symbol::LBrace)) && !self.is_at_end() { self.advance(); }
        }

        expect_token!(self, TokenType::Symbol(Symbol::LBrace));

        let mut depth = 1;
        while depth > 0 && !self.is_at_end() {
            match self.peek().kind {
                TokenType::Symbol(Symbol::LBrace) => { depth += 1; }
                TokenType::Symbol(Symbol::RBrace) => { depth -= 1; }
                _ => {}
            }
            self.advance();
        }

        Ok(Stmt::NoOp)
    }
}