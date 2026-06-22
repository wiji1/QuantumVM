use crate::parser::statement::{Stmt, StmtKind};
use {ForIter, GateOperand, Param};
use crate::parser::expression::{Expr, ExprKind};
use crate::parser::supporting_types::*;
use crate::type_checker::type_env::{FunctionSignature, GateSignature};
use crate::type_checker::static_error::StaticError;
use crate::type_checker::type_repr::Type;
use crate::type_checker::TypeChecker;
use crate::type_checker::diagnostics::Diagnostic;
use crate::type_checker::reference_registry::ReferenceType;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReturnState {
    AlwaysReturns,
    MayReturn,
    NeverReturns,
}

fn check_stmt_with_diagnostics(checker: &mut TypeChecker, stmt: &Stmt) -> Result<(), StaticError> {
    if let Err(e) = check_statement(checker, stmt) {
        let diagnostic = Diagnostic::from_type_error_with_span(e.clone(), stmt.span.clone());
        checker.add_diagnostic(diagnostic);

        if !checker.config().collect_all_errors { return Err(e); }
    }
    Ok(())
}

pub fn check_statement(checker: &mut TypeChecker, stmt: &Stmt) -> Result<(), StaticError> {
    match &stmt.kind {
        StmtKind::QuantumDecl { name, size } => {
            check_quantum_decl(checker, name, size, &stmt.span)
        }

        StmtKind::ClassicalDecl { ty, name, init } => {
            check_classical_decl(checker, ty, name, init, &stmt.span)
        }

        StmtKind::ArrayDecl { ty, name, size, init } => {
            check_array_decl(checker, ty, name, size, init, &stmt.span)
        }

        StmtKind::ConstDecl { ty, name, init } => {
            check_const_decl(checker, ty, name, init, &stmt.span)
        }

        StmtKind::IoDecl { direction: _, ty, name } => {
            let var_type = Type::from_classical_type(ty);
            checker.env_mut().define_with_span(name.clone(), var_type, false, Some(stmt.span.clone()))?;
            Ok(())
        }

        StmtKind::Assign { target, op: _, value } => {
            checker.reference_registry_mut().add_reference(
                target.name.clone(),
                stmt.span.clone(),
                crate::type_checker::reference_registry::ReferenceType::VariableWrite
            );
            check_assignment(checker, target, value)
        }

        StmtKind::Let { name, value } => {
            checker.reference_registry_mut().add_reference(
                name.clone(),
                stmt.span.clone(),
                crate::type_checker::reference_registry::ReferenceType::VariableWrite
            );
            check_let(checker, name, value)
        }

        StmtKind::If { cond, then, else_ } => {
            check_if(checker, cond, then, else_)
        }

        StmtKind::Switch { expr, cases } => {
            check_switch(checker, expr, cases)
        }

        StmtKind::For { var, ty, iter, body } => {
            check_for(checker, var, ty, iter, body)
        }

        StmtKind::While { cond, body } => {
            check_while(checker, cond, body)
        }

        StmtKind::Continue | StmtKind::Break => {
            Ok(())
        }

        StmtKind::Return(expr) => {
            check_return(checker, expr)
        }

        StmtKind::Def { name, params, return_type, body } => {
            check_def(checker, name, params, return_type, body, Some(stmt.span.clone()))
        }

        StmtKind::GateDef { name, params, qubits, body } => {
            check_gate_def(checker, name, params, qubits, body, Some(stmt.span.clone()))
        }

        StmtKind::GateCall { name, modifiers: _, params, qubits } => {
            let name_span = crate::lexer::Span {
                line: stmt.span.line,
                col: stmt.span.col,
                len: name.len(),
            };
            checker.reference_registry_mut().add_reference(
                name.clone(),
                name_span,
                ReferenceType::GateCall
            );
            check_gate_call(checker, name, params, qubits)
        }

        StmtKind::ExpressionStatement(expr) => {
            checker.check_expression(expr)?;
            Ok(())
        }

        StmtKind::Reset { qubit } => {
            check_qubit_operand(checker, qubit)
        }

        StmtKind::Barrier { qubits } => {
            for qubit in qubits {
                check_qubit_operand(checker, qubit)?;
            }
            Ok(())
        }

        StmtKind::Block(stmts) => {
            checker.env_mut().push_scope();
            for stmt in stmts {
                check_stmt_with_diagnostics(checker, stmt)?;
            }
            checker.env_mut().pop_scope();
            Ok(())
        }

        StmtKind::Include(path) => {
            check_include(checker, path)
        }

        StmtKind::IncludeFromSrc(path, content) => {
            check_include_from_src(checker, path, content)
        }

        StmtKind::Pragma | StmtKind::NoOp | StmtKind::Extern { .. } => {
            Ok(())
        }
    }
}

fn check_quantum_decl(checker: &mut TypeChecker, name: &str, size: &Option<Expr>, span: &crate::lexer::Span) -> Result<(), StaticError> {
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

            if let ExprKind::Int(n) = &expr.kind { Type::Qubit(Some(*n)) }
            else { Type::Qubit(None) }
        }
        None => Type::Qubit(None),
    };

    checker.env_mut().define_with_span(name.to_string(), qubit_type, false, Some(span.clone()))?;
    Ok(())
}

fn check_classical_decl(checker: &mut TypeChecker, ty: &ClassicalType, name: &str, init: &Option<Expr>, span: &crate::lexer::Span) -> Result<(), StaticError> {
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

        checker.env_mut().define_with_span(name.to_string(), var_type, false, Some(span.clone()))?;
    } else {
        checker.env_mut().define_uninitialized_with_span(name.to_string(), var_type, false, Some(span.clone()));
    }

    Ok(())
}

fn check_array_decl(checker: &mut TypeChecker, ty: &ClassicalType, name: &str, size: &[Expr], init: &Option<Expr>, span: &crate::lexer::Span) -> Result<(), StaticError> {
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

        if let ExprKind::Int(n) = &size_expr.kind {
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

    checker.env_mut().define_with_span(name.to_string(), array_type, false, Some(span.clone()))?;
    Ok(())
}

fn check_const_decl(checker: &mut TypeChecker, ty: &ClassicalType, name: &str, init: &Expr, span: &crate::lexer::Span) -> Result<(), StaticError> {
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

    checker.env_mut().define_with_span(name.to_string(), var_type, true, Some(span.clone()))?;
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
    checker.env_mut().define(name.to_string(), value_type, true)?;
    Ok(())
}

fn check_if(checker: &mut TypeChecker, cond: &Expr, then: &[Stmt], else_: &Option<Vec<Stmt>>) -> Result<(), StaticError> {
    let cond_type = checker.check_expression(cond)?;

    if !cond_type.can_coerce_to(&Type::Bool) {
        return Err(StaticError::NonBooleanCondition { found: cond_type });
    }

    checker.env_mut().push_scope();
    for stmt in then {
        check_stmt_with_diagnostics(checker, stmt)?;
    }
    checker.env_mut().pop_scope();

    if let Some(else_stmts) = else_ {
        checker.env_mut().push_scope();
        for stmt in else_stmts {
            check_stmt_with_diagnostics(checker, stmt)?;
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
            check_stmt_with_diagnostics(checker, stmt)?;
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
    checker.env_mut().define(var.to_string(), var_type, true)?;
    for stmt in body {
        check_stmt_with_diagnostics(checker, stmt)?;
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
        check_stmt_with_diagnostics(checker, stmt)?;
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

fn analyze_return_paths(stmts: &[Stmt]) -> ReturnState {
    let mut current_state = ReturnState::NeverReturns;

    for stmt in stmts {
        let stmt_state = analyze_stmt_return(stmt);
        current_state = combine_sequential_states(current_state, stmt_state);

        if current_state == ReturnState::AlwaysReturns { return ReturnState::AlwaysReturns; }
    }

    current_state
}

fn analyze_stmt_return(stmt: &Stmt) -> ReturnState {
    match &stmt.kind {
        StmtKind::Return(_) => ReturnState::AlwaysReturns,

        StmtKind::If { then, else_, .. } => {
            let then_state = analyze_return_paths(then);

            if let Some(else_stmts) = else_ {
                let else_state = analyze_return_paths(else_stmts);
                combine_branch_states(then_state, else_state)
            } else {
                if then_state == ReturnState::AlwaysReturns { ReturnState::MayReturn }
                else if then_state == ReturnState::MayReturn { ReturnState::MayReturn }
                else { ReturnState::NeverReturns }
            }
        }

        StmtKind::Block(stmts) => analyze_return_paths(stmts),

        StmtKind::Switch { cases, .. } => {
            if cases.is_empty() { return ReturnState::NeverReturns; }

            let mut all_return = true;
            let mut any_return = false;

            for case in cases {
                let case_state = analyze_return_paths(&case.body);
                match case_state {
                    ReturnState::AlwaysReturns => any_return = true,
                    ReturnState::MayReturn => {
                        any_return = true;
                        all_return = false;
                    }
                    ReturnState::NeverReturns => all_return = false,
                }
            }

            if all_return { ReturnState::MayReturn }
            else if any_return { ReturnState::MayReturn }
            else { ReturnState::NeverReturns }
        }

        StmtKind::For { body, .. } => {
            let body_state = analyze_return_paths(body);
            match body_state {
                ReturnState::AlwaysReturns | ReturnState::MayReturn => ReturnState::MayReturn,
                ReturnState::NeverReturns => ReturnState::NeverReturns,
            }
        }

        StmtKind::While { body, .. } => {
            let body_state = analyze_return_paths(body);
            match body_state {
                ReturnState::AlwaysReturns | ReturnState::MayReturn => ReturnState::MayReturn,
                ReturnState::NeverReturns => ReturnState::NeverReturns,
            }
        }

        _ => ReturnState::NeverReturns,
    }
}

fn combine_sequential_states(first: ReturnState, second: ReturnState) -> ReturnState {
    match (first, second) {
        (ReturnState::AlwaysReturns, _) => ReturnState::AlwaysReturns,
        (ReturnState::MayReturn, ReturnState::AlwaysReturns) => ReturnState::AlwaysReturns,
        (ReturnState::MayReturn, _) => ReturnState::MayReturn,
        (ReturnState::NeverReturns, state) => state,
    }
}

fn combine_branch_states(branch1: ReturnState, branch2: ReturnState) -> ReturnState {
    match (branch1, branch2) {
        (ReturnState::AlwaysReturns, ReturnState::AlwaysReturns) => ReturnState::AlwaysReturns,
        (ReturnState::AlwaysReturns, _) | (_, ReturnState::AlwaysReturns) => ReturnState::MayReturn,
        (ReturnState::MayReturn, _) | (_, ReturnState::MayReturn) => ReturnState::MayReturn,
        (ReturnState::NeverReturns, ReturnState::NeverReturns) => ReturnState::NeverReturns,
    }
}

pub fn check_def(checker: &mut TypeChecker, name: &str, params: &[Param], return_type: &Option<ClassicalType>, body: &[Stmt], span: Option<crate::lexer::Span>) -> Result<(), StaticError> {
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
        definition_span: span,
    };
    checker.env_mut().register_function(signature)?;
    checker.set_function_context(name.to_string(), ret_type.clone());

    checker.env_mut().push_scope();
    for (param, param_type) in params.iter().zip(param_types.iter()) {
        checker.env_mut().define(param.name.clone(), param_type.clone(), false)?;
    }

    for stmt in body {
        check_stmt_with_diagnostics(checker, stmt)?;
    }

    checker.env_mut().pop_scope();
    checker.clear_function_context();

    if ret_type != Type::Void {
        let return_state = analyze_return_paths(body);
        if return_state != ReturnState::AlwaysReturns {
            return Err(StaticError::MissingReturn { function: name.to_string() });
        }
    }

    Ok(())
}

fn check_gate_def(
    checker: &mut TypeChecker,
    name: &str,
    params: &[String],
    qubits: &[String],
    body: &[Stmt],
    span: Option<Span>,
) -> Result<(), StaticError> {
    check_gate_def_impl(checker, name, params, qubits, body, true, span)
}

pub(crate) fn check_gate_def_impl(
    checker: &mut TypeChecker,
    name: &str,
    params: &[String],
    qubits: &[String],
    body: &[Stmt],
    check_body: bool,
    span: Option<Span>,
) -> Result<(), StaticError> {
    let signature = GateSignature {
        name: name.to_string(),
        params: params.to_vec(),
        qubits: qubits.to_vec(),
        definition_span: span,
    };
    checker.env_mut().register_gate(signature)?;
    
    if !check_body { return Ok(()); }

    checker.env_mut().push_scope();

    for param in params {
        checker.env_mut().define(param.clone(), Type::Angle(None), false)?;
    }

    for qubit in qubits {
        checker.env_mut().define(qubit.clone(), Type::Qubit(None), false)?;
    }

    for stmt in body {
        check_stmt_with_diagnostics(checker, stmt)?;
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
    if path == "stdgates.inc" { return Ok(()) }

    let content = std::fs::read_to_string(path).map_err(|_| StaticError::Other {
        message: format!("Failed to read include file: {path}"),
    })?;

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
        match &stmt.kind {
            StmtKind::GateDef { name, params, qubits, body } => {
                check_gate_def_impl(checker, name, params, qubits, body, false, None)?;
            }
            StmtKind::Def { name, params, return_type, body } => {
                check_def(checker, name, params, return_type, body, None)?;
            }
            _ => {}
        }
    }

    Ok(())
}

