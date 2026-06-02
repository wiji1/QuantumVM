use crate::parser::expression::Expr;
use crate::parser::supporting_types::*;

#[derive(Debug, Clone)]
pub enum Stmt {
    QuantumDecl { name: String, size: Option<Expr> },
    ClassicalDecl { ty: ClassicalType, name: String, init: Option<Expr> },
    ArrayDecl { ty: ClassicalType, name: String, size: Vec<Expr>, init: Option<Expr> },
    ConstDecl { ty: ClassicalType, name: String, init: Expr },
    IoDecl { direction: IoDirection, ty: ClassicalType, name: String },

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

    Def { name: String, params: Vec<Param>, return_type: Option<ClassicalType>, body: Vec<Stmt> },
    GateDef { name: String, params: Vec<String>, qubits: Vec<String>, body: Vec<Stmt> },

    Assign { target: IndexedIdent, op: AssignOp, value: Expr },
    Let { name: String, value: Expr },
    Extern { name: String },

    Block(Vec<Stmt>),
    Include(String),
    GPhase(Option<Vec<Expr>>),
    Pragma
}