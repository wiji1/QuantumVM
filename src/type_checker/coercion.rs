use crate::type_checker::type_repr::Type;

pub fn unify_types(t1: &Type, t2: &Type) -> Option<Type> {
    use Type::*;

    if t1.is_compatible_with(t2) { return Some(t1.clone()); }

    if t1.can_coerce_to(t2) { return Some(t2.clone()); }

    if t2.can_coerce_to(t1) { return Some(t1.clone()); }

    match (t1, t2) {
        (Complex(_), _) | (_, Complex(_)) => {
            if t1.is_numeric() && t2.is_numeric() {
                Some(Complex(Box::new(Float(None))))
            } else { None }
        }

        (Float(_), Int(_)) | (Int(_), Float(_)) |
        (Float(_), UInt(_)) | (UInt(_), Float(_)) => {
            Some(Float(None))
        }

        (Int(_), UInt(_)) | (UInt(_), Int(_)) => {
            Some(Int(None))
        }

        (Bool, Int(_)) | (Int(_), Bool) => {
            Some(Int(None))
        }

        (Bool, Float(_)) | (Float(_), Bool) => {
            Some(Float(None))
        }

        (Bit(_), Int(_)) | (Int(_), Bit(_)) => {
            Some(Int(None))
        }

        (Angle(_), Float(_)) | (Float(_), Angle(_)) => {
            Some(Float(None))
        }

        _ => None,
    }
}

pub fn coerce_binary_operands(lhs: &Type, rhs: &Type) -> Option<Type> {
    unify_types(lhs, rhs)
}

