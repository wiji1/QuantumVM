use crate::lexer::Span;
use crate::parser::expression::Expr;
use crate::parser::supporting_types::*;

#[derive(Debug, Clone)]
pub enum StmtKind {
    QuantumDecl { name: String, name_span: Span, size: Option<Expr> },
    ClassicalDecl { ty: ClassicalType, name: String, name_span: Span, init: Option<Expr> },
    ArrayDecl { ty: ClassicalType, name: String, name_span: Span, size: Vec<Expr>, init: Option<Expr> },
    ConstDecl { ty: ClassicalType, name: String, name_span: Span, init: Expr },
    IoDecl { direction: IoDirection, ty: ClassicalType, name: String, name_span: Span },

    GateCall { name: String, modifiers: Vec<GateModifier>, params: Vec<Expr>, qubits: Vec<GateOperand> },
    ExpressionStatement(Expr),
    Reset { qubit: GateOperand },
    Barrier { qubits: Vec<GateOperand> },

    If { cond: Expr, then: Box<Vec<Stmt>>, else_: Option<Vec<Stmt>> },
    Switch { expr: Expr, cases: Vec<SwitchCase> },
    For { var: String, ty: ClassicalType, iter: ForIter, body: Box<Vec<Stmt>> },
    While { cond: Expr, body: Box<Vec<Stmt>> },
    Continue,
    Break,
    Return(Option<Expr>),

    Def { name: String, name_span: Span, params: Vec<Param>, return_type: Option<ClassicalType>, body: Vec<Stmt> },
    GateDef { name: String, name_span: Span, params: Vec<String>, qubits: Vec<String>, body: Vec<Stmt> },

    Assign { target: IndexedIdent, op: AssignOp, value: Expr },
    Let { name: String, name_span: Span, value: Expr },
    Extern { name: String },

    Block(Vec<Stmt>),
    Include(String),
    IncludeFromSrc(String, String),
    Pragma,
    NoOp
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }
}