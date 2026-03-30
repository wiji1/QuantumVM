use crate::enums::statement::Stmt;
use crate::parser::Program;

pub struct Interpreter {
    program: Program
}

impl Interpreter {
    pub(crate) fn new(program: Program) -> Interpreter {
        Interpreter { program }
    }

    pub fn start(&mut self) {
        for stmt in self.program.statements.clone() {
            self.interpret_statement(&stmt);
        }
    }

    fn interpret_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::QuantumDecl { .. } => {}
            Stmt::ClassicalDecl { .. } => {}
            Stmt::ConstDecl { .. } => {}
            Stmt::GateCall { .. } => {}
            Stmt::Measure { .. } => {}
            Stmt::Reset { .. } => {}
            Stmt::Barrier { .. } => {}
            Stmt::If { .. } => {}
            Stmt::For { .. } => {}
            Stmt::While { .. } => {}
            Stmt::Break => {}
            Stmt::Continue => {}
            Stmt::Return(_) => {}
            Stmt::GateDef { .. } => {}
            Stmt::Assign { .. } => {}
            Stmt::Block(_) => {}
        }
    }
}