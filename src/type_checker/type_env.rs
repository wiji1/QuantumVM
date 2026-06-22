use crate::lexer::Span;
use crate::type_checker::type_repr::Type;
use std::collections::{HashMap, HashSet};
use crate::type_checker::static_error::StaticError;

#[derive(Debug, Clone)]
pub struct TypeBinding {
    pub ty: Type,
    pub is_const: bool,
    pub is_initialized: bool,
    pub definition_span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<Type>,
    pub return_type: Type,
    pub definition_span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct GateSignature {
    pub name: String,
    pub params: Vec<String>,
    pub qubits: Vec<String>,
    pub definition_span: Option<Span>,
}

pub struct TypeEnv {
    scopes: Vec<HashMap<String, TypeBinding>>,
    functions: HashMap<String, FunctionSignature>,
    gates: HashMap<String, GateSignature>,
    constants: HashSet<String>,
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            gates: HashMap::new(),
            constants: HashSet::new(),
        };

        env.register_builtin_functions();
        env.register_builtin_gates();
        env.register_builtin_constants().expect("Failed to register builtin constants");

        env
    }

    fn register_builtin_constants(&mut self) -> Result<(), StaticError> {
        self.define("pi".to_string(), Type::Float(None), true)?;
        self.define("π".to_string(), Type::Float(None), true)?;

        self.define("tau".to_string(), Type::Float(None), true)?;
        self.define("τ".to_string(), Type::Float(None), true)?;

        self.define("euler".to_string(), Type::Float(None), true)?;
        self.define("ℯ".to_string(), Type::Float(None), true)?;

        self.define("im".to_string(), Type::Complex(Box::new(Type::Float(None))), true)?;
        self.define("ⅈ".to_string(), Type::Complex(Box::new(Type::Float(None))), true)?;

        Ok(())
    }

    fn register_builtin_functions(&mut self) {
        let float_fn = |name: &str| FunctionSignature {
            name: name.to_string(),
            params: vec![Type::Float(None)],
            return_type: Type::Float(None),
            definition_span: None,
        };

        self.functions.insert("sin".to_string(), float_fn("sin"));
        self.functions.insert("cos".to_string(), float_fn("cos"));
        self.functions.insert("tan".to_string(), float_fn("tan"));
        self.functions.insert("exp".to_string(), float_fn("exp"));
        self.functions.insert("ln".to_string(), float_fn("ln"));
        self.functions.insert("sqrt".to_string(), float_fn("sqrt"));

        self.functions.insert("arcsin".to_string(), float_fn("arcsin"));
        self.functions.insert("arccos".to_string(), float_fn("arccos"));
        self.functions.insert("arctan".to_string(), float_fn("arctan"));

        self.functions.insert("pow".to_string(), FunctionSignature {
            name: "pow".to_string(),
            params: vec![Type::Float(None), Type::Float(None)],
            return_type: Type::Float(None),
            definition_span: None,
        });

        self.functions.insert("mod".to_string(), FunctionSignature {
            name: "mod".to_string(),
            params: vec![Type::Int(None), Type::Int(None)],
            return_type: Type::Int(None),
            definition_span: None,
        });

        self.functions.insert("popcount".to_string(), FunctionSignature {
            name: "popcount".to_string(),
            params: vec![Type::Bit(None)],
            return_type: Type::Int(None),
            definition_span: None,
        });

        self.functions.insert("sizeof".to_string(), FunctionSignature {
            name: "sizeof".to_string(),
            params: vec![Type::Unknown],
            return_type: Type::Int(None),
            definition_span: None,
        });
    }

    fn register_builtin_gates(&mut self) {
        self.gates.insert("U".to_string(), GateSignature {
            name: "U".to_string(),
            params: vec!["theta".to_string(), "phi".to_string(), "lambda".to_string()],
            qubits: vec!["q".to_string()],
            definition_span: None,
        });

        self.gates.insert("gphase".to_string(), GateSignature {
            name: "gphase".to_string(),
            params: vec!["theta".to_string()],
            qubits: vec![],
            definition_span: None,
        });
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 { self.scopes.pop(); }
    }

    pub fn define(&mut self, name: String, ty: Type, is_const: bool) -> Result<(), StaticError> {
        self.define_with_span(name, ty, is_const, None)
    }

    pub fn define_with_span(&mut self, name: String, ty: Type, is_const: bool, span: Option<Span>) -> Result<(), StaticError> {
        let binding = TypeBinding {
            ty,
            is_const,
            is_initialized: true,
            definition_span: span,
        };

        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name) {
                return Err(StaticError::DuplicateVariableDefinition { name })
            }

            scope.insert(name.clone(), binding);
            if is_const { self.constants.insert(name); }
        }

        Ok(())
    }

    pub fn define_uninitialized(&mut self, name: String, ty: Type, is_const: bool) {
        self.define_uninitialized_with_span(name, ty, is_const, None)
    }

    pub fn define_uninitialized_with_span(&mut self, name: String, ty: Type, is_const: bool, span: Option<Span>) {
        let binding = TypeBinding {
            ty,
            is_const,
            is_initialized: false,
            definition_span: span,
        };

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, binding);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&TypeBinding> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.get(name) { return Some(binding); }
        }
        None
    }

    pub fn get_type(&self, name: &str) -> Option<Type> {
        self.lookup(name).map(|binding| binding.ty.clone())
    }

    pub fn is_const(&self, name: &str) -> bool {
        self.constants.contains(name) ||
        self.lookup(name).map_or(false, |b| b.is_const)
    }

    pub fn is_initialized(&self, name: &str) -> bool {
        self.lookup(name).map_or(false, |b| b.is_initialized)
    }

    pub fn mark_initialized(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                binding.is_initialized = true;
                return;
            }
        }
    }

    pub fn register_function(&mut self, sig: FunctionSignature) -> Result<(), StaticError> {
        if self.functions.contains_key(&sig.name) {
            return Err(StaticError::DuplicateFunctionDefinition { name: sig.name.clone() });
        }
        self.functions.insert(sig.name.clone(), sig);
        Ok(())
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }

    pub fn register_gate(&mut self, sig: GateSignature) -> Result<(), StaticError> {
        if self.gates.contains_key(&sig.name) {
            return Err(StaticError::DuplicateGateDefinition { name: sig.name.clone() });
        }
        self.gates.insert(sig.name.clone(), sig);
        Ok(())
    }

    pub fn get_gate(&self, name: &str) -> Option<&GateSignature> {
        if let Some(gate) = self.gates.get(name) { return Some(gate); }

        let name_upper = name.to_uppercase();
        self.gates.get(&name_upper)
    }

    pub fn scope_depth(&self) -> usize {
        self.scopes.len()
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}
