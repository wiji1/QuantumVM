use crate::parser::expression::Expr;
use crate::parser::supporting_types::{AssignOp, ClassicalType, ForIter, GateOperand, IndexedIdent};
use crate::lexer::type_def::TypeDefinition;

#[derive(Debug, Clone)]
pub enum Stmt {
    QuantumDecl { name: String, size: Option<Expr> },
    ClassicalDecl { ty: TypeDefinition, name: String, size: Option<Expr>, init: Option<Expr> },
    ConstDecl { ty: ClassicalType, name: String, init: Expr },

    GateCall { name: String, params: Vec<Expr>, qubits: Vec<GateOperand> },
    Measure { qubit: GateOperand, target: Option<IndexedIdent> },
    Reset { qubit: GateOperand },
    Barrier { qubits: Vec<GateOperand> },

    If { cond: Expr, then: Box<Vec<Stmt>>, else_: Option<Vec<Stmt>> },
    For { var: String, ty: ClassicalType, iter: ForIter, body: Box<Vec<Stmt>> },
    While { cond: Expr, body: Box<Vec<Stmt>> },
    Break,
    Continue,
    Return(Option<Expr>),

    GateDef { name: String, params: Vec<String>, qubits: Vec<String>, body: Vec<Stmt> },

    Assign { target: IndexedIdent, op: AssignOp, value: Expr },

    Block(Vec<Stmt>),
}