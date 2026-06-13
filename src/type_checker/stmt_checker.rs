use crate::parser::statement::Stmt;
use {ForIter, GateOperand, Param};
use crate::parser::expression::Expr;
use crate::parser::supporting_types::*;
use crate::type_checker::type_env::{FunctionSignature, GateSignature};
use crate::type_checker::static_error::StaticError;
use crate::type_checker::type_repr::Type;
use crate::type_checker::TypeChecker;
use crate::lexer::Lexer;
use crate::parser::Parser;

pub fn check_statement(checker: &mut TypeChecker, stmt: &Stmt) -> Result<(), StaticError> {
    match stmt {
        Stmt::QuantumDecl { name, size } => {
            check_quantum_decl(checker, name, size)
        }

        Stmt::ClassicalDecl { ty, name, init } => {
            check_classical_decl(checker, ty, name, init)
        }

        Stmt::ArrayDecl { ty, name, size, init } => {
            check_array_decl(checker, ty, name, size, init)
        }

        Stmt::ConstDecl { ty, name, init } => {
            check_const_decl(checker, ty, name, init)
        }

        Stmt::IoDecl { direction: _, ty, name } => {
            let var_type = Type::from_classical_type(ty);
            checker.env_mut().define(name.clone(), var_type, false)?;
            Ok(())
        }

        Stmt::Assign { target, op: _, value } => {
            check_assignment(checker, target, value)
        }

        Stmt::Let { name, value } => {
            check_let(checker, name, value)
        }

        Stmt::If { cond, then, else_ } => {
            check_if(checker, cond, then, else_)
        }

        Stmt::Switch { expr, cases } => {
            check_switch(checker, expr, cases)
        }

        Stmt::For { var, ty, iter, body } => {
            check_for(checker, var, ty, iter, body)
        }

        Stmt::While { cond, body } => {
            check_while(checker, cond, body)
        }

        Stmt::Continue | Stmt::Break => {
            Ok(())
        }

        Stmt::Return(expr) => {
            check_return(checker, expr)
        }

        Stmt::Def { name, params, return_type, body } => {
            check_def(checker, name, params, return_type, body)
        }

        Stmt::GateDef { name, params, qubits, body } => {
            check_gate_def(checker, name, params, qubits, body)
        }

        Stmt::GateCall { name, modifiers: _, params, qubits } => {
            check_gate_call(checker, name, params, qubits)
        }

        Stmt::ExpressionStatement(expr) => {
            checker.check_expression(expr)?;
            Ok(())
        }

        Stmt::Reset { qubit } => {
            check_qubit_operand(checker, qubit)
        }

        Stmt::Barrier { qubits } => {
            for qubit in qubits {
                check_qubit_operand(checker, qubit)?;
            }
            Ok(())
        }

        Stmt::Block(stmts) => {
            checker.env_mut().push_scope();
            for stmt in stmts {
                check_statement(checker, stmt)?;
            }
            checker.env_mut().pop_scope();
            Ok(())
        }

        Stmt::Include(path) => {
            check_include(checker, path)
        }

        Stmt::IncludeFromSrc(path, content) => {
            check_include_from_src(checker, path, content)
        }

        Stmt::Pragma | Stmt::NoOp | Stmt::Extern { .. } => {
            Ok(())
        }
    }
}

fn check_quantum_decl(checker: &mut TypeChecker, name: &str, size: &Option<Expr>) -> Result<(), StaticError> {
    let qubit_type = match size {
        Some(expr) => {
            let size_type = checker.check_expression(expr)?;
            if !size_type.is_integer() {
                return Err(StaticError::TypeMismatch {
                    expected: Type::Int(None),
                    found: size_type,
                    context: "qubit declaration size".to_string(),
                });
            }

            if let Expr::Int(n) = expr { Type::Qubit(Some(*n)) }
            else { Type::Qubit(None) }
        }
        None => Type::Qubit(None),
    };

    checker.env_mut().define(name.to_string(), qubit_type, false)?;
    Ok(())
}

fn check_classical_decl(checker: &mut TypeChecker, ty: &ClassicalType, name: &str, init: &Option<Expr>) -> Result<(), StaticError> {
    let var_type = Type::from_classical_type(ty);

    if let Some(init_expr) = init {
        let init_type = checker.check_expression(init_expr)?;

        if !init_type.is_compatible_with(&var_type) {
            if checker.config().allow_implicit_casts && init_type.can_coerce_to(&var_type) {
            } else {
                return Err(StaticError::TypeMismatch {
                    expected: var_type,
                    found: init_type,
                    context: format!("variable declaration '{name}'"),
                });
            }
        }

        checker.env_mut().define(name.to_string(), var_type, false)?;
    } else {
        checker.env_mut().define_uninitialized(name.to_string(), var_type, false);
    }

    Ok(())
}

fn check_array_decl(checker: &mut TypeChecker, ty: &ClassicalType, name: &str, size: &[Expr], init: &Option<Expr>) -> Result<(), StaticError> {
    let element_type = Type::from_classical_type(ty);

    let mut dimensions = Vec::new();
    for size_expr in size {
        let size_type = checker.check_expression(size_expr)?;
        if !size_type.is_integer() {
            return Err(StaticError::TypeMismatch {
                expected: Type::Int(None),
                found: size_type,
                context: "array size".to_string(),
            });
        }

        if let Expr::Int(n) = size_expr {
            dimensions.push(Some(*n));
        } else { dimensions.push(None); }
    }

    let array_type = Type::Array {
        element_type: Box::new(element_type.clone()),
        dimensions,
    };

    if let Some(init_expr) = init {
        let init_type = checker.check_expression(init_expr)?;

        if !init_type.is_compatible_with(&array_type) {
            return Err(StaticError::TypeMismatch {
                expected: array_type,
                found: init_type,
                context: format!("array declaration '{name}'"),
            });
        }
    }

    checker.env_mut().define(name.to_string(), array_type, false)?;
    Ok(())
}

fn check_const_decl(checker: &mut TypeChecker, ty: &ClassicalType, name: &str, init: &Expr) -> Result<(), StaticError> {
    let var_type = Type::from_classical_type(ty);
    let init_type = checker.check_expression(init)?;

    if !init_type.is_compatible_with(&var_type) {
        if checker.config().allow_implicit_casts && init_type.can_coerce_to(&var_type) {
        } else {
            return Err(StaticError::TypeMismatch {
                expected: var_type,
                found: init_type,
                context: format!("constant declaration '{name}'"),
            });
        }
    }

    checker.env_mut().define(name.to_string(), var_type, true)?;
    Ok(())
}

fn check_assignment(checker: &mut TypeChecker, target: &IndexedIdent, value: &Expr) -> Result<(), StaticError> {
    if checker.env().is_const(&target.name) {
        return Err(StaticError::AssignmentToConst {
            name: target.name.clone(),
        });
    }

    let target_type = if target.indices.is_empty() {
        checker.check_identifier(&target.name)?
    } else { checker.check_indexed_ident(target)? };

    let value_type = checker.check_expression(value)?;

    if !value_type.is_compatible_with(&target_type) {
        if checker.config().allow_implicit_casts && value_type.can_coerce_to(&target_type) {
        } else {
            return Err(StaticError::TypeMismatch {
                expected: target_type,
                found: value_type,
                context: format!("assignment to '{}'", target.name),
            });
        }
    }

    Ok(())
}

fn check_let(checker: &mut TypeChecker, name: &str, value: &Expr) -> Result<(), StaticError> {
    let value_type = checker.check_expression(value)?;
    checker.env_mut().define(name.to_string(), value_type, false)?;
    Ok(())
}

fn check_if(checker: &mut TypeChecker, cond: &Expr, then: &[Stmt], else_: &Option<Vec<Stmt>>) -> Result<(), StaticError> {
    let cond_type = checker.check_expression(cond)?;

    if !cond_type.can_coerce_to(&Type::Bool) {
        return Err(StaticError::NonBooleanCondition { found: cond_type });
    }

    checker.env_mut().push_scope();
    for stmt in then {
        check_statement(checker, stmt)?;
    }
    checker.env_mut().pop_scope();

    if let Some(else_stmts) = else_ {
        checker.env_mut().push_scope();
        for stmt in else_stmts {
            check_statement(checker, stmt)?;
        }
        checker.env_mut().pop_scope();
    }

    Ok(())
}

fn check_switch(checker: &mut TypeChecker, expr: &Expr, cases: &[SwitchCase]) -> Result<(), StaticError> {
    let switch_type = checker.check_expression(expr)?;

    for case in cases {
        for value in &case.values {
            let value_type = checker.check_expression(value)?;
            if !value_type.is_compatible_with(&switch_type) {
                return Err(StaticError::TypeMismatch {
                    expected: switch_type,
                    found: value_type,
                    context: "switch case".to_string(),
                });
            }
        }

        checker.env_mut().push_scope();
        for stmt in &case.body {
            check_statement(checker, stmt)?;
        }
        checker.env_mut().pop_scope();
    }

    Ok(())
}

fn check_for(checker: &mut TypeChecker, var: &str, ty: &ClassicalType, iter: &ForIter, body: &[Stmt]) -> Result<(), StaticError> {
    let var_type = Type::from_classical_type(ty);

    match iter {
        ForIter::Range { start, stop, step } => {
            if let Some(start_expr) = start {
                let start_type = checker.check_expression(start_expr)?;
                if !start_type.is_integer() {
                    return Err(StaticError::TypeMismatch {
                        expected: Type::Int(None),
                        found: start_type,
                        context: "for loop range start".to_string(),
                    });
                }
            }

            if let Some(stop_expr) = stop {
                let stop_type = checker.check_expression(stop_expr)?;
                if !stop_type.is_integer() {
                    return Err(StaticError::TypeMismatch {
                        expected: Type::Int(None),
                        found: stop_type,
                        context: "for loop range stop".to_string(),
                    });
                }
            }

            if let Some(step_expr) = step {
                let step_type = checker.check_expression(step_expr)?;
                if !step_type.is_integer() {
                    return Err(StaticError::TypeMismatch {
                        expected: Type::Int(None),
                        found: step_type,
                        context: "for loop range step".to_string(),
                    });
                }
            }

            if !var_type.is_integer() {
                return Err(StaticError::TypeMismatch {
                    expected: Type::Int(None),
                    found: var_type,
                    context: "for loop variable".to_string(),
                });
            }
        }

        ForIter::Set(exprs) => {
            for expr in exprs {
                let expr_type = checker.check_expression(expr)?;
                if !expr_type.is_compatible_with(&var_type) {
                    return Err(StaticError::TypeMismatch {
                        expected: var_type,
                        found: expr_type,
                        context: "for loop set element".to_string(),
                    });
                }
            }
        }

        ForIter::Expr(expr) => {
            let expr_type = checker.check_expression(expr)?;

            match expr_type {
                Type::Array { element_type, .. } => {
                    if !element_type.is_compatible_with(&var_type) {
                        return Err(StaticError::TypeMismatch {
                            expected: var_type,
                            found: *element_type,
                            context: "for loop array element".to_string(),
                        });
                    }
                }
                Type::Range => {
                    if !var_type.is_integer() {
                        return Err(StaticError::TypeMismatch {
                            expected: Type::Int(None),
                            found: var_type,
                            context: "for loop variable (range iteration)".to_string(),
                        });
                    }
                }
                _ => {
                    return Err(StaticError::Other {
                        message: format!("Cannot iterate over type {}", expr_type.display_name()),
                    });
                }
            }
        }
    }

    checker.env_mut().push_scope();
    checker.env_mut().define(var.to_string(), var_type, false)?;
    for stmt in body {
        check_statement(checker, stmt)?;
    }
    checker.env_mut().pop_scope();

    Ok(())
}

fn check_while(checker: &mut TypeChecker, cond: &Expr, body: &[Stmt]) -> Result<(), StaticError> {
    let cond_type = checker.check_expression(cond)?;

    if !cond_type.can_coerce_to(&Type::Bool) {
        return Err(StaticError::NonBooleanCondition { found: cond_type });
    }

    checker.env_mut().push_scope();
    for stmt in body {
        check_statement(checker, stmt)?;
    }
    checker.env_mut().pop_scope();

    Ok(())
}

fn check_return(checker: &mut TypeChecker, expr: &Option<Expr>) -> Result<(), StaticError> {
    let func_context = checker.current_function()
        .ok_or_else(|| StaticError::Other {
            message: "Return statement outside of function".to_string(),
        })?;

    let expected_type = func_context.return_type.clone();

    match (expr, &expected_type) {
        (None, Type::Void) => Ok(()),
        (None, _) => Err(StaticError::ReturnTypeMismatch {
            expected: expected_type,
            found: Type::Void,
        }),
        (Some(return_expr), Type::Void) => {
            Err(StaticError::ReturnTypeMismatch {
                expected: Type::Void,
                found: checker.check_expression(return_expr)?,
            })
        }
        (Some(return_expr), _) => {
            let return_type = checker.check_expression(return_expr)?;

            if !return_type.is_compatible_with(&expected_type) {
                if checker.config().allow_implicit_casts && return_type.can_coerce_to(&expected_type) {
                    Ok(())
                } else {
                    Err(StaticError::ReturnTypeMismatch {
                        expected: expected_type,
                        found: return_type,
                    })
                }
            } else { Ok(()) }
        }
    }
}

pub fn check_def(checker: &mut TypeChecker, name: &str, params: &[Param], return_type: &Option<ClassicalType>, body: &[Stmt]) -> Result<(), StaticError> {
    let param_types: Vec<Type> = params.iter()
        .map(|p| Type::from_param_type(&p.ty))
        .collect();

    let ret_type = return_type.as_ref()
        .map(Type::from_classical_type)
        .unwrap_or(Type::Void);

    let signature = FunctionSignature {
        name: name.to_string(),
        params: param_types.clone(),
        return_type: ret_type.clone(),
    };
    checker.env_mut().register_function(signature);
    checker.set_function_context(name.to_string(), ret_type.clone());

    checker.env_mut().push_scope();
    for (param, param_type) in params.iter().zip(param_types.iter()) {
        checker.env_mut().define(param.name.clone(), param_type.clone(), false)?;
    }

    for stmt in body { check_statement(checker, stmt)?; }

    checker.env_mut().pop_scope();
    checker.clear_function_context();

    // TODO: Check that all paths return if return_type is not Void

    Ok(())
}

fn check_gate_def(
    checker: &mut TypeChecker,
    name: &str,
    params: &[String],
    qubits: &[String],
    body: &[Stmt],
) -> Result<(), StaticError> {
    check_gate_def_impl(checker, name, params, qubits, body, true)
}

pub(crate) fn check_gate_def_impl(
    checker: &mut TypeChecker,
    name: &str,
    params: &[String],
    qubits: &[String],
    body: &[Stmt],
    check_body: bool,
) -> Result<(), StaticError> {
    let signature = GateSignature {
        name: name.to_string(),
        params: params.to_vec(),
        qubits: qubits.to_vec(),
    };
    checker.env_mut().register_gate(signature);
    
    if !check_body { return Ok(()); }

    checker.env_mut().push_scope();

    for param in params {
        checker.env_mut().define(param.clone(), Type::Angle(None), false)?;
    }

    for qubit in qubits {
        checker.env_mut().define(qubit.clone(), Type::Qubit(None), false)?;
    }

    for stmt in body {
        check_statement(checker, stmt)?;
    }

    checker.env_mut().pop_scope();

    Ok(())
}

//TODO: Allow this to support functions with gate-compatible signatures
fn check_gate_call(checker: &mut TypeChecker, name: &str, params: &[Expr], qubits: &[GateOperand]) -> Result<(), StaticError> {
    let gate_sig = checker.env().get_gate(name)
        .ok_or_else(|| StaticError::UndefinedGate {
            name: name.to_string(),
        })?;

    if gate_sig.params.len() != params.len() {
        return Err(StaticError::ArityMismatch {
            name: name.to_string(),
            expected: gate_sig.params.len(),
            found: params.len(),
        });
    }

    let expected_qubit_count = gate_sig.qubits.len();

    if expected_qubit_count == 0 {
        if !qubits.is_empty() {
            return Err(StaticError::ArityMismatch {
                name: format!("{name} (qubits)"),
                expected: 0,
                found: qubits.len(),
            });
        }
    } else if qubits.is_empty() {
        return Err(StaticError::ArityMismatch {
            name: format!("{name} (qubits)"),
            expected: expected_qubit_count,
            found: 0,
        });
    } else if qubits.len() % expected_qubit_count != 0 {
        return Err(StaticError::Other {
            message: format!(
                "Gate '{}' expects {} qubit(s), but {} were provided. For broadcasting, the number of qubits must be a multiple of {}.",
                name, expected_qubit_count, qubits.len(), expected_qubit_count
            ),
        });
    }

    for param in params {
        let param_type = checker.check_expression(param)?;
        if !param_type.is_numeric() {
            return Err(StaticError::TypeMismatch {
                expected: Type::Angle(None),
                found: param_type,
                context: format!("gate call '{name}' parameter"),
            });
        }
    }

    for qubit in qubits {
        check_qubit_operand(checker, qubit)?;
    }

    Ok(())
}

fn check_qubit_operand(checker: &mut TypeChecker, operand: &GateOperand) -> Result<(), StaticError> {
    match operand {
        GateOperand::Ident(indexed) => {
            let qubit_type = checker.check_indexed_ident(indexed)?;
            if !matches!(qubit_type, Type::Qubit(_)) {
                return Err(StaticError::TypeMismatch {
                    expected: Type::Qubit(None),
                    found: qubit_type,
                    context: "gate operand".to_string(),
                });
            }
            Ok(())
        }
        GateOperand::HardwareQubit(_) => Ok(())
    }
}

fn check_include(checker: &mut TypeChecker, path: &str) -> Result<(), StaticError> {
    let content = if path == "stdgates.inc" {
        include_str!("../lib/stdgates.inc").to_string()
    } else {
        std::fs::read_to_string(path).map_err(|_| StaticError::Other {
            message: format!("Failed to read include file: {path}"),
        })?
    };

    check_include_from_src(checker, &path.to_string(), &content)
}

fn check_include_from_src(checker: &mut TypeChecker, path: &String, content: &String) -> Result<(), StaticError> {
    let mut lexer = Lexer::new(content.to_string());
    lexer.start();

    let mut parser = Parser::new(lexer.tokens);
    let program = parser.start(false).map_err(|e| {
        StaticError::Other {
            message: format!("Failed to parse include file {path}: {e:?}"),
        }
    })?;

    for stmt in &program.statements {
        match stmt {
            Stmt::GateDef { name, params, qubits, body } => {
                check_gate_def_impl(checker, name, params, qubits, body, false)?;
            }
            Stmt::Def { name, params, return_type, body } => {
                check_def(checker, name, params, return_type, body)?;
            }
            _ => {}
        }
    }

    Ok(())
}

