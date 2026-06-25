use crate::parser::expression::{Expr, ExprKind};
use crate::parser::supporting_types::IndexedIdent;
use crate::type_checker::coercion::coerce_binary_operands;
use crate::type_checker::reference_registry::ReferenceType;
use crate::type_checker::static_error::StaticError;
use crate::type_checker::type_repr::Type;
use crate::type_checker::TypeChecker;
use Type::*;

impl TypeChecker {
    pub fn check_expression(&mut self, expr: &Expr) -> Result<Type, StaticError> {
        match &expr.kind {
            ExprKind::Int(_) => Ok(Int(None)),
            ExprKind::Float(_) => Ok(Float(None)),
            ExprKind::Bool(_) => Ok(Bool),
            ExprKind::Imaginary(_) => Ok(Complex(Box::new(Float(None)))),
            ExprKind::Bits(_, width) => Ok(Bit(Some(*width as i64))),

            ExprKind::Timing(_) => Ok(Duration),

            ExprKind::Array(elements) => { self.check_array_expr(elements) }

            ExprKind::Ident(name) => {
                self.reference_registry_mut().add_reference(
                    name.clone(),
                    expr.span.clone(),
                    ReferenceType::VariableRead
                );
                self.check_identifier(name)
            }

            ExprKind::IndexedIdent(indexed) => {
                self.reference_registry_mut().add_reference(
                    indexed.name.clone(),
                    expr.span.clone(),
                    ReferenceType::VariableRead
                );
                self.check_indexed_ident(indexed)
            }

            ExprKind::Measure(operand) => { self.check_measure(operand) }

            ExprKind::Unary { op, expr } => { self.check_unary(op, expr) }

            ExprKind::Binary { op, lhs, rhs } => {
                self.check_binary(op, lhs, rhs)
            }

            ExprKind::Call { name, name_span, args } => {
                self.reference_registry_mut().add_reference(
                    name.clone(),
                    name_span.clone(),
                    ReferenceType::FunctionCall
                );
                self.check_call(name, args)
            }

            ExprKind::Cast { ty, expr } => {
                self.check_cast(ty, expr)
            }

            ExprKind::Range { start, stop, step } => {
                self.check_range(start, stop, step)
            }
        }
    }

    fn check_array_expr(&mut self, elements: &[Expr]) -> Result<Type, StaticError> {
        if elements.is_empty() {
            return Ok(Array {
                element_type: Box::new(Unknown),
                dimensions: vec![Some(0)],
            });
        }

        let first_type = self.check_expression(&elements[0])?;

        for elem in &elements[1..] {
            let elem_type = self.check_expression(elem)?;
            if !elem_type.is_compatible_with(&first_type) {
                return Err(StaticError::TypeMismatch {
                    expected: first_type.clone(),
                    found: elem_type,
                    context: "array element".to_string(),
                });
            }
        }

        Ok(Array {
            element_type: Box::new(first_type),
            dimensions: vec![Some(elements.len() as i64)],
        })
    }

    pub fn check_identifier(&self, name: &str) -> Result<Type, StaticError> {
        self.env().get_type(name)
            .ok_or_else(|| StaticError::UndefinedVariable {
                name: name.to_string(),
            })
    }

    pub fn check_indexed_ident(&mut self, indexed: &IndexedIdent) -> Result<Type, StaticError> {
        let base_type = self.check_identifier(&indexed.name)?;

        if indexed.indices.is_empty() { return Ok(base_type); }

        let mut is_range = Vec::new();
        for index in &indexed.indices {
            let index_type = self.check_expression(index)?;
            match index_type {
                Range => is_range.push(true),
                ref t if t.is_integer() => is_range.push(false),
                _ => return Err(StaticError::NonIntegerIndex { index_type }),
            }
        }

        match base_type {
            Array { element_type, dimensions } => {
                if indexed.indices.len() > dimensions.len() {
                    return Err(StaticError::DimensionMismatch {
                        expected: dimensions.len(),
                        found: indexed.indices.len(),
                    });
                }

                let mut result_dims = Vec::new();

                for (i, &is_range_idx) in is_range.iter().enumerate() {
                    if is_range_idx {
                        let range_size = self.compute_range_size(&indexed.indices[i], dimensions[i]);
                        result_dims.push(range_size);
                    }
                }

                result_dims.extend_from_slice(&dimensions[indexed.indices.len()..]);

                if result_dims.is_empty() { Ok(*element_type) }
                else { Ok(Array { element_type, dimensions: result_dims, }) }
            }

            Bit(Some(_)) => {
                if indexed.indices.len() == 1 { Ok(Bit(Some(1))) }
                else {
                    Err(StaticError::DimensionMismatch {
                        expected: 1,
                        found: indexed.indices.len(),
                    })
                }
            }

            Qubit(Some(_)) | Qubit(None) => {
                if indexed.indices.len() == 1 { Ok(Qubit(None)) }
                else {
                    Err(StaticError::DimensionMismatch {
                        expected: 1,
                        found: indexed.indices.len(),
                    })
                }
            }

            _ => Err(StaticError::InvalidArrayAccess { type_: base_type }),
        }
    }

    fn compute_range_size(&self, range_expr: &Expr, original_size: Option<i64>) -> Option<i64> {
        if let ExprKind::Range { start, stop, step } = &range_expr.kind {
            let start_val = start.as_ref().and_then(|e| {
                if let ExprKind::Int(n) = e.kind { Some(n) } else { None }
            });
            let stop_val = stop.as_ref().and_then(|e| {
                if let ExprKind::Int(n) = e.kind { Some(n) } else { None }
            });
            let step_val = step.as_ref().and_then(|e| {
                if let ExprKind::Int(n) = e.kind { Some(n) } else { None }
            }).unwrap_or(1);

            match (start_val, stop_val, original_size) {
                (Some(s), Some(e), _) => {
                    let range_len = (e - s + 1).max(0);
                    Some((range_len + step_val - 1) / step_val)
                }
                (None, Some(e), _) => {
                    let range_len = (e + 1).max(0);
                    Some((range_len + step_val - 1) / step_val)
                }
                (Some(s), None, Some(orig)) => {
                    let range_len = (orig - s).max(0);
                    Some((range_len + step_val - 1) / step_val)
                }
                (None, None, Some(orig)) => {
                    Some((orig + step_val - 1) / step_val)
                }
                _ => {
                    None
                }
            }
        } else { original_size }
    }

    fn check_unary(&mut self, op: &crate::parser::supporting_types::UnaryOp, expr: &Expr) -> Result<Type, StaticError> {
        let expr_type = self.check_expression(expr)?;

        expr_type.unary_result_type(op)
            .ok_or_else(|| StaticError::UnaryOpTypeMismatch {
                op: format!("{op:?}"),
                operand: expr_type,
            })
    }

    fn check_binary(&mut self, op: &crate::parser::supporting_types::BinaryOp, lhs: &Expr, rhs: &Expr) -> Result<Type, StaticError> {
        let lhs_type = self.check_expression(lhs)?;
        let rhs_type = self.check_expression(rhs)?;

        if let Some(result_type) = lhs_type.binary_result_type(&rhs_type, op) {
            return Ok(result_type);
        }

        if self.config().allow_implicit_casts {
            if let Some(unified) = coerce_binary_operands(&lhs_type, &rhs_type) {
                if let Some(result_type) = unified.binary_result_type(&unified, op) {
                    return Ok(result_type);
                }
            }
        }

        Err(StaticError::BinaryOpTypeMismatch {
            op: format!("{op:?}"),
            lhs: lhs_type,
            rhs: rhs_type,
        })
    }

    fn check_call(&mut self, name: &str, args: &[Expr]) -> Result<Type, StaticError> {
        let func_sig = self.env().get_function(name)
            .ok_or_else(|| StaticError::UndefinedFunction {
                name: name.to_string(),
            })?.clone();

        if name != "sizeof" && func_sig.params.len() != args.len() {
            return Err(StaticError::ArityMismatch {
                name: name.to_string(),
                expected: func_sig.params.len(),
                found: args.len(),
            });
        }

        for (i, (param_type, arg)) in func_sig.params.iter().zip(args.iter()).enumerate() {
            let arg_type = self.check_expression(arg)?;

            if matches!(param_type, Unknown) { continue; }

            if !arg_type.is_compatible_with(param_type) {
                if self.config().allow_implicit_casts && arg_type.can_coerce_to(param_type) {
                    continue;
                }

                return Err(StaticError::TypeMismatch {
                    expected: param_type.clone(),
                    found: arg_type,
                    context: format!("function call '{name}', argument {}", i + 1),
                });
            }
        }

        Ok(func_sig.return_type.clone())
    }

    fn check_cast(&mut self, target_ty: &crate::parser::supporting_types::ClassicalType, expr: &Expr) -> Result<Type, StaticError> {
        let expr_type = self.check_expression(expr)?;
        let target_type = Type::from_classical_type(target_ty);

        if !is_valid_cast(&expr_type, &target_type) {
            return Err(StaticError::InvalidCast {
                from: expr_type,
                to: target_type,
            });
        }

        Ok(target_type)
    }

    fn check_range(&mut self, start: &Option<Box<Expr>>, stop: &Option<Box<Expr>>, step: &Option<Box<Expr>>) -> Result<Type, StaticError> {
        if let Some(start_expr) = start {
            let start_type = self.check_expression(start_expr)?;
            if !start_type.is_integer() {
                return Err(StaticError::TypeMismatch {
                    expected: Int(None),
                    found: start_type,
                    context: "range start".to_string(),
                });
            }
        }

        if let Some(stop_expr) = stop {
            let stop_type = self.check_expression(stop_expr)?;
            if !stop_type.is_integer() {
                return Err(StaticError::TypeMismatch {
                    expected: Int(None),
                    found: stop_type,
                    context: "range stop".to_string(),
                });
            }
        }

        if let Some(step_expr) = step {
            let step_type = self.check_expression(step_expr)?;
            if !step_type.is_integer() {
                return Err(StaticError::TypeMismatch {
                    expected: Int(None),
                    found: step_type,
                    context: "range step".to_string(),
                });
            }
        }

        Ok(Range)
    }

    fn check_measure(&mut self, operand: &crate::parser::supporting_types::GateOperand) -> Result<Type, StaticError> {
        use crate::parser::supporting_types::GateOperand;

        match operand {
            GateOperand::Ident(indexed) => {
                let qubit_type = self.check_indexed_ident(indexed)?;

                match qubit_type {
                    Qubit(None) => Ok(Bit(Some(1))),
                    Qubit(Some(size)) => Ok(Bit(Some(size))),
                    _ => Err(StaticError::TypeMismatch {
                        expected: Qubit(None),
                        found: qubit_type,
                        context: "measure operand".to_string(),
                    })
                }
            }
            GateOperand::HardwareQubit(_) => { Ok(Bit(Some(1))) }
        }
    }
}

fn is_valid_cast(from: &Type, to: &Type) -> bool {
    if from == to { return true; }

    if from.is_numeric() && to.is_numeric() { return true; }

    if matches!(from, Bool) && matches!(to, Int(_) | UInt(_)) { return true; }
    if matches!(from, Int(_) | UInt(_)) && matches!(to, Bool) { return true; }

    if matches!(from, Bit(_)) && matches!(to, Int(_) | UInt(_)) { return true; }
    if matches!(from, Int(_) | UInt(_)) && matches!(to, Bit(_)) { return true; }

    if matches!(from, Bit(_)) && matches!(to, Bool) { return true; }
    if matches!(from, Bool) && matches!(to, Bit(_)) { return true; }

    if matches!(from, Angle(_)) && matches!(to, Float(_)) { return true; }
    if matches!(from, Float(_)) && matches!(to, Angle(_)) { return true; }

    false
}
