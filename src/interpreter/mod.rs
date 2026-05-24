pub(crate) mod value;
pub(crate) mod runtime_error;
pub(crate) mod control_flow;
mod function;

use crate::interpreter::runtime_error::RuntimeError;
use crate::interpreter::value::Value;
use crate::parser::expression::Expr;
use crate::parser::statement::Stmt;
use crate::parser::supporting_types::{AssignOp, BinaryOp, ClassicalType, ForIter, IndexedIdent, IoDirection, SwitchCase, UnaryOp};
use crate::parser::{Parser, Program};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use crate::interpreter::control_flow::ControlFlow;
use crate::interpreter::function::{get_default_functions, Function};
use crate::lexer::Lexer;
use crate::lexer::type_def::TypeDefinition;

macro_rules! numeric_op {
    ($lhs:expr, $rhs:expr, $op:tt, $expr:expr) => {
        match ($lhs, $rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a $op b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a $op b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 $op b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a $op b as f64)),
            _ => Err(RuntimeError::UnsupportedOperation($expr.to_string()))
        }
    };
}

macro_rules! bitwise_op {
    ($lhs:expr, $rhs:expr, $op:tt, $expr:expr) => {
        match ($lhs, $rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a $op b)),
            (Value::Bits { value: a, width: wa }, Value::Bits { value: b, width: wb }) => {
                if wa != wb { return Err(RuntimeError::TypeMismatch($expr.to_string())); }
                Ok(Value::Bits { value: a $op b, width: wa })
            },
            (Value::Bits { value: a, width: wa }, Value::Int(b)) => {
                let mask = (1u64 << wa) - 1;
                Ok(Value::Bits { value: (a $op (b as u64)) & mask, width: wa })
            },
            (Value::Int(a), Value::Bits { value: b, width: wb }) => {
                let mask = (1u64 << wb) - 1;
                Ok(Value::Bits { value: ((a as u64) $op b) & mask, width: wb })
            },
            _ => Err(RuntimeError::UnsupportedOperation($expr.to_string()))
        }
    };
}

macro_rules! comparison_op {
    ($lhs:expr, $rhs:expr, $op:tt, $expr:expr) => {
        match ($lhs, $rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a $op b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a $op b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a $op b)),
            (Value::Bits { value: a, .. }, Value::Bits { value: b, .. }) => Ok(Value::Bool(a $op b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) $op b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a $op b as f64)),
            _ => Err(RuntimeError::UnsupportedOperation($expr.to_string()))
        }
    };
}

pub struct Interpreter {
    program: Program,
    scopes: Vec<HashMap<String, Value>>,
    constants: HashSet<String>,
    functions: HashMap<String, Function>,
    inputs: HashMap<String, Value>,
    outputs: HashMap<String, Value>,
    script_dir: PathBuf,
}

impl Interpreter {
    pub(crate) fn new(program: Program, path_buf: PathBuf) -> Interpreter {
        Interpreter {
            program,
            scopes: vec![],
            constants: HashSet::new(),
            functions: get_default_functions(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            script_dir: path_buf,
        }
    }

    pub fn set_input(&mut self, name: &str, value: Value) {
        self.inputs.insert(name.to_string(), value);
    }

    pub fn get_output(&self, name: &str) -> Option<Value> {
        self.outputs.get(name).cloned()
    }

    pub fn get_outputs(&self) -> &HashMap<String, Value> {
        &self.outputs
    }

    pub fn start(&mut self) {
        self.push_scope();

        let defaults_result = self.define_default_operations();
        match defaults_result {
            Ok(_) => {},
            Err(e) => panic!("Runtime error: {:?}", e)
        };

        for stmt in self.program.statements.clone() {
            match self.interpret_statement(&stmt) {
                Ok(ControlFlow::Return(_)) => break,
                Ok(_) => {}
                Err(e) => { panic!("Runtime error: {:?}", e); }
            }
        }

        for output in self.outputs.clone().keys() {
            let output_value = self.lookup(output);

            match output_value {
                Some(value) => self.outputs.insert(output.to_string(), value.clone()),
                None => panic!("Output '{:?}' not found!", output)
            };
        }

        self.pop_scope();
    }

    fn define(&mut self, name: String, value: Value) -> Result<ControlFlow, RuntimeError> {
        if self.constants.contains(&name) {
            return Err(RuntimeError::ConstAssignment(name.to_string()))
        };

        self.scopes.last_mut().unwrap().insert(name, value);
        Ok(ControlFlow::None)
    }

    fn assign(&mut self, name: &str, value: Value) -> Result<ControlFlow, RuntimeError> {
        if self.constants.contains(name) {
            return Err(RuntimeError::ConstAssignment(name.to_string()));
        }
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(ControlFlow::None);
            }
        }
        Err(RuntimeError::UndefinedVariable(name.to_string()))
    }

    fn define_default_operations(&mut self) -> Result<(), RuntimeError> {
        self.define("pi".to_string(), Value::Float(std::f64::consts::PI))?;
        self.define("euler".to_string(), Value::Float(std::f64::consts::E))?;
        self.define("tau".to_string(), Value::Float(std::f64::consts::TAU))?;

        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) { return Some(val); }
        }
        None
    }

    fn lookup_mut(&mut self, name: &str) -> Option<&mut Value> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(val) = scope.get_mut(name) { return Some(val); }
        }
        None
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn interpret_statement(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        match stmt {
            Stmt::QuantumDecl { .. } => todo!(),
            Stmt::ClassicalDecl { .. } => self.interpret_classical_declaration(stmt),
            Stmt::ArrayDecl { .. } => self.interpret_array_declaration(stmt),
            Stmt::ConstDecl { .. } => self.interpret_const_declaration(stmt),
            Stmt::GateCall { .. } => todo!(),
            Stmt::Measure { .. } => todo!(),
            Stmt::Reset { .. } => todo!(),
            Stmt::Barrier { .. } => todo!(),
            Stmt::If { .. } => self.interpret_if(stmt),
            Stmt::Switch { .. } => self.interpret_switch(stmt),
            Stmt::For { .. } => self.interpret_for(stmt),
            Stmt::While { .. } => self.interpret_while(stmt),
            Stmt::IoDecl { .. } => self.interpret_io_decl(stmt),
            Stmt::Include(s) => self.interpret_include(s),
            Stmt::Continue => Ok(ControlFlow::Continue),
            Stmt::Break => Ok(ControlFlow::Break),
            Stmt::ExpressionStatement(expr) => {
                self.evaluate_expression(expr)?;
                Ok(ControlFlow::None)
            }
            Stmt::Return(expr) => {
                let value = match expr {
                    Some(e) => self.evaluate_expression(e)?,
                    None => Value::Void,
                };
                Ok(ControlFlow::Return(value))
            },
            Stmt::GateDef { name, params, qubits, body } => {
                self.functions.insert(name.to_string(), Function::Gate {
                    params: params.clone(),
                    qubits: qubits.clone(),
                    body: body.clone(),
                });

                Ok(ControlFlow::None)
            },
            Stmt::Assign { .. } => self.interpret_assignment(stmt),
            Stmt::Def { name, params, return_type, body } => {
                self.functions.insert(name.clone(), Function::UserDefined {
                    params: params.clone(),
                    return_type: return_type.clone(),
                    body: body.clone(),
                });
                Ok(ControlFlow::None)
            }
            Stmt::Block(stmts) => {
                self.push_scope();
                let flow = self.interpret_statements(stmts)?;
                self.pop_scope();

                Ok(flow)
            },
        }
    }

    fn interpret_statements(&mut self, stmts: &[Stmt]) -> Result<ControlFlow, RuntimeError> {
        for stmt in stmts {
            let flow = self.interpret_statement(stmt)?;
            match flow {
                ControlFlow::None => {}
                other => return Ok(other),
            }
        }
        Ok(ControlFlow::None)
    }

    fn interpret_classical_declaration(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::ClassicalDecl { ty, name, size, init } = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        let default_value = ty.get_default_value(size.clone())?;

        let init_value = match init {
            Some(expr) => self.evaluate_expression(expr)?,
            None => default_value,
        };

        self.define(name.to_string(), init_value)?;

        Ok(ControlFlow::None)
    }

    fn interpret_array_declaration(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::ArrayDecl { ty, type_size, name, size, init} = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        let mut dimensions: Vec<i64> = vec![];

        for size_expr in size {
            match size_expr {
                Expr::Int(i) => dimensions.push(*i),
                _ => return Err(RuntimeError::InvalidSize),
            }
        }

        if dimensions.is_empty() || dimensions.len() > 7 { return Err(RuntimeError::InvalidSize); }

        let value = match init {
            Some(expr) => self.evaluate_expression(expr)?,
            None => self.default_array(ty, type_size, &dimensions)?
        };

        self.define(name.clone(), value)?;
        Ok(ControlFlow::None)
    }

    fn default_array(&self, ty: &TypeDefinition, type_size: &Option<Expr>, dimensions: &[i64]) -> Result<Value, RuntimeError> {
        if dimensions.is_empty() { return ty.get_default_value(type_size.clone()); }

        let size = dimensions[0] as usize;
        let inner: Result<Vec<Value>, _> = (0..size)
            .map(|_| self.default_array(ty, type_size, &dimensions[1..]))
            .collect();
        Ok(Value::Array(inner?))
    }

    fn interpret_const_declaration(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::ConstDecl { ty, name, size, init } = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        let init_value = self.evaluate_expression(init)?;
        self.define(name.to_string(), init_value)?;
        self.constants.insert(name.to_string());

        Ok(ControlFlow::None)
    }

    fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {

        match expr {
            Expr::Int(i) => { Ok(Value::Int(*i)) }
            Expr::Float(f) => { Ok(Value::Float(*f)) }
            Expr::Bool(b) => { Ok(Value::Bool(*b)) }
            Expr::Array(a) => { self.evaluate_array(a) }
            Expr::Imaginary(i) => { todo!() }
            Expr::Timing(_) => { todo!() }
            Expr::Bits(v, w) => { Ok(Value::Bits {value: *v, width: *w})}
            Expr::Ident(name) => {
                self.lookup(name).cloned().ok_or(RuntimeError::UndefinedVariable(name.clone()))
            }
            Expr::IndexedIdent(i) => { self.evaluate_indexed_ident(i) }
            Expr::Measure(_) => { todo!() }
            Expr::Unary { .. } => { self.evaluate_unary(expr) }
            Expr::Binary { .. } => { self.evaluate_binary(expr) }
            Expr::Index { .. } => { todo!() }
            Expr::Call { name, args } => { self.call_function(name, args) }
            Expr::Cast { ty, expr } => {
                let value = self.evaluate_expression(expr)?;
                self.apply_cast(ty, value)
            }
            Expr::Range { .. } => { todo!() }
        }
    }

    fn evaluate_array(&mut self, array: &Box<Vec<Expr>>) -> Result<Value, RuntimeError> {
        let mut vec: Vec<Value> = vec![];
        for expr in array.iter() {
            vec.push(self.evaluate_expression(expr)?);
        }

        Ok(Value::Array(vec))
    }

    fn evaluate_indexed_ident(&mut self, ident: &IndexedIdent) -> Result<Value, RuntimeError> {
        let mut value = self.lookup(&ident.name).cloned().ok_or(RuntimeError::UndefinedVariable(ident.name.clone()))?;

        for expr in &ident.indices {
            let index = match self.evaluate_expression(expr)? {
                Value::Int(i) => i,
                _ => Err(RuntimeError::TypeMismatch("index must be int".to_string()))?,
            };

            value = match value {
                Value::Array(arr) => arr.clone().into_iter().nth(index as usize)
                    .ok_or(RuntimeError::IndexOutOfBounds(index as usize, arr.len()))?,
                _ => Err(RuntimeError::TypeMismatch("cannot index non-array".to_string()))?
            }
        }

        Ok(value)
    }

    fn evaluate_unary(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        let Expr::Unary { op, expr } = expr else {
            unreachable!("Incorrect statement signature!");
        };

        let evaluated = self.evaluate_expression(&expr)?;

        match op {
            UnaryOp::Neg => {
                match evaluated {
                    Value::Int(i) => { Ok(Value::Int(-i)) }
                    Value::Float(f) => { Ok(Value::Float(-f)) }
                    _ => { Err(RuntimeError::UnsupportedOperation(expr.to_string())) }
                }
            }
            UnaryOp::BitNot => {
                match evaluated {
                    Value::Int(i) => { Ok(Value::Int(!i)) },
                    Value::Bits {value, width} => {
                        let mask = (1u64 << width) - 1;
                        Ok(Value::Bits { value: !value & mask, width })
                    }
                    _ => { Err(RuntimeError::UnsupportedOperation(expr.to_string())) }
                }
            }
            UnaryOp::LogicNot => {
                match evaluated {
                    Value::Bool(b) => { Ok(Value::Bool(!b)) },
                    _ => { Err(RuntimeError::UnsupportedOperation(expr.to_string())) }
                }
            }
        }
    }

    fn evaluate_binary(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        let Expr::Binary { op, lhs, rhs } = expr else {
            unreachable!("Incorrect statement signature!");
        };

        let lhs_evaluated = self.evaluate_expression(&lhs)?;
        let rhs_evaluated = self.evaluate_expression(&rhs)?;

        match op {
            BinaryOp::Add => numeric_op!(lhs_evaluated, rhs_evaluated, +, expr),
            BinaryOp::Sub => numeric_op!(lhs_evaluated, rhs_evaluated, -, expr),
            BinaryOp::Mul => numeric_op!(lhs_evaluated, rhs_evaluated, *, expr),
            BinaryOp::Div => numeric_op!(lhs_evaluated, rhs_evaluated, /, expr),
            BinaryOp::Mod => numeric_op!(lhs_evaluated, rhs_evaluated, %, expr),
            BinaryOp::Pow => match (lhs_evaluated, rhs_evaluated) {
                (Value::Int(a), Value::Int(b))     => Ok(Value::Float((a as f64).powf(b as f64))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
                (Value::Int(a), Value::Float(b))   => Ok(Value::Float((a as f64).powf(b))),
                (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a.powf(b as f64))),
                _ => Err(RuntimeError::UnsupportedOperation(expr.to_string()))
            },
            BinaryOp::And => bitwise_op!(lhs_evaluated, rhs_evaluated, &, expr),
            BinaryOp::Or  => bitwise_op!(lhs_evaluated, rhs_evaluated, |, expr),
            BinaryOp::Xor => bitwise_op!(lhs_evaluated, rhs_evaluated, ^, expr),
            BinaryOp::Shl => bitwise_op!(lhs_evaluated, rhs_evaluated, <<, expr),
            BinaryOp::Shr => bitwise_op!(lhs_evaluated, rhs_evaluated, >>, expr),
            BinaryOp::LogicAnd => match (lhs_evaluated, rhs_evaluated) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                _ => Err(RuntimeError::UnsupportedOperation(expr.to_string()))
            },
            BinaryOp::LogicOr => match (lhs_evaluated, rhs_evaluated) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                _ => Err(RuntimeError::UnsupportedOperation(expr.to_string()))
            },
            BinaryOp::Eq  => comparison_op!(lhs_evaluated, rhs_evaluated, ==, expr),
            BinaryOp::Neq => comparison_op!(lhs_evaluated, rhs_evaluated, !=, expr),
            BinaryOp::Lt  => comparison_op!(lhs_evaluated, rhs_evaluated, <,  expr),
            BinaryOp::Gt  => comparison_op!(lhs_evaluated, rhs_evaluated, >,  expr),
            BinaryOp::Leq => comparison_op!(lhs_evaluated, rhs_evaluated, <=, expr),
            BinaryOp::Geq => comparison_op!(lhs_evaluated, rhs_evaluated, >=, expr),
        }
    }

    fn interpret_assignment(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::Assign { target, op, value } = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        let rhs = self.evaluate_expression(value)?;

        let evaluated = match op {
            AssignOp::Eq => rhs,
            AssignOp::Compound(bin_op) => {
                let current = if target.indices.is_empty() {
                    self.lookup(&target.name)
                        .cloned()
                        .ok_or(RuntimeError::UndefinedVariable(target.name.clone()))?
                } else {
                    self.evaluate_indexed_ident(&IndexedIdent {
                        name: target.name.clone(),
                        indices: target.indices.clone(),
                    })?
                };
                self.apply_binary_op(bin_op, current, rhs)?
            }
        };

        if target.indices.is_empty() {
            self.assign(&target.name, evaluated)
        } else {
            self.assign_indexed(&target.name, &target.indices, evaluated)
        }
    }

    fn apply_binary_op(&self, op: &BinaryOp, lhs: Value, rhs: Value) -> Result<Value, RuntimeError> {
        match op {
            BinaryOp::Add => numeric_op!(lhs, rhs, +, "+="),
            BinaryOp::Sub => numeric_op!(lhs, rhs, -, "-="),
            BinaryOp::Mul => numeric_op!(lhs, rhs, *, "*="),
            BinaryOp::Div => numeric_op!(lhs, rhs, /, "/="),
            BinaryOp::Mod => numeric_op!(lhs, rhs, %, "%="),
            BinaryOp::And => bitwise_op!(lhs, rhs, &, "&="),
            BinaryOp::Or  => bitwise_op!(lhs, rhs, |, "|="),
            BinaryOp::Xor => bitwise_op!(lhs, rhs, ^, "^="),
            BinaryOp::Shl => bitwise_op!(lhs, rhs, <<, "<<="),
            BinaryOp::Shr => bitwise_op!(lhs, rhs, >>, ">>="),
            BinaryOp::Pow => match (lhs, rhs) {
                (Value::Int(a), Value::Int(b))     => Ok(Value::Float((a as f64).powf(b as f64))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
                (Value::Int(a), Value::Float(b))   => Ok(Value::Float((a as f64).powf(b))),
                (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a.powf(b as f64))),
                _ => Err(RuntimeError::UnsupportedOperation("**=".to_string()))
            },
            _ => Err(RuntimeError::UnsupportedOperation(format!("{:?}", op)))
        }
    }

    fn assign_indexed(&mut self, name: &str, indices: &[Expr], value: Value) -> Result<ControlFlow, RuntimeError> {
        let evaluated_indices: Vec<usize> = indices.iter().map(|expr| match self.evaluate_expression(expr) {
            Ok(Value::Int(i)) => Ok(i as usize),
            Ok(_) => Err(RuntimeError::TypeMismatch("index must be int".to_string())),
            Err(e) => Err(e),
        }).collect::<Result<Vec<usize>, RuntimeError>>()?;

        let arr = self.lookup_mut(name);
        match arr {
            Some(Value::Array(a)) => {
                Self::set_nested_evaluated(a.as_mut(), &evaluated_indices, value)?;
                Ok(ControlFlow::None)
            }
            Some(_) => { Err(RuntimeError::TypeMismatch("only arrays can be index assigned".to_string())) }
            None => { Err(RuntimeError::NullPointer) }
        }
    }

    fn set_nested_evaluated(arr: &mut Vec<Value>, indices: &[usize], value: Value) -> Result<ControlFlow, RuntimeError> {
        let index = indices[0];

        let len = arr.len();
        let elem = arr.get_mut(index).ok_or(RuntimeError::IndexOutOfBounds(index, len))?;

        if indices.len() == 1 { *elem = value }
        else {
            match elem {
                Value::Array(inner) => Self::set_nested_evaluated(inner, &indices[1..], value)?,
                _ => return Err(RuntimeError::TypeMismatch("cannot index non-array".to_string())),
            };
        }

        Ok(ControlFlow::None)
    }

    fn interpret_if(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::If { cond, then, else_ } = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        let evaluated = self.evaluate_expression(cond)?;

        let Value::Bool(bool) = evaluated else {
            unreachable!("Expression evaluation failed!");
        };

        if bool {
            self.push_scope();
            let flow = self.interpret_statements(then)?;
            self.pop_scope();

            return Ok(flow);
        } else {
            match else_ {
                Some(else_stmt) => {
                    self.push_scope();
                    let flow = self.interpret_statements(else_stmt)?;
                    self.pop_scope();

                    return Ok(flow);
                },
                None => {}
            }
        }

        Ok(ControlFlow::None)
    }

    fn interpret_switch(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::Switch { expr, cases } = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        let value = self.evaluate_expression(expr)?;
        let mut default_case: Option<SwitchCase> = None;

        for case in cases {
            if case.values.is_empty() {
                default_case = Some(case.clone());
                continue;
            }

            for expr in case.values.clone() {
                let expr_value = self.evaluate_expression(&expr)?;

                if value == expr_value {
                    self.push_scope();
                    let flow =self.interpret_statements(&case.body)?;
                    self.pop_scope();

                    return Ok(flow);
                }
            }
        }

        match default_case {
            Some(case) => {
                self.push_scope();
                let flow = self.interpret_statements(&case.body)?;
                self.pop_scope();

                Ok(flow)
            }
            None => {
                Ok(ControlFlow::None)
            }
        }
    }

    fn interpret_while(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::While { cond, body } = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        loop {
            let evaluated = self.evaluate_expression(cond)?;
            match evaluated {
                Value::Bool(true) => {}
                Value::Bool(false) => break,
                _ => return Err(RuntimeError::TypeMismatch("while condition must be bool".to_string())),
            }

            self.push_scope();
            let flow = self.interpret_statements(body)?;
            self.pop_scope();

            match flow {
                ControlFlow::None => {}
                ControlFlow::Break => break,
                ControlFlow::Continue => continue,
                ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
            }
        }

        Ok(ControlFlow::None)
    }

    fn interpret_for(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::For { var, ty, iter, body } = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        match iter {
            ForIter::Set(exprs) => {
                for expr in exprs.iter() {
                    self.push_scope();
                    let evaluated = self.evaluate_expression(expr)?;
                    self.define(var.clone(), evaluated)?;
                    let flow = self.interpret_statements(body)?;
                    self.pop_scope();

                    match flow {
                        ControlFlow::None => {}
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                    }
                }
            }
            ForIter::Range { start, stop, step } => {
                let start_value = match start {
                    Some(start) => self.evaluate_expression(start)?,
                    None => ty.get_default_value(None)?
                };

                let start_int = match start_value {
                    Value::Int(i) => i,
                    _ => return Err(RuntimeError::TypeMismatch("range start must be int".to_string())),
                };

                self.define(var.clone(), Value::Int(start_int))?;

                let stop_int = match stop {
                    Some(expr) => match self.evaluate_expression(expr)? {
                        Value::Int(i) => i,
                        _ => return Err(RuntimeError::TypeMismatch("range stop must be int".to_string())),
                    },
                    None => return Err(RuntimeError::TypeMismatch("range stop must be int".to_string())),
                };

                let step_value = match step {
                    Some(step) => self.evaluate_expression(step)?,
                    None => Value::Int(1)
                };

                let step_int = match step_value {
                    Value::Int(i) => i,
                    _ => return Err(RuntimeError::TypeMismatch("range step must be int".to_string())),
                };

                loop {
                    let current_int = match self.lookup(var).cloned() {
                        Some(Value::Int(i)) => i,
                        Some(_) => return Err(RuntimeError::TypeMismatch("range var must be int".to_string())),
                        None => return Err(RuntimeError::NullPointer),
                    };

                    if current_int >= stop_int { break; }
                    self.push_scope();
                    let flow = self.interpret_statements(body)?;
                    self.pop_scope();

                    self.assign(var, Value::Int(current_int + step_int))?;

                    match flow {
                        ControlFlow::None => {}
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                    }
                }
            }
            ForIter::Expr(expr) => {
                let evaluated = self.evaluate_expression(expr)?;

                let values = match evaluated {
                    Value::Array(v) => v,
                    _ => return Err(RuntimeError::TypeMismatch("for iter must be an array".to_string())),
                };

                for value in values.iter() {
                    self.push_scope();
                    self.define(var.clone(), value.clone())?;
                    let flow = self.interpret_statements(body)?;
                    self.pop_scope();

                    match flow {
                        ControlFlow::None => {}
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                    }
                }
            }
        }
        Ok(ControlFlow::None)
    }

    fn call_function(&mut self, name: &str, args: &Vec<Expr>) -> Result<Value, RuntimeError> {
        let func = self.functions.get(name)
            .cloned()
            .ok_or(RuntimeError::UndefinedFunction(name.to_string()))?;

        let mut evaluated_args = vec![];
        for x in args {
            evaluated_args.push(self.evaluate_expression(x)?);
        }

        match func {
            Function::BuiltIn(f) => f(evaluated_args),
            Function::UserDefined { params, return_type, body } => {
                if args.len() != params.len() {
                    return Err(RuntimeError::InvalidArgCount(params.len(), args.len()));
                }

                self.push_scope();

                for (param, value) in params.iter().zip(evaluated_args.into_iter()) {
                    self.define(param.name.clone(), value)?;
                }

                let flow = self.interpret_statements(&body)?;

                self.pop_scope();

                match flow {
                    ControlFlow::Return(value) => Ok(value),
                    ControlFlow::None => Ok(Value::Void),
                    ControlFlow::Break | ControlFlow::Continue => {
                        Err(RuntimeError::InvalidControlFlow)
                    }
                }
            },
            Function::Gate { .. } => {
                Err(RuntimeError::InvalidCall(
                    format!("'{}' is a gate and cannot be called as a classical function", name)
                ))
            }
        }
    }

    fn apply_cast(&self, ty: &ClassicalType, value: Value) -> Result<Value, RuntimeError> {
        match &ty {
            ClassicalType::Int(_) => match value {
                Value::Int(i)   => Ok(Value::Int(i)),
                Value::Float(f) => Ok(Value::Int(f as i64)),
                Value::Bool(b)  => Ok(Value::Int(b as i64)),
                Value::Bits { value, .. } => Ok(Value::Int(value as i64)),
                _ => Err(RuntimeError::TypeMismatch("cannot cast to int".to_string())),
            },
            ClassicalType::Float(_) => match value {
                Value::Float(f) => Ok(Value::Float(f)),
                Value::Int(i)   => Ok(Value::Float(i as f64)),
                Value::Bool(b)  => Ok(Value::Float(b as i64 as f64)),
                _ => Err(RuntimeError::TypeMismatch("cannot cast to float".to_string())),
            },
            ClassicalType::Bool(_) => match value {
                Value::Bool(b)  => Ok(Value::Bool(b)),
                Value::Int(i)   => Ok(Value::Bool(i != 0)),
                Value::Float(f) => Ok(Value::Bool(f != 0.0)),
                Value::Bits { value, .. } => Ok(Value::Bool(value != 0)),
                _ => Err(RuntimeError::TypeMismatch("cannot cast to bool".to_string())),
            },
            ClassicalType::Bit(size_expr) => {
                let width = match size_expr {
                    Some(Expr::Int(n)) => n.clone() as usize,
                    None => 1,
                    _ => return Err(RuntimeError::InvalidSize),
                };
                match value {
                    Value::Int(i)  => Ok(Value::Bits { value: i as u64, width }),
                    Value::Bits { value, .. } => Ok(Value::Bits { value, width }),
                    Value::Bool(b) => Ok(Value::Bits { value: b as u64, width }),
                    _ => Err(RuntimeError::TypeMismatch("cannot cast to bit".to_string())),
                }
            },
            _ => Err(RuntimeError::UnsupportedType),
        }
    }

    fn interpret_io_decl(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::IoDecl { direction, ty, size, name } = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        match direction {
            IoDirection::Input => {
                let value = self.inputs.get(name);
                match value {
                    Some(value) => {
                        self.define(name.to_string(), value.clone())?;
                        Ok(ControlFlow::None)
                    },
                    None => Err(RuntimeError::UndefinedVariable(name.to_string())),
                }
            }
            IoDirection::Output => {
                self.define(name.to_string(), ty.get_default_value(size.clone())?)?;
                self.outputs.insert(name.to_string(), ty.get_default_value(size.clone())?);

                Ok(ControlFlow::None)
            }
        }
    }

    fn interpret_include(&mut self, path: &String) -> Result<ControlFlow, RuntimeError> {
        let full_path = self.script_dir.join(path);
        let source = std::fs::read_to_string(&full_path)
            .map_err(|_| RuntimeError::FileNotFound(path.clone()))?;

        let mut lexer = Lexer::new(source);
        lexer.start();

        let mut parser = Parser::new(lexer.tokens);
        let program = parser.start()
            .map_err(|e| RuntimeError::ParseError(e))?;

        self.interpret_statements(&program.statements)
    }
}