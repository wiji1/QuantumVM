pub mod quantum;

pub(crate) mod value;
pub(crate) mod runtime_error;
pub(crate) mod control_flow;
mod function;

use crate::interpreter::control_flow::ControlFlow;
use crate::interpreter::function::{get_default_functions, Function};
use crate::interpreter::quantum::gates::cartesian_product;
use crate::interpreter::quantum::resolved_gate::ResolvedGate;
use crate::interpreter::quantum::statevector::{StateVector, C64};
use crate::interpreter::runtime_error::RuntimeError;
use crate::interpreter::value::Value;
use crate::lexer::Lexer;
use crate::parser::expression::Expr;
use crate::parser::statement::Stmt;
use crate::parser::supporting_types::{AssignOp, BinaryOp, ClassicalType, ForIter, GateModifier, GateOperand, IndexedIdent, IoDirection, SwitchCase, UnaryOp};
use crate::parser::{Parser, Program};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use crate::coercion::{cast_value, coerce_to_bool, coerce_value, infer_type_from_value};
use crate::type_checker::type_repr::Type;

macro_rules! numeric_op {
    ($lhs:expr, $rhs:expr, $op:tt, $expr:expr) => {
        match ($lhs, $rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a $op b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a $op b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 $op b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a $op b as f64)),
            (Value::Complex(re1, im1), Value::Complex(re2, im2)) => {
                Ok(Value::Complex(re1 $op re2, im1 $op im2))
            },
            (Value::Float(a), Value::Complex(re, im)) => Ok(Value::Complex(a $op re, im)),
            (Value::Complex(re, im), Value::Float(b)) => Ok(Value::Complex(re $op b, im)),
            (Value::Int(a), Value::Complex(re, im)) => Ok(Value::Complex(a as f64 $op re, im)),
            (Value::Complex(re, im), Value::Int(b)) => Ok(Value::Complex(re $op b as f64, im)),
            _ => Err(RuntimeError::UnsupportedOperation($expr.to_string())),
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
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a $op (b as f64))),
            (Value::Bits { value: a, .. }, Value::Int(b)) => {
                let lhs_val: i64 = a as i64;
                let rhs_val: i64 = b;
                Ok(Value::Bool(lhs_val $op rhs_val))
            },
            (Value::Int(a), Value::Bits { value: b, .. }) => {
                let lhs_val: i64 = a;
                let rhs_val: i64 = b as i64;
                Ok(Value::Bool(lhs_val $op rhs_val))
            },
            (Value::Bits { value: a, .. }, Value::Float(b)) => {
                let lhs_val: f64 = a as f64;
                Ok(Value::Bool(lhs_val $op b))
            },
            (Value::Float(a), Value::Bits { value: b, .. }) => {
                let rhs_val: f64 = b as f64;
                Ok(Value::Bool(a $op rhs_val))
            },
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
    state_vector: Option<StateVector>,
    qubit_map: HashMap<String, Vec<usize>>,
    num_qubits: usize,
    gate_cache: HashMap<(String, Vec<u64>), ResolvedGate>,
    call_depth: usize
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
            state_vector: None,
            qubit_map: HashMap::new(),
            num_qubits: 0,
            gate_cache: HashMap::new(),
            call_depth: 0
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

    pub fn get_state_vector(&self) -> Option<StateVector> {
        self.state_vector.clone()
    }

    fn normalize_index(idx: i64, len: usize) -> Result<usize, RuntimeError> {
        if idx < 0 {
            let pos = len as i64 + idx;
            if pos < 0 { return Err(RuntimeError::IndexOutOfBounds(idx as usize, len)); }
            Ok(pos as usize)
        } else {
            let idx_usize = idx as usize;
            if idx_usize >= len { return Err(RuntimeError::IndexOutOfBounds(idx_usize, len)); }
            Ok(idx_usize)
        }
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

    fn define(&mut self, name: String, value: Value, const_decl: bool) -> Result<ControlFlow, RuntimeError> {
        if self.scopes.last().unwrap().contains_key(&name) {
            return Err(RuntimeError::DuplicateVariable(name));
        }

        if const_decl { self.constants.insert(name.to_string()); }
        self.scopes.last_mut().unwrap().insert(name, value);
        Ok(ControlFlow::None)
    }

    fn assign(&mut self, name: &str, value: Value) -> Result<ControlFlow, RuntimeError> {
        if self.constants.contains(name) {
            return Err(RuntimeError::ConstReassignment(name.to_string()));
        }

        for scope in self.scopes.iter_mut().rev() {
            if let Some(current_value) = scope.get(name) {
                let target_type = infer_type_from_value(current_value);

                let coerced_value = coerce_value(value, &target_type)
                    .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;

                scope.insert(name.to_string(), coerced_value);
                return Ok(ControlFlow::None);
            }
        }
        Err(RuntimeError::UndefinedVariable(name.to_string()))
    }

    fn define_default_operations(&mut self) -> Result<(), RuntimeError> {
        self.define("pi".to_string(), Value::Float(std::f64::consts::PI), true)?;
        self.define("π".to_string(), Value::Float(std::f64::consts::PI), true)?;

        self.define("euler".to_string(), Value::Float(std::f64::consts::E), true)?;
        self.define("ℯ".to_string(), Value::Float(std::f64::consts::E), true)?;

        self.define("tau".to_string(), Value::Float(std::f64::consts::TAU), true)?;
        self.define("τ".to_string(), Value::Float(std::f64::consts::TAU), true)?;

        self.define("im".to_string(), Value::Complex(0.0, 1.0), true)?;
        self.define("ⅈ".to_string(), Value::Complex(0.0, 1.0), true)?;

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
            Stmt::QuantumDecl { .. } => self.interpret_quantum_decl(stmt),
            Stmt::ClassicalDecl { .. } => self.interpret_classical_declaration(stmt),
            Stmt::ArrayDecl { .. } => self.interpret_array_declaration(stmt),
            Stmt::ConstDecl { .. } => self.interpret_const_declaration(stmt),
            Stmt::GateCall { .. } => self.interpret_gate_call(stmt),
            Stmt::If { .. } => self.interpret_if(stmt),
            Stmt::Switch { .. } => self.interpret_switch(stmt),
            Stmt::For { .. } => self.interpret_for(stmt),
            Stmt::While { .. } => self.interpret_while(stmt),
            Stmt::IoDecl { .. } => self.interpret_io_decl(stmt),
            Stmt::Include(s) => self.interpret_include(s),
            Stmt::IncludeFromSrc(_, s) => self.interpret_include_from_src(s),
            Stmt::Let { .. } => self.interpret_let(stmt),
            Stmt::Barrier { .. } => Ok(ControlFlow::None),
            Stmt::Pragma => Ok(ControlFlow::None),
            Stmt::Continue => Ok(ControlFlow::Continue),
            Stmt::Break => Ok(ControlFlow::Break),
            Stmt::Reset { qubit } => {
                let qubit_indices = self.resolve_qubits(qubit)?;
                let sv = self.state_vector.as_mut().ok_or(RuntimeError::NoStateVector)?;
                for qubit_index in qubit_indices { sv.reset_qubit(qubit_index); }
                Ok(ControlFlow::None)
            }
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
                if self.functions.contains_key(name) {
                    return Err(RuntimeError::DuplicateGate(name.to_string()));
                }
                self.functions.insert(name.to_string(), Function::Gate {
                    params: params.clone(),
                    qubits: qubits.clone(),
                    body: body.clone(),
                });

                Ok(ControlFlow::None)
            },
            Stmt::Assign { value, .. } => {
                match value {
                    Expr::Measure(_) => self.interpret_measure(stmt),
                    _ => self.interpret_assignment(stmt)
                }
            },
            Stmt::Def { name, params, return_type, body } => {
                if self.functions.contains_key(name) {
                    return Err(RuntimeError::DuplicateFunction(name.to_string()));
                }
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
            Stmt::Extern { name } => {
                self.functions.insert(name.clone(), Function::Extern);
                Ok(ControlFlow::None)
            }
            Stmt::NoOp => Ok(ControlFlow::None)
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
        let Stmt::ClassicalDecl { ty, name, init } = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        let default_value = ty.get_default_value(ty.get_size())?;

        let init_value = match init {
            Some(expr) => self.evaluate_expression(expr)?,
            None => default_value,
        };

        let target_type = Type::from_classical_type(ty);
        let coerced_value = coerce_value(init_value, &target_type)
            .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;

        self.define(name.to_string(), coerced_value, false)?;

        Ok(ControlFlow::None)
    }

    fn interpret_array_declaration(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::ArrayDecl { ty, name, size, init} = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        let mut dimensions: Vec<i64> = vec![];

        for size_expr in size {
            let evaluated = self.evaluate_expression(size_expr)?;
            match evaluated {
                Value::Int(i) => dimensions.push(i),
                _ => return Err(RuntimeError::InvalidSize),
            }
        }

        if dimensions.is_empty() || dimensions.len() > 7 { return Err(RuntimeError::InvalidSize); }

        let value = match init {
            Some(expr) => self.evaluate_expression(expr)?,
            None => self.default_array(ty, &dimensions)?
        };

        self.define(name.clone(), value, false)?;
        Ok(ControlFlow::None)
    }

    fn default_array(&self, ty: &ClassicalType, dimensions: &[i64]) -> Result<Value, RuntimeError> {
        if dimensions.is_empty() { return ty.get_default_value(ty.get_size()); }

        let size = dimensions[0] as usize;
        let inner: Result<Vec<Value>, _> = (0..size)
            .map(|_| self.default_array(ty, &dimensions[1..]))
            .collect();
        Ok(Value::Array(inner?))
    }

    fn interpret_const_declaration(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::ConstDecl { ty, name, init } = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        let init_value = self.evaluate_expression(init)?;
        self.define(name.to_string(), init_value, true)?;

        Ok(ControlFlow::None)
    }

    fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {

        match expr {
            Expr::Int(i) => { Ok(Value::Int(*i)) }
            Expr::Float(f) => { Ok(Value::Float(*f)) }
            Expr::Bool(b) => { Ok(Value::Bool(*b)) }
            Expr::Array(a) => { self.evaluate_array(a) }
            Expr::Imaginary(f) => { Ok(Value::Complex(0.0, *f)) }
            Expr::Timing(s) => { Ok(Value::Timing(s.clone())) }
            Expr::Bits(v, w) => { Ok(Value::Bits {value: *v, width: *w})}
            Expr::Ident(name) => {
                if let Some(val) = self.lookup(name) { Ok(val.clone()) }
                else if let Some(indices) = self.qubit_map.get(name) {
                    Ok(Value::Qubit(indices.clone()))
                } else { Err(RuntimeError::UndefinedVariable(name.clone())) }
            }
            Expr::IndexedIdent(i) => { self.evaluate_indexed_ident(i) }
            Expr::Measure(op) => { self.evaluate_measure(op) }
            Expr::Unary { .. } => { self.evaluate_unary(expr) }
            Expr::Binary { .. } => { self.evaluate_binary(expr) }
            Expr::Call { name, args } => { self.call_function(name, args) }
            Expr::Cast { ty, expr } => {
                let value = self.evaluate_expression(expr)?;
                let target_type = Type::from_classical_type(ty);
                cast_value(value, &target_type)
                    .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))
            }
            Expr::Range { .. } => { self.evaluate_range(expr) }
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
        if let Some(indices) = self.qubit_map.get(&ident.name).cloned() {
            return self.evaluate_qubit_ident(indices, &ident.indices);
        }

        let mut value = self.lookup(&ident.name).cloned()
            .ok_or(RuntimeError::UndefinedVariable(ident.name.clone()))?;

        for expr in &ident.indices {
            let index_val = self.evaluate_expression(expr)?;
            value = match (value, index_val) {
                (Value::Array(arr), Value::Int(i)) => {
                    let len = arr.len();
                    let normalized_idx = Self::normalize_index(i, len)?;
                    arr.into_iter().nth(normalized_idx)
                        .ok_or(RuntimeError::IndexOutOfBounds(normalized_idx, len))?
                }
                (Value::Array(arr), Value::Range { start, stop, step }) => {
                    let len = arr.len();
                    let start = if let Some(s) = start { Self::normalize_index(s, len)? }
                    else { 0 };
                    let stop = if let Some(s) = stop { Self::normalize_index(s, len)? }
                    else { len - 1 };
                    let step = step.unwrap_or(1) as usize;
                    Value::Array((start..=stop).step_by(step).map(|i| arr[i].clone()).collect())
                }
                (Value::Bits { value, width }, Value::Int(i)) => {
                    let normalized_idx = if i < 0 {
                        let pos = width as i64 + i;
                        if pos < 0 { return Err(RuntimeError::IndexOutOfBounds(i as usize, width)); }
                        pos as usize
                    } else {
                        let idx = i as usize;
                        if idx >= width { return Err(RuntimeError::IndexOutOfBounds(idx, width)); }
                        idx
                    };
                    Value::Bits { value: (value >> normalized_idx) & 1, width: 1 }
                }
                _ => return Err(RuntimeError::TypeMismatch("cannot index with this type".to_string())),
            };
        }
        Ok(value)
    }

    fn evaluate_qubit_ident(&mut self, indices: Vec<usize>, exprs: &[Expr]) -> Result<Value, RuntimeError> {
        if exprs.is_empty() { return Ok(Value::Qubit(indices)); }

        let index_val = self.evaluate_expression(&exprs[0])?;

        let selected = match index_val {
            Value::Int(i) => {
                let qubit = indices.get(i as usize)
                    .copied()
                    .ok_or(RuntimeError::IndexOutOfBounds(i as usize, indices.len()))?;
                vec![qubit]
            }
            Value::Array(arr) => {
                arr.into_iter().map(|val| match val {
                    Value::Int(i) => indices.get(i as usize)
                        .copied()
                        .ok_or(RuntimeError::IndexOutOfBounds(i as usize, indices.len())),
                    _ => Err(RuntimeError::TypeMismatch("qubit set index must be int".to_string())),
                }).collect::<Result<Vec<_>, _>>()?
            }
            Value::Range { start, stop, step } => {
                let start = start.unwrap_or(0) as usize;
                let stop = stop.unwrap_or(indices.len() as i64) as usize;
                let step = step.unwrap_or(1) as usize;
                (start..=stop).step_by(step)
                    .map(|i| indices.get(i).copied().ok_or(RuntimeError::IndexOutOfBounds(i, indices.len())))
                    .collect::<Result<Vec<_>, _>>()?
            }
            other => {
                let coerced = coerce_value(other, &Type::Int(None))
                    .map_err(|_| RuntimeError::TypeMismatch("qubit index must be int, range, or set".to_string()))?;
                match coerced {
                    Value::Int(i) => {
                        let qubit = indices.get(i as usize)
                            .copied()
                            .ok_or(RuntimeError::IndexOutOfBounds(i as usize, indices.len()))?;
                        vec![qubit]
                    }
                    _ => return Err(RuntimeError::TypeMismatch("qubit index coercion did not produce int".to_string())),
                }
            }
        };

        if exprs.len() > 1 { return self.evaluate_qubit_ident(selected, &exprs[1..]); }

        Ok(Value::Qubit(selected))
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

        if matches!(op, BinaryOp::Div | BinaryOp::Mod) {
            let is_zero = match &rhs_evaluated {
                Value::Int(i) => *i == 0,
                Value::Float(f) => *f == 0.0,
                _ => false,
            };
            if is_zero { return Err(RuntimeError::DivideByZero); }
        }

        match op {
            BinaryOp::Add => numeric_op!(lhs_evaluated, rhs_evaluated, +, expr),
            BinaryOp::Sub => numeric_op!(lhs_evaluated, rhs_evaluated, -, expr),
            BinaryOp::Mul => numeric_op!(lhs_evaluated, rhs_evaluated, *, expr),
            BinaryOp::Div => match (lhs_evaluated, rhs_evaluated) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Float(a as f64 / b as f64)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 / b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / b as f64)),
                _ => Err(RuntimeError::UnsupportedOperation(expr.to_string()))
            },
            BinaryOp::Mod => numeric_op!(lhs_evaluated, rhs_evaluated, %, expr),
            BinaryOp::Pow => match (lhs_evaluated, rhs_evaluated) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Float((a as f64).powf(b as f64))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((a as f64).powf(b))),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.powf(b as f64))),
                _ => Err(RuntimeError::UnsupportedOperation(expr.to_string()))
            },
            BinaryOp::And => bitwise_op!(lhs_evaluated, rhs_evaluated, &, expr),
            BinaryOp::Or  => bitwise_op!(lhs_evaluated, rhs_evaluated, |, expr),
            BinaryOp::Xor => bitwise_op!(lhs_evaluated, rhs_evaluated, ^, expr),
            BinaryOp::Shl => bitwise_op!(lhs_evaluated, rhs_evaluated, <<, expr),
            BinaryOp::Shr => bitwise_op!(lhs_evaluated, rhs_evaluated, >>, expr),
            BinaryOp::LogicAnd => {
                let lhs_bool = coerce_to_bool(lhs_evaluated)
                    .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;
                let rhs_bool = coerce_to_bool(rhs_evaluated)
                    .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;

                match (lhs_bool, rhs_bool) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                    _ => unreachable!("coerce_to_bool should always return Bool"),
                }
            },
            BinaryOp::LogicOr => {
                let lhs_bool = coerce_to_bool(lhs_evaluated)
                    .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;
                let rhs_bool = coerce_to_bool(rhs_evaluated)
                    .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;

                match (lhs_bool, rhs_bool) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                    _ => unreachable!("coerce_to_bool should always return Bool"),
                }
            },
            BinaryOp::Eq  => comparison_op!(lhs_evaluated, rhs_evaluated, ==, expr),
            BinaryOp::Neq => comparison_op!(lhs_evaluated, rhs_evaluated, !=, expr),
            BinaryOp::Lt  => comparison_op!(lhs_evaluated, rhs_evaluated, <,  expr),
            BinaryOp::Gt  => comparison_op!(lhs_evaluated, rhs_evaluated, >,  expr),
            BinaryOp::Leq => comparison_op!(lhs_evaluated, rhs_evaluated, <=, expr),
            BinaryOp::Geq => comparison_op!(lhs_evaluated, rhs_evaluated, >=, expr),
            BinaryOp::Concat => self.concatenate_values(&lhs_evaluated, &rhs_evaluated),
        }
    }

    fn evaluate_measure(&mut self, operand: &GateOperand) -> Result<Value, RuntimeError> {
        let qubit_indices = self.resolve_qubits(operand)?;
        self.measure_qubits(qubit_indices)
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

    fn concatenate_values(&self, lhs: &Value, rhs: &Value) -> Result<Value, RuntimeError> {
        match (lhs, rhs) {
            (Value::Qubit(indices_a), Value::Qubit(indices_b)) => {
                let set_a: HashSet<_> = indices_a.iter().collect();
                let set_b: HashSet<_> = indices_b.iter().collect();

                if set_a.intersection(&set_b).count() > 0 {
                    return Err(RuntimeError::SelfConcatenation(
                        "Cannot concatenate a qubit register with itself".to_string()
                    ));
                }

                let mut result = indices_a.clone();
                result.extend(indices_b);
                Ok(Value::Qubit(result))
            }

            (Value::Bits { value: v1, width: w1 }, Value::Bits { value: v2, width: w2 }) => {
                let new_width = w1 + w2;
                if new_width > 64 {
                    return Err(RuntimeError::UnsupportedOperation(
                        format!("Bit concatenation exceeds 64-bit limit: {} + {}", w1, w2)
                    ));
                }

                let new_value = (v2 << w1) | v1;
                Ok(Value::Bits { value: new_value, width: new_width })
            }

            (Value::Array(arr1), Value::Array(arr2)) => {
                let mut result = arr1.clone();
                result.extend(arr2.clone());
                Ok(Value::Array(result))
            }

            _ => Err(RuntimeError::UnsupportedOperation(
                format!("Cannot concatenate {:?} and {:?}", lhs, rhs)
            ))
        }
    }

    fn apply_binary_op(&self, op: &BinaryOp, lhs: Value, rhs: Value) -> Result<Value, RuntimeError> {
        if matches!(op, BinaryOp::Div | BinaryOp::Mod) {
            let is_zero = match &rhs {
                Value::Int(i) => *i == 0,
                Value::Float(f) => *f == 0.0,
                _ => false,
            };
            if is_zero { return Err(RuntimeError::DivideByZero); }
        }

        match op {
            BinaryOp::Add => numeric_op!(lhs, rhs, +, "+="),
            BinaryOp::Sub => numeric_op!(lhs, rhs, -, "-="),
            BinaryOp::Mul => numeric_op!(lhs, rhs, *, "*="),
            BinaryOp::Div => match (lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Float(a as f64 / b as f64)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 / b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / b as f64)),
                _ => Err(RuntimeError::UnsupportedOperation("/".to_string()))
            },
            BinaryOp::Mod => numeric_op!(lhs, rhs, %, "%="),
            BinaryOp::And => {
                if matches!(lhs, Value::Bool(_)) || matches!(rhs, Value::Bool(_)) {
                    let lhs_bool = coerce_to_bool(lhs)
                        .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;
                    let rhs_bool = coerce_to_bool(rhs)
                        .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;
                    match (lhs_bool, rhs_bool) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a & b)),
                        _ => unreachable!(),
                    }
                } else { bitwise_op!(lhs, rhs, &, "&=") }
            },
            BinaryOp::Or => {
                if matches!(lhs, Value::Bool(_)) || matches!(rhs, Value::Bool(_)) {
                    let lhs_bool = coerce_to_bool(lhs)
                        .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;
                    let rhs_bool = coerce_to_bool(rhs)
                        .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;
                    match (lhs_bool, rhs_bool) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a | b)),
                        _ => unreachable!(),
                    }
                } else { bitwise_op!(lhs, rhs, |, "|=") }
            },
            BinaryOp::Xor => {
                if matches!(lhs, Value::Bool(_)) || matches!(rhs, Value::Bool(_)) {
                    let lhs_bool = coerce_to_bool(lhs)
                        .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;
                    let rhs_bool = coerce_to_bool(rhs)
                        .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;
                    match (lhs_bool, rhs_bool) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a ^ b)),
                        _ => unreachable!(),
                    }
                } else { bitwise_op!(lhs, rhs, ^, "^=") }
            },
            BinaryOp::Shl => bitwise_op!(lhs, rhs, <<, "<<="),
            BinaryOp::Shr => bitwise_op!(lhs, rhs, >>, ">>="),
            BinaryOp::Pow => match (lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Float((a as f64).powf(b as f64))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((a as f64).powf(b))),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.powf(b as f64))),
                _ => Err(RuntimeError::UnsupportedOperation("**=".to_string()))
            },
            BinaryOp::Concat => self.concatenate_values(&lhs, &rhs),
            _ => Err(RuntimeError::UnsupportedOperation(format!("{:?}", op)))
        }
    }

    fn assign_indexed(&mut self, name: &str, indices: &[Expr], new_value: Value) -> Result<ControlFlow, RuntimeError> {
        let first_index = self.evaluate_expression(&indices[0])?;

        if let Value::Range { start, stop, step } = first_index {
            let new_values = match new_value {
                Value::Array(v) => v,
                _ => return Err(RuntimeError::TypeMismatch("slice assignment requires array".to_string())),
            };

            let (normalized_start, normalized_stop, step_val) = {
                let target = self.lookup(name).ok_or(RuntimeError::NullPointer)?;
                if let Value::Array(arr) = target {
                    let len = arr.len();
                    let start_idx = if let Some(s) = start { Self::normalize_index(s, len)? }
                    else { 0 };
                    let stop_idx = if let Some(s) = stop { Self::normalize_index(s, len)? }
                    else { len - 1 };
                    (start_idx, stop_idx, step.unwrap_or(1) as usize)
                } else {
                    return Err(RuntimeError::TypeMismatch("cannot slice non-array".to_string()));
                }
            };

            let target = self.lookup_mut(name).ok_or(RuntimeError::NullPointer)?;
            if let Value::Array(arr) = target {
                for (i, val) in (normalized_start..=normalized_stop).step_by(step_val).zip(new_values) {
                    arr[i] = val;
                }
                return Ok(ControlFlow::None);
            }
            return Err(RuntimeError::TypeMismatch("cannot slice non-array".to_string()));
        }

        let evaluated_indices: Vec<i64> = indices.iter().map(|expr| match self.evaluate_expression(expr) {
            Ok(Value::Int(i)) => Ok(i),
            Ok(_) => Err(RuntimeError::TypeMismatch("index must be int".to_string())),
            Err(e) => Err(e),
        }).collect::<Result<Vec<i64>, RuntimeError>>()?;

        let is_bits = matches!(self.lookup(name), Some(Value::Bits { .. }));

        if is_bits {
            let target = self.lookup_mut(name).ok_or(RuntimeError::NullPointer)?;
            if let Value::Bits { value, width } = target {
                let bit_pos = if evaluated_indices[0] < 0 {
                    let pos = *width as i64 + evaluated_indices[0];
                    if pos < 0 { return Err(RuntimeError::IndexOutOfBounds(evaluated_indices[0] as usize, *width)); }
                    pos as usize
                } else {
                    let idx = evaluated_indices[0] as usize;
                    if idx >= *width { return Err(RuntimeError::IndexOutOfBounds(idx, *width)); }
                    idx
                };
                match new_value {
                    Value::Bits { value: new_bit, .. } => {
                        if new_bit != 0 { *value |= 1 << bit_pos; }
                        else { *value &= !(1 << bit_pos); }
                    }
                    Value::Bool(b) => {
                        if b { *value |= 1 << bit_pos; }
                        else { *value &= !(1 << bit_pos); }
                    }
                    Value::Int(i) => {
                        if i != 0 { *value |= 1 << bit_pos; }
                        else { *value &= !(1 << bit_pos); }
                    }
                    _ => return Err(RuntimeError::TypeMismatch("cannot assign to bit register".to_string())),
                }
                return Ok(ControlFlow::None);
            }
        }

        let target = self.lookup_mut(name).ok_or(RuntimeError::NullPointer)?;
        match target {
            Value::Array(a) => {
                Self::set_nested_with_i64_indices(a, &evaluated_indices, new_value)?;
                Ok(ControlFlow::None)
            }
            _ => Err(RuntimeError::TypeMismatch("only arrays and bit registers can be index assigned".to_string()))
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

    fn set_nested_with_i64_indices(arr: &mut Vec<Value>, indices: &[i64], value: Value) -> Result<ControlFlow, RuntimeError> {
        let len = arr.len();
        let normalized_idx = Self::normalize_index(indices[0], len)?;
        let elem = arr.get_mut(normalized_idx).ok_or(RuntimeError::IndexOutOfBounds(normalized_idx, len))?;

        if indices.len() == 1 { *elem = value }
        else {
            match elem {
                Value::Array(inner) => Self::set_nested_with_i64_indices(inner, &indices[1..], value)?,
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

        let coerced = coerce_to_bool(evaluated)
            .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;

        let Value::Bool(bool) = coerced else {
            unreachable!("coerce_to_bool should always return a Bool value!");
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

            let coerced = coerce_to_bool(evaluated)
                .map_err(|e| RuntimeError::TypeMismatch(e.to_string()))?;

            match coerced {
                Value::Bool(true) => {}
                Value::Bool(false) => break,
                _ => unreachable!("coerce_to_bool should always return a Bool value!"),
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
                    self.define(var.clone(), evaluated, false)?;
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

                self.push_scope();
                self.define(var.clone(), Value::Int(start_int), false)?;

                loop {
                    let current_int = match self.lookup(var).cloned() {
                        Some(Value::Int(i)) => i,
                        Some(_) => {
                            self.pop_scope();
                            return Err(RuntimeError::TypeMismatch("range var must be int".to_string()));
                        }
                        None => {
                            self.pop_scope();
                            return Err(RuntimeError::NullPointer);
                        }
                    };

                    if current_int > stop_int { break; }
                    self.push_scope();
                    let flow = self.interpret_statements(body)?;
                    self.pop_scope();

                    self.assign(var, Value::Int(current_int + step_int))?;

                    match flow {
                        ControlFlow::None => {}
                        ControlFlow::Break => break,
                        ControlFlow::Continue => continue,
                        ControlFlow::Return(v) => {
                            self.pop_scope();
                            return Ok(ControlFlow::Return(v));
                        }
                    }
                }

                self.pop_scope();
            }
            ForIter::Expr(expr) => {
                let evaluated = self.evaluate_expression(expr)?;

                let values = match evaluated {
                    Value::Array(v) => v,
                    _ => return Err(RuntimeError::TypeMismatch("for iter must be an array".to_string())),
                };

                for value in values.iter() {
                    self.push_scope();
                    self.define(var.clone(), value.clone(), false)?;
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
            Function::BuiltIn(f) => f.call(evaluated_args),
            Function::UserDefined { params, return_type, body } => {
                if args.len() != params.len() {
                    return Err(RuntimeError::InvalidArgCount(params.len(), args.len()));
                }

                self.push_scope();
                self.call_depth += 1;

                if self.call_depth > 512 {
                    self.call_depth -= 1;
                    return Err(RuntimeError::RecursionLimit);
                }

                let mut qubit_params: Vec<(String, Option<Vec<usize>>)> = vec![];

                for (param, value) in params.iter().zip(evaluated_args.into_iter()) {
                    match value {
                        Value::Qubit(indices) => {
                            let old = self.qubit_map.insert(param.name.clone(), indices);
                            qubit_params.push((param.name.clone(), old));
                        }
                        _ => { self.define(param.name.clone(), value, false)?; }
                    }
                }

                let flow = self.interpret_statements(&body)?;

                for (name, old_indices) in qubit_params {
                    match old_indices {
                        Some(indices) => { self.qubit_map.insert(name, indices); }
                        None => { self.qubit_map.remove(&name); }
                    }
                }

                self.pop_scope();
                self.call_depth -= 1;

                match flow {
                    ControlFlow::Return(value) => Ok(value),
                    ControlFlow::None => Ok(Value::Void),
                    ControlFlow::Break | ControlFlow::Continue => {
                        Err(RuntimeError::InvalidControlFlow)
                    }
                }
            },
            Function::Extern => {
                println!("Warning: extern function '{}' called, returning void", name);
                Ok(Value::Void)
            }
            Function::Gate { .. } | Function::BuiltInGate { .. } => {
                Err(RuntimeError::InvalidCall(
                    format!("'{}' is a gate and cannot be called as a classical function", name)
                ))
            }
        }
    }


    fn interpret_io_decl(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::IoDecl { direction, ty, name } = stmt else {
            unreachable!("Incorrect statement signature!");
        };

        match direction {
            IoDirection::Input => {
                let value = self.inputs.get(name);
                match value {
                    Some(value) => {
                        self.define(name.to_string(), value.clone(), false)?;
                        Ok(ControlFlow::None)
                    },
                    None => Err(RuntimeError::UndefinedVariable(name.to_string())),
                }
            }
            IoDirection::Output => {
                self.define(name.to_string(), ty.get_default_value(ty.get_size())?, false)?;
                self.outputs.insert(name.to_string(), ty.get_default_value(ty.get_size())?);

                Ok(ControlFlow::None)
            }
        }
    }

    fn interpret_include(&mut self, path: &String) -> Result<ControlFlow, RuntimeError> {
        if *path == "stdgates.inc".to_string() { return Ok(ControlFlow::None); }

        let full_path = self.script_dir.join(path);
        let source = std::fs::read_to_string(&full_path)
            .map_err(|_| RuntimeError::FileNotFound(path.clone()))?;

        self.interpret_include_from_src(&source)
    }

    fn interpret_include_from_src(&mut self, source: &String) -> Result<ControlFlow, RuntimeError> {
        let mut lexer = Lexer::new(source.clone());
        lexer.start();

        let mut parser = Parser::new(lexer.tokens);
        let program = parser.start(false)
            .map_err(|e| RuntimeError::ParseError(e))?;

        self.interpret_statements(&program.statements)
    }


    fn allocate_qubits(&mut self, name: &str, count: usize) {
        let start_index = self.num_qubits;
        let indices: Vec<usize> = (start_index..start_index + count).collect();
        self.qubit_map.insert(name.to_string(), indices);
        self.num_qubits += count;

        let new_size = 2usize.pow(self.num_qubits as u32);
        match &mut self.state_vector {
            None => {
                self.state_vector = Some(StateVector::new(self.num_qubits));
            }
            Some(sv) => {
                let old_amplitudes = sv.amplitudes.clone();
                sv.amplitudes = vec![C64::new(0.0, 0.0); new_size];
                for (i, amp) in old_amplitudes.iter().enumerate() {
                    sv.amplitudes[i] = *amp;
                }
                sv.num_qubits = self.num_qubits;
            }
        }
    }

    fn interpret_quantum_decl(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::QuantumDecl { name, size } = stmt else {
            unreachable!();
        };

        let count = match size {
            Some(expr) => {
                let evaluated_size = self.evaluate_expression(expr)?;

                match evaluated_size {
                    Value::Int(size) => size as usize,
                    _ => return Err(RuntimeError::InvalidSize)
                }
            },
            None => 1,
            _ => return Err(RuntimeError::InvalidSize),
        };

        self.allocate_qubits(name, count);
        Ok(ControlFlow::None)
    }

    fn resolve_qubits(&mut self, operand: &GateOperand) -> Result<Vec<usize>, RuntimeError> {
        match operand {
            GateOperand::Ident(ident) => {
                let indices: Vec<usize> = self.qubit_map
                    .get(&ident.name)
                    .ok_or_else(|| RuntimeError::UndefinedVariable(ident.name.clone()))?
                    .clone();

                if ident.indices.is_empty() {
                    Ok(indices.clone())
                } else {
                    let index = match &ident.indices[0] {
                        Expr::Int(i) => *i as usize,
                        _ => match self.evaluate_expression(&ident.indices[0])? {
                            Value::Int(i) => i as usize,
                            _ => return Err(RuntimeError::TypeMismatch("qubit index must be int".to_string(), )),
                        },
                    };
                    let qubit = indices.get(index)
                        .copied()
                        .ok_or(RuntimeError::IndexOutOfBounds(index, indices.len()))?;

                    Ok(vec![qubit])
                }
            }
            GateOperand::HardwareQubit(i) => {
                self.ensure_hardware_qubit(*i as usize);
                Ok(vec![*i as usize])
            },
        }
    }

    fn interpret_gate_call(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::GateCall { name, modifiers, params, qubits } = stmt else {
            unreachable!();
        };

        let param_values: Vec<f64> = params.iter()
            .map(|p| {
                match self.evaluate_expression(p) {
                    Ok(Value::Float(f)) => Ok(f),
                    Ok(Value::Int(i)) => Ok(i as f64),
                    Ok(_) => Err(RuntimeError::TypeMismatch("gate param must be numeric".to_string())),
                    Err(e) => Err(e),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let qubit_groups: Vec<Vec<usize>> = qubits.iter()
            .map(|q| self.resolve_qubits(q))
            .collect::<Result<Vec<_>, _>>()?;

        let func = self.functions.get(name)
            .ok_or(RuntimeError::UndefinedFunction(name.clone()))?
            .clone();

        let expected_param_count = func.get_params().len();
        if param_values.len() != expected_param_count {
            return Err(RuntimeError::InvalidArgCount(expected_param_count, param_values.len()));
        }

        if name == "gphase" { return Ok(ControlFlow::None); }

        if let Function::UserDefined { .. } = &func {
            let mut args: Vec<Expr> = params.clone();
            for qubit in qubits {
                match qubit {
                    GateOperand::Ident(ident) => args.push(Expr::IndexedIdent(ident.clone())),
                    GateOperand::HardwareQubit(_) => {}
                }
            }
            self.call_function(name, &args)?;
            return Ok(ControlFlow::None);
        }

        let resolved = self.resolve_gate(name, &func, &param_values)?;

        let combinations = cartesian_product(&qubit_groups);
        for qubit_indices in combinations {
            let modified = self.apply_modifiers_to_gate(resolved, modifiers)?;
            let sv = self.state_vector.as_mut().ok_or(RuntimeError::NoStateVector)?;
            modified.apply_to_statevector(sv, &qubit_indices);
        }

        Ok(ControlFlow::None)
    }

    fn resolve_gate(&mut self, name: &str, func: &Function, param_values: &[f64]) -> Result<ResolvedGate, RuntimeError> {
        match func {
            Function::BuiltInGate(gate) => {
                gate.get_resolved(param_values)
            }
            Function::Gate { params, qubits, body } => {
                let param_bits: Vec<u64> = param_values.iter().map(|f| f.to_bits()).collect();
                let cache_key = (name.to_string(), param_bits);

                if let Some(cached) = self.gate_cache.get(&cache_key) { return Ok(*cached); }

                let resolved = self.extract_gate_matrix(params, qubits, body, param_values)?;
                self.gate_cache.insert(cache_key, resolved);
                Ok(resolved)
            }
            _ => Err(RuntimeError::UndefinedFunction("not a gate".to_string()))
        }
    }

    fn extract_gate_matrix(&mut self, param_names: &[String], qubit_names: &[String], body: &[Stmt], param_values: &[f64]) -> Result<ResolvedGate, RuntimeError> {
        let n = qubit_names.len();
        let size = 1 << n;
        let mut columns: Vec<Vec<C64>> = vec![];

        for col in 0..size {
            let mut sv = StateVector::new(n);
            sv.amplitudes = vec![C64::new(0.0, 0.0); size];
            sv.amplitudes[col] = C64::new(1.0, 0.0);

            let old_sv = self.state_vector.replace(sv);
            let old_num_qubits = self.num_qubits;
            self.num_qubits = n;

            let old_qubit_entries: Vec<(String, Vec<usize>)> = qubit_names.iter().enumerate()
                .map(|(i, name)| {
                    let old = self.qubit_map.remove(name);
                    self.qubit_map.insert(name.clone(), vec![i]);
                    (name.clone(), old.unwrap_or_default())
                })
                .collect();

            self.push_scope();
            for (pname, pval) in param_names.iter().zip(param_values.iter()) {
                self.define(pname.clone(), Value::Float(*pval), false)?;
            }
            let result = self.interpret_statements(body);
            self.pop_scope();

            let col_amps = self.state_vector.as_ref().unwrap().amplitudes.clone();
            columns.push(col_amps);

            self.state_vector = old_sv;
            self.state_vector.as_ref().map(|sv| sv.amplitudes.len()).unwrap_or(0);
            self.num_qubits = old_num_qubits;
            for (name, old_indices) in old_qubit_entries {
                self.qubit_map.remove(&name);
                if !old_indices.is_empty() { self.qubit_map.insert(name, old_indices); }
            }

            result?;
        }

        match n {
            1 => {
                let m = [[columns[0][0], columns[1][0]], [columns[0][1], columns[1][1]]];
                Ok(ResolvedGate::SingleQubit(m))
            }
            2 => {
                let mut m = [[C64::new(0.0,0.0); 4]; 4];
                for col in 0..4 {
                    for row in 0..4 {
                        m[row][col] = columns[col][row];
                    }
                }

                use crate::interpreter::quantum::gates::swap_qubit_order;
                let canonical_m = swap_qubit_order(&m);

                Ok(ResolvedGate::TwoQubit(canonical_m))
            }
            3 => {
                let mut m = [[C64::new(0.0,0.0); 8]; 8];
                for col in 0..8 {
                    for row in 0..8 {
                        m[row][col] = columns[col][row];
                    }
                }
                Ok(ResolvedGate::ThreeQubit(m))
            }
            _ => Err(RuntimeError::UnsupportedOperation("gates with >3 qubits not supported".to_string()))
        }
    }

    fn apply_modifiers_to_gate(&mut self, mut gate: ResolvedGate, modifiers: &[GateModifier]) -> Result<ResolvedGate, RuntimeError> {
        for modifier in modifiers.iter().rev() {
            gate = match modifier {
                GateModifier::Inv => gate.apply_inv(),
                GateModifier::Pow(expr) => {
                    let n = match self.evaluate_expression(expr)? {
                        Value::Int(i) => i,
                        Value::Float(f) => f as i64,
                        _ => return Err(RuntimeError::TypeMismatch("pow requires numeric".to_string())),
                    };
                    gate.apply_pow(n)
                }
                GateModifier::Ctrl(count) => {
                    let n = match count {
                        Some(expr) => match self.evaluate_expression(expr)? {
                            Value::Int(i) => i as usize,
                            Value::Float(f) => f as usize,
                            _ => return Err(RuntimeError::TypeMismatch("ctrl count must be int".to_string())),
                        },
                        None => 1,
                    };
                    for _ in 0..n { gate = gate.apply_ctrl(); }
                    gate
                }
                GateModifier::NegCtrl(count) => {
                    let n = match count {
                        Some(expr) => match self.evaluate_expression(expr)? {
                            Value::Int(i) => i as usize,
                            Value::Float(f) => f as usize,
                            _ => return Err(RuntimeError::TypeMismatch("negctrl count must be int".to_string())),
                        },
                        None => 1,
                    };
                    for _ in 0..n { gate = gate.apply_neg_ctrl(); }
                    gate
                }
            };
        }
        Ok(gate)
    }

    fn interpret_measure(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::Assign { target, value, .. } = stmt else {
            unreachable!();
        };

        let Expr::Measure(operand) = value else {
            unreachable!();
        };

        let qubit_indices = self.resolve_qubits(operand)?;
        let value = self.measure_qubits(qubit_indices)?;

        if target.indices.is_empty() { self.assign(&target.name, value)?; }
        else { self.assign_indexed(&target.name, &target.indices, value)?; }

        Ok(ControlFlow::None)
    }

    fn measure_qubits(&mut self, qubit_indices: Vec<usize>) -> Result<Value, RuntimeError> {
        let mut result_value: u64 = 0;
        let width = qubit_indices.len();

        for (bit_pos, &qubit_idx) in qubit_indices.iter().enumerate() {
            let sv = self.state_vector.as_mut().ok_or(RuntimeError::NoStateVector)?;
            let outcome = sv.measure_qubit(qubit_idx);
            if outcome {
                result_value |= 1 << bit_pos;
            }
        }

        Ok(Value::Bits { value: result_value, width })
    }

    fn evaluate_int_bound(&mut self, e: &Option<Box<Expr>>) -> Result<Option<i64>, RuntimeError> {
        match e {
            Some(e) => match self.evaluate_expression(e)? {
                Value::Int(i) => Ok(Some(i)),
                _ => Err(RuntimeError::TypeMismatch("range bound must be int".to_string())),
            },
            None => Ok(None),
        }
    }

    fn evaluate_range(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        let Expr::Range { start, stop, step } = expr else {
            unreachable!();
        };
        Ok(Value::Range {
            start: self.evaluate_int_bound(start)?,
            stop: self.evaluate_int_bound(stop)?,
            step: self.evaluate_int_bound(step)?,
        })
    }

    fn interpret_let(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let Stmt::Let { name, value} = stmt else {
            unreachable!();
        };

        match self.evaluate_expression(value)? {
            Value::Qubit(indices) => {
                self.qubit_map.insert(name.clone(), indices);
                Ok(ControlFlow::None)
            }
            Value::Array(arr) => {
                let mut all_qubits = true;
                let mut flattened_indices = Vec::new();

                for val in &arr {
                    match val {
                        Value::Qubit(indices) => {
                            flattened_indices.extend(indices);
                        }
                        _ => {
                            all_qubits = false;
                            break;
                        }
                    }
                }

                if all_qubits {
                    self.qubit_map.insert(name.clone(), flattened_indices);
                    Ok(ControlFlow::None)
                } else {
                    self.define(name.clone(), Value::Array(arr), true)?;
                    Ok(ControlFlow::None)
                }
            }
            other => {
                self.define(name.clone(), other, true)?;
                Ok(ControlFlow::None)
            }
        }
    }

    fn ensure_hardware_qubit(&mut self, index: usize) {
        if index >= self.num_qubits {
            let needed = index + 1 - self.num_qubits;
            let name = format!("$hw_{}", index);
            self.allocate_qubits(&name, needed);
        }
    }
}