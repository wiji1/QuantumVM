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
use crate::interpreter::runtime_error::{RuntimeError, RuntimeErrorKind};
use crate::interpreter::value::Value;
    use crate::lexer::{Lexer, Span};
use crate::parser::expression::{Expr, ExprKind};
use crate::parser::statement::{Stmt, StmtKind};
use crate::parser::supporting_types::{AssignOp, BinaryOp, ClassicalType, ForIter, GateModifier, GateOperand, IndexedIdent, IoDirection, SwitchCase, UnaryOp};
use crate::parser::{Parser, Program};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use crate::coercion::{cast_value, coerce_to_bool, coerce_value, infer_type_from_value};
use crate::type_checker::type_repr::Type;

macro_rules! numeric_op {
    ($lhs:expr, $rhs:expr, $op:tt, $expr:expr, $span:expr) => {
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
            _ => Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation($expr.to_string()), $span)),
        }
    };
}

macro_rules! bitwise_op {
    ($lhs:expr, $rhs:expr, $op:tt, $expr:expr, $span:expr) => {
        match ($lhs, $rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a $op b)),
            (Value::Bits { value: a, width: wa }, Value::Bits { value: b, width: wb }) => {
                if wa != wb { return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch($expr.to_string()), $span)); }
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
            _ => Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation($expr.to_string()), $span))
        }
    };
}

macro_rules! comparison_op {
    ($lhs:expr, $rhs:expr, $op:tt, $expr:expr, $span:expr) => {
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
            _ => Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation($expr.to_string()), $span))
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

    fn normalize_index(idx: i64, len: usize, span: Span) -> Result<usize, RuntimeError> {
        if idx < 0 { return Err(RuntimeError::with_span(RuntimeErrorKind::InvalidIndex, span.clone())); }

        let idx_usize = idx as usize;
        if idx_usize >= len { return Err(RuntimeError::with_span(RuntimeErrorKind::IndexOutOfBounds(idx_usize, len), span)); }
        Ok(idx_usize)
    }

    pub fn start(&mut self) -> Result<(), RuntimeError> {
        self.push_scope();

        self.define_default_operations()?;

        for stmt in self.program.statements.clone() {
            match self.interpret_statement(&stmt)? {
                ControlFlow::Return(_) => break,
                _ => {}
            }
        }

        for output in self.outputs.clone().keys() {
            if let Some(value) = self.lookup(output) {
                self.outputs.insert(output.to_string(), value.clone());
            }
        }

        self.pop_scope();
        Ok(())
    }

    fn define(&mut self, name: String, value: Value, const_decl: bool) -> Result<ControlFlow, RuntimeError> {
        if self.scopes.last().unwrap().contains_key(&name) {
            return Err(RuntimeError::new(RuntimeErrorKind::DuplicateVariable(name)));
        }

        if const_decl { self.constants.insert(name.to_string()); }
        self.scopes.last_mut().unwrap().insert(name, value);
        Ok(ControlFlow::None)
    }

    fn assign(&mut self, name: &str, value: Value) -> Result<ControlFlow, RuntimeError> {
        if self.constants.contains(name) {
            return Err(RuntimeError::new(RuntimeErrorKind::ConstReassignment(name.to_string())));
        }

        for scope in self.scopes.iter_mut().rev() {
            if let Some(current_value) = scope.get(name) {
                let target_type = infer_type_from_value(current_value);

                let coerced_value = coerce_value(value, &target_type)
                    .map_err(|e| RuntimeError::new(RuntimeErrorKind::TypeMismatch(e.to_string())))?;

                scope.insert(name.to_string(), coerced_value);
                return Ok(ControlFlow::None);
            }
        }
        Err(RuntimeError::new(RuntimeErrorKind::UndefinedVariable(name.to_string())))
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
        match &stmt.kind {
            StmtKind::QuantumDecl { .. } => self.interpret_quantum_decl(stmt),
            StmtKind::ClassicalDecl { .. } => self.interpret_classical_declaration(stmt),
            StmtKind::ArrayDecl { .. } => self.interpret_array_declaration(stmt),
            StmtKind::ConstDecl { .. } => self.interpret_const_declaration(stmt),
            StmtKind::GateCall { .. } => self.interpret_gate_call(stmt),
            StmtKind::If { .. } => self.interpret_if(stmt),
            StmtKind::Switch { .. } => self.interpret_switch(stmt),
            StmtKind::For { .. } => self.interpret_for(stmt),
            StmtKind::While { .. } => self.interpret_while(stmt),
            StmtKind::IoDecl { .. } => self.interpret_io_decl(stmt),
            StmtKind::Include(s) => self.interpret_include(s),
            StmtKind::IncludeFromSrc(_, s) => self.interpret_include_from_src(s),
            StmtKind::Let { .. } => self.interpret_let(stmt),
            StmtKind::Barrier { .. } => Ok(ControlFlow::None),
            StmtKind::Pragma => Ok(ControlFlow::None),
            StmtKind::Continue => Ok(ControlFlow::Continue),
            StmtKind::Break => Ok(ControlFlow::Break),
            StmtKind::Reset { qubit } => {
                let qubit_indices = self.resolve_qubits(qubit, stmt.span.clone())?;
                let sv = self.state_vector.as_mut().ok_or(RuntimeError::with_span(RuntimeErrorKind::NoStateVector, stmt.span.clone()))?;
                for qubit_index in qubit_indices { sv.reset_qubit(qubit_index); }
                Ok(ControlFlow::None)
            }
            StmtKind::ExpressionStatement(expr) => {
                self.evaluate_expression(expr)?;
                Ok(ControlFlow::None)
            }
            StmtKind::Return(expr) => {
                let value = match expr {
                    Some(e) => self.evaluate_expression(e)?,
                    None => Value::Void,
                };
                Ok(ControlFlow::Return(value))
            },
            StmtKind::GateDef { name, name_span: _, params, qubits, body } => {
                if self.functions.contains_key(name) {
                    return Err(RuntimeError::new(RuntimeErrorKind::DuplicateGate(name.to_string())));
                }
                self.functions.insert(name.to_string(), Function::Gate {
                    params: params.iter().map(|(name, _)| name.clone()).collect(),
                    qubits: qubits.iter().map(|(name, _)| name.clone()).collect(),
                    body: body.clone(),
                });

                Ok(ControlFlow::None)
            },
            StmtKind::Assign { value, .. } => {
                match &value.kind {
                    ExprKind::Measure(_) => self.interpret_measure(stmt),
                    _ => self.interpret_assignment(stmt)
                }
            },
            StmtKind::Def { name, name_span: _, params, return_type, body } => {
                if self.functions.contains_key(name) {
                    return Err(RuntimeError::new(RuntimeErrorKind::DuplicateFunction(name.to_string())));
                }
                self.functions.insert(name.clone(), Function::UserDefined {
                    params: params.clone(),
                    return_type: return_type.clone(),
                    body: body.clone(),
                });
                Ok(ControlFlow::None)
            }
            StmtKind::Block(stmts) => {
                self.push_scope();
                let flow = self.interpret_statements(stmts)?;
                self.pop_scope();

                Ok(flow)
            },
            StmtKind::Extern { name } => {
                self.functions.insert(name.clone(), Function::Extern);
                Ok(ControlFlow::None)
            }
            StmtKind::NoOp => Ok(ControlFlow::None)
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
        let StmtKind::ClassicalDecl { ty, name, name_span: _, init } = &stmt.kind else {
            unreachable!("Incorrect statement signature!");
        };

        let default_value = ty.get_default_value(ty.get_size())?;

        let init_value = match init {
            Some(expr) => self.evaluate_expression(expr)?,
            None => default_value,
        };

        let target_type = Type::from_classical_type(ty);
        let coerced_value = coerce_value(init_value, &target_type)
            .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), stmt.span.clone()))?;

        self.define(name.to_string(), coerced_value, false)?;

        Ok(ControlFlow::None)
    }

    fn interpret_array_declaration(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let StmtKind::ArrayDecl { ty, name, name_span: _, size, init} = &stmt.kind else {
            unreachable!("Incorrect statement signature!");
        };

        let mut dimensions: Vec<i64> = vec![];

        for size_expr in size {
            let evaluated = self.evaluate_expression(size_expr)?;
            match evaluated {
                Value::Int(i) => dimensions.push(i),
                _ => return Err(RuntimeError::with_span(RuntimeErrorKind::InvalidSize, size_expr.span.clone())),
            }
        }

        if dimensions.is_empty() || dimensions.len() > 7 { return Err(RuntimeError::with_span(RuntimeErrorKind::InvalidSize, stmt.span.clone())); }

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
        let StmtKind::ConstDecl { ty: _, name, name_span: _, init } = &stmt.kind else {
            unreachable!("Incorrect statement signature!");
        };

        let init_value = self.evaluate_expression(init)?;
        self.define(name.to_string(), init_value, true)?;

        Ok(ControlFlow::None)
    }

    fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {

        match &expr.kind {
            ExprKind::Int(i) => { Ok(Value::Int(*i)) }
            ExprKind::Float(f) => { Ok(Value::Float(*f)) }
            ExprKind::Bool(b) => { Ok(Value::Bool(*b)) }
            ExprKind::Array(a) => { self.evaluate_array(a) }
            ExprKind::Imaginary(f) => { Ok(Value::Complex(0.0, *f)) }
            ExprKind::Timing(s) => { Ok(Value::Timing(s.clone())) }
            ExprKind::Bits(v, w) => { Ok(Value::Bits {value: *v, width: *w})}
            ExprKind::Ident(name) => {
                if let Some(val) = self.lookup(name) { Ok(val.clone()) }
                else if let Some(indices) = self.qubit_map.get(name) {
                    Ok(Value::Qubit(indices.clone()))
                } else { Err(RuntimeError::with_span(RuntimeErrorKind::UndefinedVariable(name.clone()), expr.span.clone())) }
            }
            ExprKind::IndexedIdent(i) => { self.evaluate_indexed_ident(i, expr.span.clone()) }
            ExprKind::Measure(op) => { self.evaluate_measure(op, expr.span.clone()) }
            ExprKind::Unary { .. } => { self.evaluate_unary(expr) }
            ExprKind::Binary { .. } => { self.evaluate_binary(expr) }
            ExprKind::Call { name, name_span: _, args } => { self.call_function(name, args, expr.span.clone()) }
            ExprKind::Cast { ty, expr: inner_expr } => {
                let value = self.evaluate_expression(inner_expr)?;
                let target_type = Type::from_classical_type(ty);
                cast_value(value, &target_type)
                    .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), expr.span.clone()))
            }
            ExprKind::Range { .. } => { self.evaluate_range(expr) }
        }
    }

    fn evaluate_array(&mut self, array: &Box<Vec<Expr>>) -> Result<Value, RuntimeError> {
        let mut vec: Vec<Value> = vec![];
        for expr in array.iter() {
            vec.push(self.evaluate_expression(expr)?);
        }

        Ok(Value::Array(vec))
    }

    fn evaluate_indexed_ident(&mut self, ident: &IndexedIdent, span: Span) -> Result<Value, RuntimeError> {
        if let Some(indices) = self.qubit_map.get(&ident.name).cloned() {
            return self.evaluate_qubit_ident(indices, &ident.indices, span);
        }

        let mut value = self.lookup(&ident.name).cloned()
            .ok_or(RuntimeError::with_span(RuntimeErrorKind::UndefinedVariable(ident.name.clone()), span.clone()))?;

        for (idx, expr) in ident.indices.iter().enumerate() {
            let index_val = self.evaluate_expression(expr)?;
            let index_span = if idx < ident.indices.len() { expr.span.clone() } else { span.clone() };
            value = match (value, index_val) {
                (Value::Array(arr), Value::Int(i)) => {
                    let len = arr.len();
                    let normalized_idx = Self::normalize_index(i, len, index_span.clone())?;
                    arr.into_iter().nth(normalized_idx)
                        .ok_or(RuntimeError::with_span(RuntimeErrorKind::IndexOutOfBounds(normalized_idx, len), index_span))?
                }
                (Value::Array(arr), Value::Range { start, stop, step }) => {
                    let len = arr.len();
                    let start = if let Some(s) = start { Self::normalize_index(s, len, index_span.clone())? }
                    else { 0 };
                    let stop = if let Some(s) = stop { Self::normalize_index(s, len, index_span.clone())? }
                    else { len - 1 };
                    let step = step.unwrap_or(1) as usize;
                    Value::Array((start..=stop).step_by(step).map(|i| arr[i].clone()).collect())
                }
                (Value::Bits { value, width }, Value::Int(i)) => {
                    let normalized_idx = if i < 0 {
                        let pos = width as i64 + i;
                        if pos < 0 { return Err(RuntimeError::with_span(RuntimeErrorKind::IndexOutOfBounds(i as usize, width), index_span)); }
                        pos as usize
                    } else {
                        let idx = i as usize;
                        if idx >= width { return Err(RuntimeError::with_span(RuntimeErrorKind::IndexOutOfBounds(idx, width), index_span)); }
                        idx
                    };
                    Value::Bits { value: (value >> normalized_idx) & 1, width: 1 }
                }
                _ => return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("cannot index with this type".to_string()), index_span)),
            };
        }
        Ok(value)
    }

    fn evaluate_qubit_ident(&mut self, indices: Vec<usize>, exprs: &[Expr], span: Span) -> Result<Value, RuntimeError> {
        if exprs.is_empty() { return Ok(Value::Qubit(indices)); }

        let index_val = self.evaluate_expression(&exprs[0])?;
        let index_span = exprs[0].span.clone();

        let selected = match index_val {
            Value::Int(i) => {
                let qubit = indices.get(i as usize)
                    .copied()
                    .ok_or(RuntimeError::with_span(RuntimeErrorKind::IndexOutOfBounds(i as usize, indices.len()), index_span.clone()))?;
                vec![qubit]
            }
            Value::Array(arr) => {
                arr.into_iter().map(|val| match val {
                    Value::Int(i) => indices.get(i as usize)
                        .copied()
                        .ok_or(RuntimeError::with_span(RuntimeErrorKind::IndexOutOfBounds(i as usize, indices.len()), index_span.clone())),
                    _ => Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("qubit set index must be int".to_string()), index_span.clone())),
                }).collect::<Result<Vec<_>, _>>()?
            }
            Value::Range { start, stop, step } => {
                let start = start.unwrap_or(0) as usize;
                let stop = stop.unwrap_or(indices.len() as i64) as usize;
                let step = step.unwrap_or(1) as usize;
                (start..=stop).step_by(step)
                    .map(|i| indices.get(i).copied().ok_or(RuntimeError::with_span(RuntimeErrorKind::IndexOutOfBounds(i, indices.len()), index_span.clone())))
                    .collect::<Result<Vec<_>, _>>()?
            }
            other => {
                let coerced = coerce_value(other, &Type::Int(None))
                    .map_err(|_| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("qubit index must be int, range, or set".to_string()), index_span.clone()))?;
                match coerced {
                    Value::Int(i) => {
                        let qubit = indices.get(i as usize)
                            .copied()
                            .ok_or(RuntimeError::with_span(RuntimeErrorKind::IndexOutOfBounds(i as usize, indices.len()), index_span.clone()))?;
                        vec![qubit]
                    }
                    _ => return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("qubit index coercion did not produce int".to_string()), index_span)),
                }
            }
        };

        if exprs.len() > 1 { return self.evaluate_qubit_ident(selected, &exprs[1..], span); }

        Ok(Value::Qubit(selected))
    }

    fn evaluate_unary(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        let ExprKind::Unary { op, expr: inner } = &expr.kind else {
            unreachable!("Incorrect statement signature!");
        };

        let evaluated = self.evaluate_expression(&inner)?;

        match op {
            UnaryOp::Neg => {
                match evaluated {
                    Value::Int(i) => { Ok(Value::Int(-i)) }
                    Value::Float(f) => { Ok(Value::Float(-f)) }
                    _ => { Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation(format!("negation of {}", inner.to_string())), expr.span.clone())) }
                }
            }
            UnaryOp::BitNot => {
                match evaluated {
                    Value::Int(i) => { Ok(Value::Int(!i)) },
                    Value::Bits {value, width} => {
                        let mask = (1u64 << width) - 1;
                        Ok(Value::Bits { value: !value & mask, width })
                    }
                    _ => { Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation(format!("bitwise NOT of {}", inner.to_string())), expr.span.clone())) }
                }
            }
            UnaryOp::LogicNot => {
                match evaluated {
                    Value::Bool(b) => { Ok(Value::Bool(!b)) },
                    _ => { Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation(format!("logical NOT of {}", inner.to_string())), expr.span.clone())) }
                }
            }
        }
    }

    fn evaluate_binary(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        let ExprKind::Binary { op, lhs, rhs } = &expr.kind else {
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
            if is_zero { return Err(RuntimeError::with_span(RuntimeErrorKind::DivideByZero, expr.span.clone())); }
        }

        match op {
            BinaryOp::Add => numeric_op!(lhs_evaluated, rhs_evaluated, +, expr, expr.span.clone()),
            BinaryOp::Sub => numeric_op!(lhs_evaluated, rhs_evaluated, -, expr, expr.span.clone()),
            BinaryOp::Mul => numeric_op!(lhs_evaluated, rhs_evaluated, *, expr, expr.span.clone()),
            BinaryOp::Div => match (lhs_evaluated, rhs_evaluated) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Float(a as f64 / b as f64)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 / b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / b as f64)),
                _ => Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation(expr.to_string()), expr.span.clone()))
            },
            BinaryOp::Mod => numeric_op!(lhs_evaluated, rhs_evaluated, %, expr, expr.span.clone()),
            BinaryOp::Pow => match (lhs_evaluated, rhs_evaluated) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Float((a as f64).powf(b as f64))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((a as f64).powf(b))),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.powf(b as f64))),
                _ => Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation(expr.to_string()), expr.span.clone()))
            },
            BinaryOp::And => bitwise_op!(lhs_evaluated, rhs_evaluated, &, expr, expr.span.clone()),
            BinaryOp::Or  => bitwise_op!(lhs_evaluated, rhs_evaluated, |, expr, expr.span.clone()),
            BinaryOp::Xor => bitwise_op!(lhs_evaluated, rhs_evaluated, ^, expr, expr.span.clone()),
            BinaryOp::Shl => bitwise_op!(lhs_evaluated, rhs_evaluated, <<, expr, expr.span.clone()),
            BinaryOp::Shr => bitwise_op!(lhs_evaluated, rhs_evaluated, >>, expr, expr.span.clone()),
            BinaryOp::LogicAnd => {
                let lhs_bool = coerce_to_bool(lhs_evaluated)
                    .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), expr.span.clone()))?;
                let rhs_bool = coerce_to_bool(rhs_evaluated)
                    .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), expr.span.clone()))?;

                match (lhs_bool, rhs_bool) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                    _ => unreachable!("coerce_to_bool should always return Bool"),
                }
            },
            BinaryOp::LogicOr => {
                let lhs_bool = coerce_to_bool(lhs_evaluated)
                    .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), expr.span.clone()))?;
                let rhs_bool = coerce_to_bool(rhs_evaluated)
                    .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), expr.span.clone()))?;

                match (lhs_bool, rhs_bool) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                    _ => unreachable!("coerce_to_bool should always return Bool"),
                }
            },
            BinaryOp::Eq  => comparison_op!(lhs_evaluated, rhs_evaluated, ==, expr, expr.span.clone()),
            BinaryOp::Neq => comparison_op!(lhs_evaluated, rhs_evaluated, !=, expr, expr.span.clone()),
            BinaryOp::Lt  => comparison_op!(lhs_evaluated, rhs_evaluated, <,  expr, expr.span.clone()),
            BinaryOp::Gt  => comparison_op!(lhs_evaluated, rhs_evaluated, >,  expr, expr.span.clone()),
            BinaryOp::Leq => comparison_op!(lhs_evaluated, rhs_evaluated, <=, expr, expr.span.clone()),
            BinaryOp::Geq => comparison_op!(lhs_evaluated, rhs_evaluated, >=, expr, expr.span.clone()),
            BinaryOp::Concat => self.concatenate_values(&lhs_evaluated, &rhs_evaluated, expr.span.clone()),
        }
    }

    fn evaluate_measure(&mut self, operand: &GateOperand, span: Span) -> Result<Value, RuntimeError> {
        let qubit_indices = self.resolve_qubits(operand, span.clone())?;
        self.measure_qubits(qubit_indices, span)
    }

    fn interpret_assignment(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let StmtKind::Assign { target, op, value } = &stmt.kind else {
            unreachable!("Incorrect statement signature!");
        };

        let rhs = self.evaluate_expression(value)?;

        let evaluated = match op {
            AssignOp::Eq => rhs,
            AssignOp::Compound(bin_op) => {
                let current = if target.indices.is_empty() {
                    self.lookup(&target.name)
                        .cloned()
                        .ok_or(RuntimeError::with_span(RuntimeErrorKind::UndefinedVariable(target.name.clone()), stmt.span.clone()))?
                } else {
                    self.evaluate_indexed_ident(&IndexedIdent {
                        name: target.name.clone(),
                        indices: target.indices.clone(),
                        span: None,
                    }, stmt.span.clone())?
                };
                self.apply_binary_op(bin_op, current, rhs, stmt.span.clone())?
            }
        };

        if target.indices.is_empty() {
            self.assign(&target.name, evaluated)
        } else {
            self.assign_indexed(&target.name, &target.indices, evaluated, stmt.span.clone())
        }
    }

    fn concatenate_values(&self, lhs: &Value, rhs: &Value, span: Span) -> Result<Value, RuntimeError> {
        match (lhs, rhs) {
            (Value::Qubit(indices_a), Value::Qubit(indices_b)) => {
                let set_a: HashSet<_> = indices_a.iter().collect();
                let set_b: HashSet<_> = indices_b.iter().collect();

                if set_a.intersection(&set_b).count() > 0 {
                    return Err(RuntimeError::with_span(RuntimeErrorKind::SelfConcatenation(
                        "Cannot concatenate a qubit register with itself".to_string()
                    ), span));
                }

                let mut result = indices_a.clone();
                result.extend(indices_b);
                Ok(Value::Qubit(result))
            }

            (Value::Bits { value: v1, width: w1 }, Value::Bits { value: v2, width: w2 }) => {
                let new_width = w1 + w2;
                if new_width > 64 {
                    return Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation(
                        format!("Bit concatenation exceeds 64-bit limit: {} + {}", w1, w2)
                    ), span));
                }

                let new_value = (v2 << w1) | v1;
                Ok(Value::Bits { value: new_value, width: new_width })
            }

            (Value::Array(arr1), Value::Array(arr2)) => {
                let mut result = arr1.clone();
                result.extend(arr2.clone());
                Ok(Value::Array(result))
            }

            _ => Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation(
                format!("Cannot concatenate {:?} and {:?}", lhs, rhs)
            ), span))
        }
    }

    fn apply_binary_op(&self, op: &BinaryOp, lhs: Value, rhs: Value, span: Span) -> Result<Value, RuntimeError> {
        if matches!(op, BinaryOp::Div | BinaryOp::Mod) {
            let is_zero = match &rhs {
                Value::Int(i) => *i == 0,
                Value::Float(f) => *f == 0.0,
                _ => false,
            };
            if is_zero { return Err(RuntimeError::with_span(RuntimeErrorKind::DivideByZero, span)); }
        }

        match op {
            BinaryOp::Add => numeric_op!(lhs, rhs, +, "+=", span),
            BinaryOp::Sub => numeric_op!(lhs, rhs, -, "-=", span),
            BinaryOp::Mul => numeric_op!(lhs, rhs, *, "*=", span),
            BinaryOp::Div => match (lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Float(a as f64 / b as f64)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 / b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / b as f64)),
                _ => Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation("/".to_string()), span))
            },
            BinaryOp::Mod => numeric_op!(lhs, rhs, %, "%=", span),
            BinaryOp::And => {
                if matches!(lhs, Value::Bool(_)) || matches!(rhs, Value::Bool(_)) {
                    let lhs_bool = coerce_to_bool(lhs)
                        .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), span.clone()))?;
                    let rhs_bool = coerce_to_bool(rhs)
                        .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), span.clone()))?;
                    match (lhs_bool, rhs_bool) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a & b)),
                        _ => unreachable!(),
                    }
                } else { bitwise_op!(lhs, rhs, &, "&=", span) }
            },
            BinaryOp::Or => {
                if matches!(lhs, Value::Bool(_)) || matches!(rhs, Value::Bool(_)) {
                    let lhs_bool = coerce_to_bool(lhs)
                        .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), span.clone()))?;
                    let rhs_bool = coerce_to_bool(rhs)
                        .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), span.clone()))?;
                    match (lhs_bool, rhs_bool) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a | b)),
                        _ => unreachable!(),
                    }
                } else { bitwise_op!(lhs, rhs, |, "|=", span) }
            },
            BinaryOp::Xor => {
                if matches!(lhs, Value::Bool(_)) || matches!(rhs, Value::Bool(_)) {
                    let lhs_bool = coerce_to_bool(lhs)
                        .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), span.clone()))?;
                    let rhs_bool = coerce_to_bool(rhs)
                        .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), span.clone()))?;
                    match (lhs_bool, rhs_bool) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a ^ b)),
                        _ => unreachable!(),
                    }
                } else { bitwise_op!(lhs, rhs, ^, "^=", span) }
            },
            BinaryOp::Shl => bitwise_op!(lhs, rhs, <<, "<<=", span),
            BinaryOp::Shr => bitwise_op!(lhs, rhs, >>, ">>=", span),
            BinaryOp::Pow => match (lhs, rhs) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Float((a as f64).powf(b as f64))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((a as f64).powf(b))),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.powf(b as f64))),
                _ => Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation("**=".to_string()), span.clone()))
            },
            BinaryOp::Concat => self.concatenate_values(&lhs, &rhs, span.clone()),
            _ => Err(RuntimeError::with_span(RuntimeErrorKind::UnsupportedOperation(format!("{:?}", op)), span))
        }
    }

    fn assign_indexed(&mut self, name: &str, indices: &[Expr], new_value: Value, span: Span) -> Result<ControlFlow, RuntimeError> {
        let first_index = self.evaluate_expression(&indices[0])?;

        if let Value::Range { start, stop, step } = first_index {
            let new_values = match new_value {
                Value::Array(v) => v,
                _ => return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("slice assignment requires array".to_string()), span)),
            };

            let (normalized_start, normalized_stop, step_val) = {
                let target = self.lookup(name).ok_or(RuntimeError::with_span(RuntimeErrorKind::NullPointer, span.clone()))?;
                if let Value::Array(arr) = target {
                    let len = arr.len();
                    let start_idx = if let Some(s) = start { Self::normalize_index(s, len, span.clone())? }
                    else { 0 };
                    let stop_idx = if let Some(s) = stop { Self::normalize_index(s, len, span.clone())? }
                    else { len - 1 };
                    (start_idx, stop_idx, step.unwrap_or(1) as usize)
                } else {
                    return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("cannot slice non-array".to_string()), span.clone()));
                }
            };

            let target = self.lookup_mut(name).ok_or(RuntimeError::with_span(RuntimeErrorKind::NullPointer, span.clone()))?;
            if let Value::Array(arr) = target {
                for (i, val) in (normalized_start..=normalized_stop).step_by(step_val).zip(new_values) {
                    arr[i] = val;
                }
                return Ok(ControlFlow::None);
            }
            return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("cannot slice non-array".to_string()), span));
        }

        let index_span = indices[0].span.clone();
        let evaluated_indices: Vec<i64> = indices.iter().map(|expr| match self.evaluate_expression(expr) {
            Ok(Value::Int(i)) => Ok(i),
            Ok(_) => Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("index must be int".to_string()), expr.span.clone())),
            Err(e) => Err(e),
        }).collect::<Result<Vec<i64>, RuntimeError>>()?;

        let is_bits = matches!(self.lookup(name), Some(Value::Bits { .. }));

        if is_bits {
            let target = self.lookup_mut(name).ok_or(RuntimeError::with_span(RuntimeErrorKind::NullPointer, span.clone()))?;
            if let Value::Bits { value, width } = target {
                let bit_pos = if evaluated_indices[0] < 0 {
                    let pos = *width as i64 + evaluated_indices[0];
                    if pos < 0 { return Err(RuntimeError::with_span(RuntimeErrorKind::IndexOutOfBounds(evaluated_indices[0] as usize, *width), index_span)); }
                    pos as usize
                } else {
                    let idx = evaluated_indices[0] as usize;
                    if idx >= *width { return Err(RuntimeError::with_span(RuntimeErrorKind::IndexOutOfBounds(idx, *width), index_span)); }
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
                    _ => return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("cannot assign to bit register".to_string()), span)),
                }
                return Ok(ControlFlow::None);
            }
        }

        let target = self.lookup_mut(name).ok_or(RuntimeError::with_span(RuntimeErrorKind::NullPointer, span.clone()))?;
        match target {
            Value::Array(a) => {
                Self::set_nested_with_i64_indices(a, &evaluated_indices, new_value, span)?;
                Ok(ControlFlow::None)
            }
            _ => Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("only arrays and bit registers can be index assigned".to_string()), span))
        }
    }

    fn set_nested_evaluated(arr: &mut Vec<Value>, indices: &[usize], value: Value) -> Result<ControlFlow, RuntimeError> {
        let index = indices[0];

        let len = arr.len();
        let elem = arr.get_mut(index).ok_or(RuntimeError::new(RuntimeErrorKind::IndexOutOfBounds(index, len)))?;

        if indices.len() == 1 { *elem = value }
        else {
            match elem {
                Value::Array(inner) => Self::set_nested_evaluated(inner, &indices[1..], value)?,
                _ => return Err(RuntimeError::new(RuntimeErrorKind::TypeMismatch("cannot index non-array".to_string()))),
            };
        }

        Ok(ControlFlow::None)
    }

    fn set_nested_with_i64_indices(arr: &mut Vec<Value>, indices: &[i64], value: Value, span: Span) -> Result<ControlFlow, RuntimeError> {
        let len = arr.len();
        let normalized_idx = Self::normalize_index(indices[0], len, span.clone())?;
        let elem = arr.get_mut(normalized_idx).ok_or(RuntimeError::with_span(RuntimeErrorKind::IndexOutOfBounds(normalized_idx, len), span.clone()))?;

        if indices.len() == 1 { *elem = value }
        else {
            match elem {
                Value::Array(inner) => Self::set_nested_with_i64_indices(inner, &indices[1..], value, span.clone())?,
                _ => return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("cannot index non-array".to_string()), span)),
            };
        }

        Ok(ControlFlow::None)
    }

    fn interpret_if(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let StmtKind::If { cond, then, else_ } = &stmt.kind else {
            unreachable!("Incorrect statement signature!");
        };

        let evaluated = self.evaluate_expression(cond)?;

        let coerced = coerce_to_bool(evaluated)
            .map_err(|e| RuntimeError::new(RuntimeErrorKind::TypeMismatch(e.to_string())))?;

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
        let StmtKind::Switch { expr, cases } = &stmt.kind else {
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
        let StmtKind::While { cond, body } = &stmt.kind else {
            unreachable!("Incorrect statement signature!");
        };

        loop {
            let evaluated = self.evaluate_expression(cond)?;

            let coerced = coerce_to_bool(evaluated)
                .map_err(|e| RuntimeError::with_span(RuntimeErrorKind::TypeMismatch(e.to_string()), cond.span.clone()))?;

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
        let StmtKind::For { var, ty, iter, body } = &stmt.kind else {
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
                    _ => return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("range start must be int".to_string()), stmt.span.clone())),
                };

                let stop_int = match stop {
                    Some(expr) => match self.evaluate_expression(expr)? {
                        Value::Int(i) => i,
                        _ => return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("range stop must be int".to_string()), expr.span.clone())),
                    },
                    None => return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("range stop must be int".to_string()), stmt.span.clone())),
                };

                let step_value = match step {
                    Some(step_expr) => self.evaluate_expression(step_expr)?,
                    None => Value::Int(1)
                };

                let step_span = step.as_ref().map(|e| e.span.clone()).unwrap_or_else(|| stmt.span.clone());
                let step_int = match step_value {
                    Value::Int(i) => i,
                    _ => return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("range step must be int".to_string()), step_span)),
                };

                self.push_scope();
                self.define(var.clone(), Value::Int(start_int), false)?;

                loop {
                    let current_int = match self.lookup(var).cloned() {
                        Some(Value::Int(i)) => i,
                        Some(_) => {
                            self.pop_scope();
                            return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("range var must be int".to_string()), stmt.span.clone()));
                        }
                        None => {
                            self.pop_scope();
                            return Err(RuntimeError::with_span(RuntimeErrorKind::NullPointer, stmt.span.clone()));
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
                    _ => return Err(RuntimeError::new(RuntimeErrorKind::TypeMismatch("for iter must be an array".to_string()))),
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

    fn call_function(&mut self, name: &str, args: &Vec<Expr>, span: Span) -> Result<Value, RuntimeError> {
        let func = self.functions.get(name)
            .cloned()
            .ok_or(RuntimeError::with_span(RuntimeErrorKind::UndefinedFunction(name.to_string()), span.clone()))?;

        let mut evaluated_args = vec![];
        for x in args {
            evaluated_args.push(self.evaluate_expression(x)?);
        }

        match func {
            Function::BuiltIn(f) => f.call(evaluated_args),
            Function::UserDefined { params, return_type: _, body } => {
                if args.len() != params.len() {
                    return Err(RuntimeError::with_span(RuntimeErrorKind::InvalidArgCount(params.len(), args.len()), span));
                }

                self.push_scope();
                self.call_depth += 1;

                if self.call_depth > 512 {
                    self.call_depth -= 1;
                    return Err(RuntimeError::with_span(RuntimeErrorKind::RecursionLimit, span));
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
                        Err(RuntimeError::with_span(RuntimeErrorKind::InvalidControlFlow, span))
                    }
                }
            },
            Function::Extern => {
                println!("Warning: extern function '{}' called, returning void", name);
                Ok(Value::Void)
            }
            Function::Gate { .. } | Function::BuiltInGate { .. } => {
                Err(RuntimeError::with_span(RuntimeErrorKind::InvalidCall(
                    format!("'{}' is a gate and cannot be called as a classical function", name)
                ), span))
            }
        }
    }


    fn interpret_io_decl(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let StmtKind::IoDecl { direction, ty, name, name_span: _ } = &stmt.kind else {
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
                    None => Err(RuntimeError::new(RuntimeErrorKind::UndefinedVariable(name.to_string()))),
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
            .map_err(|_| RuntimeError::new(RuntimeErrorKind::FileNotFound(path.clone())))?;

        self.interpret_include_from_src(&source)
    }

    fn interpret_include_from_src(&mut self, source: &String) -> Result<ControlFlow, RuntimeError> {
        let mut lexer = Lexer::new(source.clone());
        lexer.start();

        let mut parser = Parser::new(lexer.tokens);
        let parse_result = parser.start(false);

        if !parse_result.errors.is_empty() {
            return Err(RuntimeError::new(RuntimeErrorKind::ParseError(parse_result.errors[0].clone())));
        }

        self.interpret_statements(&parse_result.program.statements)
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
        let StmtKind::QuantumDecl { name, name_span: _, size } = &stmt.kind else {
            unreachable!();
        };

        let count = match size {
            Some(expr) => {
                let evaluated_size = self.evaluate_expression(expr)?;

                match evaluated_size {
                    Value::Int(size) => size as usize,
                    _ => return Err(RuntimeError::with_span(RuntimeErrorKind::InvalidSize, expr.span.clone()))
                }
            },
            None => 1,
            _ => return Err(RuntimeError::with_span(RuntimeErrorKind::InvalidSize, stmt.span.clone())),
        };

        self.allocate_qubits(name, count);
        Ok(ControlFlow::None)
    }

    fn resolve_qubits(&mut self, operand: &GateOperand, span: Span) -> Result<Vec<usize>, RuntimeError> {
        match operand {
            GateOperand::Ident(ident) => {
                let indices: Vec<usize> = self.qubit_map
                    .get(&ident.name)
                    .ok_or_else(|| RuntimeError::with_span(RuntimeErrorKind::UndefinedVariable(ident.name.clone()), span.clone()))?
                    .clone();

                if ident.indices.is_empty() {
                    Ok(indices.clone())
                } else {
                    let index_span = if !ident.indices.is_empty() { ident.indices[0].span.clone() } else { span.clone() };
                    let index = match &ident.indices[0].kind {
                        ExprKind::Int(i) => *i as usize,
                        _ => match self.evaluate_expression(&ident.indices[0])? {
                            Value::Int(i) => i as usize,
                            _ => return Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("qubit index must be int".to_string()), index_span.clone())),
                        },
                    };
                    let qubit = indices.get(index)
                        .copied()
                        .ok_or(RuntimeError::with_span(RuntimeErrorKind::IndexOutOfBounds(index, indices.len()), index_span))?;

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
        let StmtKind::GateCall { name, modifiers, params, qubits } = &stmt.kind else {
            unreachable!();
        };

        let param_values: Vec<f64> = params.iter()
            .map(|p| {
                match self.evaluate_expression(p) {
                    Ok(Value::Float(f)) => Ok(f),
                    Ok(Value::Int(i)) => Ok(i as f64),
                    Ok(_) => Err(RuntimeError::with_span(RuntimeErrorKind::TypeMismatch("gate param must be numeric".to_string()), p.span.clone())),
                    Err(e) => Err(e),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let qubit_groups: Vec<Vec<usize>> = qubits.iter()
            .map(|q| self.resolve_qubits(q, stmt.span.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let func = self.functions.get(name)
            .ok_or(RuntimeError::with_span(RuntimeErrorKind::UndefinedFunction(name.clone()), stmt.span.clone()))?
            .clone();

        let expected_param_count = func.get_params().len();
        if param_values.len() != expected_param_count {
            return Err(RuntimeError::with_span(RuntimeErrorKind::InvalidArgCount(expected_param_count, param_values.len()), stmt.span.clone()));
        }

        if name == "gphase" { return Ok(ControlFlow::None); }

        if let Function::UserDefined { .. } = &func {
            let mut args: Vec<Expr> = params.clone();
            for qubit in qubits {
                match qubit {
                    GateOperand::Ident(ident) => args.push(Expr::new(ExprKind::IndexedIdent(ident.clone()), crate::lexer::Span { line: 0, col: 0, len: 0, file: None })),
                    GateOperand::HardwareQubit(_) => {}
                }
            }
            self.call_function(name, &args, stmt.span.clone())?;
            return Ok(ControlFlow::None);
        }

        let resolved = self.resolve_gate(name, &func, &param_values)?;

        let combinations = cartesian_product(&qubit_groups);
        for qubit_indices in combinations {
            let modified = self.apply_modifiers_to_gate(resolved, modifiers)?;
            let sv = self.state_vector.as_mut().ok_or(RuntimeError::with_span(RuntimeErrorKind::NoStateVector, stmt.span.clone()))?;
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
            _ => Err(RuntimeError::new(RuntimeErrorKind::UndefinedFunction("not a gate".to_string())))
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
            _ => Err(RuntimeError::new(RuntimeErrorKind::UnsupportedOperation("gates with >3 qubits not supported".to_string())))
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
                        _ => return Err(RuntimeError::new(RuntimeErrorKind::TypeMismatch("pow requires numeric".to_string()))),
                    };
                    gate.apply_pow(n)
                }
                GateModifier::Ctrl(count) => {
                    let n = match count {
                        Some(expr) => match self.evaluate_expression(expr)? {
                            Value::Int(i) => i as usize,
                            Value::Float(f) => f as usize,
                            _ => return Err(RuntimeError::new(RuntimeErrorKind::TypeMismatch("ctrl count must be int".to_string()))),
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
                            _ => return Err(RuntimeError::new(RuntimeErrorKind::TypeMismatch("negctrl count must be int".to_string()))),
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
        let StmtKind::Assign { target, value, .. } = &stmt.kind else {
            unreachable!();
        };

        let ExprKind::Measure(operand) = &value.kind else {
            unreachable!();
        };

        let qubit_indices = self.resolve_qubits(operand, stmt.span.clone())?;
        let value = self.measure_qubits(qubit_indices, stmt.span.clone())?;

        if target.indices.is_empty() { self.assign(&target.name, value)?; }
        else { self.assign_indexed(&target.name, &target.indices, value, stmt.span.clone())?; }

        Ok(ControlFlow::None)
    }

    fn measure_qubits(&mut self, qubit_indices: Vec<usize>, span: Span) -> Result<Value, RuntimeError> {
        let mut result_value: u64 = 0;
        let width = qubit_indices.len();

        for (bit_pos, &qubit_idx) in qubit_indices.iter().enumerate() {
            let sv = self.state_vector.as_mut().ok_or(RuntimeError::with_span(RuntimeErrorKind::NoStateVector, span.clone()))?;
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
                _ => Err(RuntimeError::new(RuntimeErrorKind::TypeMismatch("range bound must be int".to_string()))),
            },
            None => Ok(None),
        }
    }

    fn evaluate_range(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        let ExprKind::Range { start, stop, step } = &expr.kind else {
            unreachable!();
        };
        Ok(Value::Range {
            start: self.evaluate_int_bound(start)?,
            stop: self.evaluate_int_bound(stop)?,
            step: self.evaluate_int_bound(step)?,
        })
    }

    fn interpret_let(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        let StmtKind::Let { name, name_span: _, value} = &stmt.kind else {
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