use crate::parser::expression::Expr;
use crate::parser::supporting_types::{AssignOp, ClassicalType, ForIter, GateModifier, GateOperand, IndexedIdent, IoDirection, Param, SwitchCase};
use crate::lexer::type_def::TypeDefinition;

#[derive(Debug, Clone)]
pub enum Stmt {
    QuantumDecl { name: String, size: Option<Expr> },
    ClassicalDecl { ty: TypeDefinition, name: String, size: Option<Expr>, init: Option<Expr> },
    ArrayDecl { ty: TypeDefinition, type_size: Option<Expr>, name: String, size: Vec<Expr>, init: Option<Expr> },
    ConstDecl { ty: TypeDefinition, name: String, size: Option<Expr>, init: Expr },
    IoDecl { direction: IoDirection, ty: TypeDefinition, size: Option<Expr>, name: String },

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

    Def { name: String, params: Vec<Param>, return_type: Option<(TypeDefinition, Option<Expr>)>, body: Vec<Stmt> },
    GateDef { name: String, params: Vec<String>, qubits: Vec<String>, body: Vec<Stmt> },

    Assign { target: IndexedIdent, op: AssignOp, value: Expr },

    Block(Vec<Stmt>),
    Include(String),
}