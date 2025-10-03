use crate::ast::*;
use crate::error::{Result, VeyraError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// Reference type for implementing borrowing
#[derive(Debug, Clone)]
pub struct Reference {
    pub value: Rc<RefCell<Value>>,
    pub mutable: bool,
}

impl PartialEq for Reference {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Boolean(bool),
    None,
    Array(Vec<Value>),
    Dictionary(HashMap<String, Value>),
    Set(std::collections::HashSet<String>),
    Tuple(Vec<Value>),
    Reference(Reference),
}

impl Value {
    fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Char(_) => "char",
            Value::Boolean(_) => "bool",
            Value::None => "none",
            Value::Array(_) => "array",
            Value::Dictionary(_) => "dictionary",
            Value::Set(_) => "set",
            Value::Tuple(_) => "tuple",
            Value::Reference(r) => {
                if r.mutable {
                    "&mut"
                } else {
                    "&"
                }
            }
        }
    }
}

impl Value {
    fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::None => false,
            Value::Integer(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Char(c) => *c != '\0',
            Value::Array(arr) => !arr.is_empty(),
            Value::Dictionary(map) => !map.is_empty(),
            Value::Set(set) => !set.is_empty(),
            Value::Tuple(tuple) => !tuple.is_empty(),
            Value::Reference(r) => r.value.borrow().is_truthy(),
        }
    }
}

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Global scope
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }

    fn set(&mut self, name: &str, value: Value) -> Result<()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        Err(VeyraError::runtime_error(format!(
            "Undefined variable '{}'",
            name
        )))
    }
}

pub struct Interpreter {
    environment: Environment,
    functions: HashMap<String, Function>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
            functions: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn interpret(&mut self, program: &Program) -> Result<Value> {
        self.interpret_program(program)?;
        Ok(Value::None)
    }

    fn interpret_program(&mut self, program: &Program) -> Result<()> {
        // First pass: collect all function definitions
        for item in &program.items {
            if let Item::Function(func) = item {
                self.functions.insert(func.name.clone(), func.clone());
            }
        }

        // Execute statements and expressions at module level
        for item in &program.items {
            match item {
                Item::Function(_) => {
                    // Already handled in first pass
                }
                Item::Statement(statement) => {
                    // Execute top-level statements
                    self.execute_statement(statement)?;
                }
                Item::Import(import) => {
                    self.handle_import(import)?;
                }
                _ => {
                    // For now, just skip other items like structs, impls, etc.
                }
            }
        }

        // Look for a main function and execute it
        if let Some(main_func) = self.functions.get("main") {
            if main_func.parameters.is_empty() {
                self.call_function("main", &[])?;
            }
        }

        Ok(())
    }

    fn call_function(&mut self, name: &str, args: &[Value]) -> Result<Value> {
        // Handle module functions (e.g., math::abs)
        if name.contains("::") {
            return self.call_module_function(name, args);
        }

        // Built-in functions
        match name {
            "print" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "print() takes exactly one argument",
                    ));
                }
                println!("{}", Self::value_to_string(&args[0]));
                return Ok(Value::None);
            }
            "str" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "str() takes exactly one argument",
                    ));
                }
                return Ok(Value::String(Self::value_to_string(&args[0])));
            }
            "len" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "len() takes exactly one argument",
                    ));
                }
                return match &args[0] {
                    Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
                    Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                    Value::Dictionary(map) => Ok(Value::Integer(map.len() as i64)),
                    Value::Set(set) => Ok(Value::Integer(set.len() as i64)),
                    Value::Tuple(tuple) => Ok(Value::Integer(tuple.len() as i64)),
                    _ => Err(VeyraError::runtime_error("len() can only be called on arrays, strings, dictionaries, sets, and tuples")),
                };
            }
            "push" => {
                if args.len() != 2 {
                    return Err(VeyraError::runtime_error(
                        "push() takes exactly two arguments",
                    ));
                }
                // Note: This is a simplified implementation - real implementation would modify in place
                if let Value::Array(arr) = &args[0] {
                    let mut new_arr = arr.clone();
                    new_arr.push(args[1].clone());
                    return Ok(Value::Array(new_arr));
                }
                return Err(VeyraError::runtime_error(
                    "push() can only be called on arrays",
                ));
            }
            "pop" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "pop() takes exactly one argument",
                    ));
                }
                if let Value::Array(arr) = &args[0] {
                    if arr.is_empty() {
                        return Ok(Value::None);
                    }
                    return Ok(arr[arr.len() - 1].clone());
                }
                return Err(VeyraError::runtime_error(
                    "pop() can only be called on arrays",
                ));
            }
            "type_of" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "type_of() takes exactly one argument",
                    ));
                }
                return Ok(Value::String(args[0].type_name().to_string()));
            }
            "int" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "int() takes exactly one argument",
                    ));
                }
                return self.cast_value(args[0].clone(), &Type::Primitive(PrimitiveType::I64));
            }
            "float" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "float() takes exactly one argument",
                    ));
                }
                return self.cast_value(args[0].clone(), &Type::Primitive(PrimitiveType::F64));
            }
            "bool" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "bool() takes exactly one argument",
                    ));
                }
                return self.cast_value(args[0].clone(), &Type::Primitive(PrimitiveType::Bool));
            }
            "char" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "char() takes exactly one argument",
                    ));
                }
                return self.cast_value(args[0].clone(), &Type::Primitive(PrimitiveType::Char));
            }
            "is_int" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "is_int() takes exactly one argument",
                    ));
                }
                return Ok(Value::Boolean(matches!(args[0], Value::Integer(_))));
            }
            "is_float" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "is_float() takes exactly one argument",
                    ));
                }
                return Ok(Value::Boolean(matches!(args[0], Value::Float(_))));
            }
            "is_string" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "is_string() takes exactly one argument",
                    ));
                }
                return Ok(Value::Boolean(matches!(args[0], Value::String(_))));
            }
            "is_bool" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "is_bool() takes exactly one argument",
                    ));
                }
                return Ok(Value::Boolean(matches!(args[0], Value::Boolean(_))));
            }
            "is_char" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "is_char() takes exactly one argument",
                    ));
                }
                return Ok(Value::Boolean(matches!(args[0], Value::Char(_))));
            }
            "is_array" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "is_array() takes exactly one argument",
                    ));
                }
                return Ok(Value::Boolean(matches!(args[0], Value::Array(_))));
            }
            "is_dict" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "is_dict() takes exactly one argument",
                    ));
                }
                return Ok(Value::Boolean(matches!(args[0], Value::Dictionary(_))));
            }
            "is_none" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "is_none() takes exactly one argument",
                    ));
                }
                return Ok(Value::Boolean(matches!(args[0], Value::None)));
            }
            "sqrt" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "sqrt() takes exactly one argument",
                    ));
                }
                match &args[0] {
                    Value::Float(f) => return Ok(Value::Float(f.sqrt())),
                    Value::Integer(n) => return Ok(Value::Float((*n as f64).sqrt())),
                    _ => return Err(VeyraError::runtime_error("sqrt() requires a number")),
                }
            }
            "pow" => {
                if args.len() != 2 {
                    return Err(VeyraError::runtime_error(
                        "pow() takes exactly two arguments",
                    ));
                }
                return self.apply_binary_operator(&BinaryOperator::Power, &args[0], &args[1]);
            }
            "abs" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "abs() takes exactly one argument",
                    ));
                }
                match &args[0] {
                    Value::Integer(n) => return Ok(Value::Integer(n.abs())),
                    Value::Float(f) => return Ok(Value::Float(f.abs())),
                    _ => return Err(VeyraError::runtime_error("abs() requires a number")),
                }
            }
            "min" => {
                if args.len() != 2 {
                    return Err(VeyraError::runtime_error(
                        "min() takes exactly two arguments",
                    ));
                }
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => return Ok(Value::Integer(*a.min(b))),
                    (Value::Float(a), Value::Float(b)) => return Ok(Value::Float(a.min(*b))),
                    (Value::Integer(a), Value::Float(b)) => {
                        return Ok(Value::Float((*a as f64).min(*b)))
                    }
                    (Value::Float(a), Value::Integer(b)) => {
                        return Ok(Value::Float(a.min(*b as f64)))
                    }
                    _ => return Err(VeyraError::runtime_error("min() requires numbers")),
                }
            }
            "max" => {
                if args.len() != 2 {
                    return Err(VeyraError::runtime_error(
                        "max() takes exactly two arguments",
                    ));
                }
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => return Ok(Value::Integer(*a.max(b))),
                    (Value::Float(a), Value::Float(b)) => return Ok(Value::Float(a.max(*b))),
                    (Value::Integer(a), Value::Float(b)) => {
                        return Ok(Value::Float((*a as f64).max(*b)))
                    }
                    (Value::Float(a), Value::Integer(b)) => {
                        return Ok(Value::Float(a.max(*b as f64)))
                    }
                    _ => return Err(VeyraError::runtime_error("max() requires numbers")),
                }
            }
            "floor" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "floor() takes exactly one argument",
                    ));
                }
                match &args[0] {
                    Value::Float(f) => return Ok(Value::Integer(f.floor() as i64)),
                    Value::Integer(n) => return Ok(Value::Integer(*n)),
                    _ => return Err(VeyraError::runtime_error("floor() requires a number")),
                }
            }
            "ceil" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "ceil() takes exactly one argument",
                    ));
                }
                match &args[0] {
                    Value::Float(f) => return Ok(Value::Integer(f.ceil() as i64)),
                    Value::Integer(n) => return Ok(Value::Integer(*n)),
                    _ => return Err(VeyraError::runtime_error("ceil() requires a number")),
                }
            }
            "round" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "round() takes exactly one argument",
                    ));
                }
                match &args[0] {
                    Value::Float(f) => return Ok(Value::Integer(f.round() as i64)),
                    Value::Integer(n) => return Ok(Value::Integer(*n)),
                    _ => return Err(VeyraError::runtime_error("round() requires a number")),
                }
            }
            "string_to_upper" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "string_to_upper() takes exactly one argument",
                    ));
                }
                match &args[0] {
                    Value::String(s) => return Ok(Value::String(s.to_uppercase())),
                    _ => {
                        return Err(VeyraError::runtime_error(
                            "string_to_upper() requires a string",
                        ))
                    }
                }
            }
            "string_to_lower" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "string_to_lower() takes exactly one argument",
                    ));
                }
                match &args[0] {
                    Value::String(s) => return Ok(Value::String(s.to_lowercase())),
                    _ => {
                        return Err(VeyraError::runtime_error(
                            "string_to_lower() requires a string",
                        ))
                    }
                }
            }
            "string_trim" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "string_trim() takes exactly one argument",
                    ));
                }
                match &args[0] {
                    Value::String(s) => return Ok(Value::String(s.trim().to_string())),
                    _ => return Err(VeyraError::runtime_error("string_trim() requires a string")),
                }
            }
            "clamp" => {
                if args.len() != 3 {
                    return Err(VeyraError::runtime_error(
                        "clamp() takes exactly three arguments: value, min, max",
                    ));
                }
                match (&args[0], &args[1], &args[2]) {
                    (Value::Float(val), Value::Float(min_val), Value::Float(max_val)) => {
                        return Ok(Value::Float(val.max(*min_val).min(*max_val)));
                    }
                    (Value::Integer(val), Value::Integer(min_val), Value::Integer(max_val)) => {
                        return Ok(Value::Integer(*val.max(min_val).min(max_val)));
                    }
                    _ => {
                        return Err(VeyraError::runtime_error(
                            "clamp() requires numeric arguments",
                        ))
                    }
                }
            }
            "array_sum" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "array_sum() takes exactly one argument",
                    ));
                }
                match &args[0] {
                    Value::Array(arr) => {
                        let mut sum_int = 0i64;
                        let mut sum_float = 0.0f64;
                        let mut has_float = false;

                        for val in arr {
                            match val {
                                Value::Integer(n) => {
                                    if has_float {
                                        sum_float += *n as f64;
                                    } else {
                                        sum_int += n;
                                    }
                                }
                                Value::Float(f) => {
                                    if !has_float {
                                        sum_float = sum_int as f64;
                                        has_float = true;
                                    }
                                    sum_float += f;
                                }
                                _ => {
                                    return Err(VeyraError::runtime_error(
                                        "array_sum() requires an array of numbers",
                                    ))
                                }
                            }
                        }

                        if has_float {
                            return Ok(Value::Float(sum_float));
                        } else {
                            return Ok(Value::Integer(sum_int));
                        }
                    }
                    _ => return Err(VeyraError::runtime_error("array_sum() requires an array")),
                }
            }
            "array_avg" => {
                if args.len() != 1 {
                    return Err(VeyraError::runtime_error(
                        "array_avg() takes exactly one argument",
                    ));
                }
                match &args[0] {
                    Value::Array(arr) => {
                        if arr.is_empty() {
                            return Ok(Value::Float(0.0));
                        }

                        let mut sum = 0.0f64;
                        for val in arr {
                            match val {
                                Value::Integer(n) => sum += *n as f64,
                                Value::Float(f) => sum += f,
                                _ => {
                                    return Err(VeyraError::runtime_error(
                                        "array_avg() requires an array of numbers",
                                    ))
                                }
                            }
                        }

                        return Ok(Value::Float(sum / arr.len() as f64));
                    }
                    _ => return Err(VeyraError::runtime_error("array_avg() requires an array")),
                }
            }
            "now" => {
                if !args.is_empty() {
                    return Err(VeyraError::runtime_error("now() takes no arguments"));
                }
                // Return current Unix timestamp in seconds
                use std::time::{SystemTime, UNIX_EPOCH};
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                return Ok(Value::Integer(timestamp as i64));
            }
            "range" => {
                if args.is_empty() || args.len() > 3 {
                    return Err(VeyraError::runtime_error(
                        "range() takes 1, 2, or 3 arguments",
                    ));
                }

                let (start, end, step) = match args.len() {
                    1 => match &args[0] {
                        Value::Integer(n) => (0, *n, 1),
                        _ => {
                            return Err(VeyraError::runtime_error(
                                "range() requires integer arguments",
                            ))
                        }
                    },
                    2 => match (&args[0], &args[1]) {
                        (Value::Integer(s), Value::Integer(e)) => (*s, *e, 1),
                        _ => {
                            return Err(VeyraError::runtime_error(
                                "range() requires integer arguments",
                            ))
                        }
                    },
                    3 => match (&args[0], &args[1], &args[2]) {
                        (Value::Integer(s), Value::Integer(e), Value::Integer(step)) => {
                            (*s, *e, *step)
                        }
                        _ => {
                            return Err(VeyraError::runtime_error(
                                "range() requires integer arguments",
                            ))
                        }
                    },
                    _ => unreachable!(),
                };

                if step == 0 {
                    return Err(VeyraError::runtime_error("range() step cannot be zero"));
                }

                let mut result = Vec::new();
                if step > 0 {
                    let mut i = start;
                    while i < end {
                        result.push(Value::Integer(i));
                        i += step;
                    }
                } else {
                    let mut i = start;
                    while i > end {
                        result.push(Value::Integer(i));
                        i += step;
                    }
                }

                return Ok(Value::Array(result));
            }
            _ => {}
        }

        let function = self
            .functions
            .get(name)
            .ok_or_else(|| VeyraError::runtime_error(format!("Undefined function '{}'", name)))?
            .clone();

        if args.len() != function.parameters.len() {
            return Err(VeyraError::runtime_error(format!(
                "Function '{}' expects {} arguments, got {}",
                name,
                function.parameters.len(),
                args.len()
            )));
        }

        // Create new scope for function
        self.environment.push_scope();

        // Bind parameters
        for (param, arg) in function.parameters.iter().zip(args) {
            self.environment.define(param.name.clone(), arg.clone());
        }

        // Execute function body
        let result = match self.execute_block(&function.body) {
            Ok(_) => Ok(Value::None), // Function completed without return
            Err(VeyraError::RuntimeError { message }) if message.starts_with("return:") => {
                // Parse return value from error message
                let value_part = &message[7..]; // Remove "return:" prefix
                self.parse_return_value(value_part)
            }
            Err(e) => Err(e),
        };

        // Clean up scope
        self.environment.pop_scope();

        result
    }

    fn call_module_function(&mut self, name: &str, _args: &[Value]) -> Result<Value> {
        // Module functions are now implemented in Veyra stdlib files
        // This function is kept for future extensibility but currently unused
        Err(VeyraError::runtime_error(format!(
            "Undefined function '{}'",
            name
        )))
    }

    fn execute_block(&mut self, block: &Block) -> Result<()> {
        for statement in &block.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Expression(expr_stmt) => {
                self.evaluate_expression(&expr_stmt.expression)?;
            }
            Statement::VariableDeclaration(var_decl) => {
                let value = self.evaluate_expression(&var_decl.initializer)?;
                self.environment.define(var_decl.name.clone(), value);
            }
            Statement::Assignment(assignment) => {
                let value = self.evaluate_expression(&assignment.value)?;
                match &assignment.target {
                    Expression::Identifier(name) => match assignment.operator {
                        AssignmentOperator::Assign => {
                            self.environment.set(name, value)?;
                        }
                        AssignmentOperator::AddAssign => {
                            let old_value = self
                                .environment
                                .get(name)
                                .ok_or_else(|| {
                                    VeyraError::runtime_error(format!(
                                        "Undefined variable '{}'",
                                        name
                                    ))
                                })?
                                .clone();
                            let new_value = self.add_values(&old_value, &value)?;
                            self.environment.set(name, new_value)?;
                        }
                        AssignmentOperator::SubAssign => {
                            let old_value = self
                                .environment
                                .get(name)
                                .ok_or_else(|| {
                                    VeyraError::runtime_error(format!(
                                        "Undefined variable '{}'",
                                        name
                                    ))
                                })?
                                .clone();
                            let new_value = self.apply_binary_operator(
                                &BinaryOperator::Subtract,
                                &old_value,
                                &value,
                            )?;
                            self.environment.set(name, new_value)?;
                        }
                        AssignmentOperator::MulAssign => {
                            let old_value = self
                                .environment
                                .get(name)
                                .ok_or_else(|| {
                                    VeyraError::runtime_error(format!(
                                        "Undefined variable '{}'",
                                        name
                                    ))
                                })?
                                .clone();
                            let new_value = self.apply_binary_operator(
                                &BinaryOperator::Multiply,
                                &old_value,
                                &value,
                            )?;
                            self.environment.set(name, new_value)?;
                        }
                        AssignmentOperator::DivAssign => {
                            let old_value = self
                                .environment
                                .get(name)
                                .ok_or_else(|| {
                                    VeyraError::runtime_error(format!(
                                        "Undefined variable '{}'",
                                        name
                                    ))
                                })?
                                .clone();
                            let new_value = self.apply_binary_operator(
                                &BinaryOperator::Divide,
                                &old_value,
                                &value,
                            )?;
                            self.environment.set(name, new_value)?;
                        }
                        AssignmentOperator::ModAssign => {
                            let old_value = self
                                .environment
                                .get(name)
                                .ok_or_else(|| {
                                    VeyraError::runtime_error(format!(
                                        "Undefined variable '{}'",
                                        name
                                    ))
                                })?
                                .clone();
                            let new_value = self.apply_binary_operator(
                                &BinaryOperator::Modulo,
                                &old_value,
                                &value,
                            )?;
                            self.environment.set(name, new_value)?;
                        }
                        AssignmentOperator::BitwiseAndAssign => {
                            let old_value = self
                                .environment
                                .get(name)
                                .ok_or_else(|| {
                                    VeyraError::runtime_error(format!(
                                        "Undefined variable '{}'",
                                        name
                                    ))
                                })?
                                .clone();
                            let new_value = self.apply_binary_operator(
                                &BinaryOperator::BitwiseAnd,
                                &old_value,
                                &value,
                            )?;
                            self.environment.set(name, new_value)?;
                        }
                        AssignmentOperator::BitwiseOrAssign => {
                            let old_value = self
                                .environment
                                .get(name)
                                .ok_or_else(|| {
                                    VeyraError::runtime_error(format!(
                                        "Undefined variable '{}'",
                                        name
                                    ))
                                })?
                                .clone();
                            let new_value = self.apply_binary_operator(
                                &BinaryOperator::BitwiseOr,
                                &old_value,
                                &value,
                            )?;
                            self.environment.set(name, new_value)?;
                        }
                        AssignmentOperator::BitwiseXorAssign => {
                            let old_value = self
                                .environment
                                .get(name)
                                .ok_or_else(|| {
                                    VeyraError::runtime_error(format!(
                                        "Undefined variable '{}'",
                                        name
                                    ))
                                })?
                                .clone();
                            let new_value = self.apply_binary_operator(
                                &BinaryOperator::BitwiseXor,
                                &old_value,
                                &value,
                            )?;
                            self.environment.set(name, new_value)?;
                        }
                        AssignmentOperator::LeftShiftAssign => {
                            let old_value = self
                                .environment
                                .get(name)
                                .ok_or_else(|| {
                                    VeyraError::runtime_error(format!(
                                        "Undefined variable '{}'",
                                        name
                                    ))
                                })?
                                .clone();
                            let new_value = self.apply_binary_operator(
                                &BinaryOperator::LeftShift,
                                &old_value,
                                &value,
                            )?;
                            self.environment.set(name, new_value)?;
                        }
                        AssignmentOperator::RightShiftAssign => {
                            let old_value = self
                                .environment
                                .get(name)
                                .ok_or_else(|| {
                                    VeyraError::runtime_error(format!(
                                        "Undefined variable '{}'",
                                        name
                                    ))
                                })?
                                .clone();
                            let new_value = self.apply_binary_operator(
                                &BinaryOperator::RightShift,
                                &old_value,
                                &value,
                            )?;
                            self.environment.set(name, new_value)?;
                        }
                    },
                    _ => {
                        return Err(VeyraError::runtime_error(
                            "Complex assignment targets not implemented yet",
                        ));
                    }
                }
            }
            Statement::If(if_stmt) => {
                let condition = self.evaluate_expression(&if_stmt.condition)?;
                if condition.is_truthy() {
                    self.execute_block(&if_stmt.then_branch)?;
                } else {
                    // Check elif branches
                    for (elif_condition, elif_body) in &if_stmt.elif_branches {
                        let elif_result = self.evaluate_expression(elif_condition)?;
                        if elif_result.is_truthy() {
                            self.execute_block(elif_body)?;
                            return Ok(());
                        }
                    }

                    // Execute else branch if present
                    if let Some(else_branch) = &if_stmt.else_branch {
                        self.execute_block(else_branch)?;
                    }
                }
            }
            Statement::While(while_stmt) => {
                while self.evaluate_expression(&while_stmt.condition)?.is_truthy() {
                    match self.execute_block(&while_stmt.body) {
                        Ok(()) => {}
                        Err(VeyraError::RuntimeError { message }) if message == "break" => break,
                        Err(VeyraError::RuntimeError { message }) if message == "continue" => {
                            continue
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
            Statement::For(for_stmt) => {
                let iterable = self.evaluate_expression(&for_stmt.iterable)?;
                match iterable {
                    Value::Array(arr) => {
                        for item in arr {
                            self.environment.define(for_stmt.variable.clone(), item);
                            match self.execute_block(&for_stmt.body) {
                                Ok(()) => {}
                                Err(VeyraError::RuntimeError { message }) if message == "break" => {
                                    break
                                }
                                Err(VeyraError::RuntimeError { message })
                                    if message == "continue" =>
                                {
                                    continue
                                }
                                Err(e) => return Err(e),
                            }
                        }
                    }
                    _ => {
                        return Err(VeyraError::runtime_error(
                            "Cannot iterate over non-array value",
                        ))
                    }
                }
            }
            Statement::Return(return_stmt) => {
                let value = if let Some(expr) = &return_stmt.value {
                    self.evaluate_expression(expr)?
                } else {
                    Value::None
                };
                // Use error mechanism to bubble up return value (hack)
                return Err(VeyraError::runtime_error(format!(
                    "return:{}",
                    Self::value_to_string(&value)
                )));
            }
            Statement::Break => {
                return Err(VeyraError::runtime_error("break"));
            }
            Statement::Continue => {
                return Err(VeyraError::runtime_error("continue"));
            }
            Statement::Block(block) => {
                self.environment.push_scope();
                let result = self.execute_block(block);
                self.environment.pop_scope();
                result?;
            }
            Statement::Match(_) => {
                return Err(VeyraError::runtime_error(
                    "match statements not implemented",
                ));
            }
        }
        Ok(())
    }

    fn evaluate_expression(&mut self, expression: &Expression) -> Result<Value> {
        match expression {
            Expression::Literal(literal) => Ok(self.literal_to_value(literal)),
            Expression::Identifier(name) => {
                if name == "self" {
                    return Err(VeyraError::runtime_error(
                        "'self' not supported in this context",
                    ));
                }
                self.environment.get(name).cloned().ok_or_else(|| {
                    VeyraError::runtime_error(format!("Undefined variable '{}'", name))
                })
            }
            Expression::Binary(binary) => {
                let left = self.evaluate_expression(&binary.left)?;
                let right = self.evaluate_expression(&binary.right)?;
                self.apply_binary_operator(&binary.operator, &left, &right)
            }
            Expression::Unary(unary) => {
                let operand = self.evaluate_expression(&unary.operand)?;
                self.apply_unary_operator(&unary.operator, &operand)
            }
            Expression::Call(call) => {
                let mut args = Vec::new();
                for arg_expr in &call.arguments {
                    args.push(self.evaluate_expression(arg_expr)?);
                }

                match call.callee.as_ref() {
                    Expression::Identifier(func_name) => self.call_function(func_name, &args),
                    Expression::ModuleAccess(module_access) => {
                        // For stdlib functions, they're loaded globally, so call by item name
                        self.call_function(&module_access.item, &args)
                    }
                    _ => Err(VeyraError::runtime_error(
                        "Complex function calls not implemented",
                    )),
                }
            }
            Expression::Array(array) => {
                let mut elements = Vec::new();
                for elem_expr in &array.elements {
                    elements.push(self.evaluate_expression(elem_expr)?);
                }
                Ok(Value::Array(elements))
            }
            Expression::Dictionary(dict) => {
                let mut map = HashMap::new();
                for (key_expr, value_expr) in &dict.pairs {
                    let key = self.evaluate_expression(key_expr)?;
                    let value = self.evaluate_expression(value_expr)?;

                    // Convert key to string for now
                    let key_str = match key {
                        Value::String(s) => s,
                        Value::Integer(i) => i.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Boolean(b) => b.to_string(),
                        Value::Char(c) => c.to_string(),
                        _ => return Err(VeyraError::runtime_error(
                            "Dictionary keys must be hashable (string, int, float, bool, or char)",
                        )),
                    };

                    map.insert(key_str, value);
                }
                Ok(Value::Dictionary(map))
            }
            Expression::Set(set) => {
                let mut hash_set = std::collections::HashSet::new();
                for elem_expr in &set.elements {
                    let elem = self.evaluate_expression(elem_expr)?;

                    // Convert element to string for now
                    let elem_str =
                        match elem {
                            Value::String(s) => s,
                            Value::Integer(i) => i.to_string(),
                            Value::Float(f) => f.to_string(),
                            Value::Boolean(b) => b.to_string(),
                            Value::Char(c) => c.to_string(),
                            _ => return Err(VeyraError::runtime_error(
                                "Set elements must be hashable (string, int, float, bool, or char)",
                            )),
                        };

                    hash_set.insert(elem_str);
                }
                Ok(Value::Set(hash_set))
            }
            Expression::Tuple(tuple) => {
                let mut elements = Vec::new();
                for elem_expr in &tuple.elements {
                    elements.push(self.evaluate_expression(elem_expr)?);
                }
                Ok(Value::Tuple(elements))
            }
            Expression::Index(index) => {
                let object = self.evaluate_expression(&index.object)?;
                let index_val = self.evaluate_expression(&index.index)?;

                match (object, index_val) {
                    (Value::Array(arr), Value::Integer(i)) => {
                        if i < 0 || i as usize >= arr.len() {
                            return Err(VeyraError::runtime_error("Array index out of bounds"));
                        }
                        Ok(arr[i as usize].clone())
                    }
                    (Value::Dictionary(map), Value::String(key)) => {
                        map.get(&key).cloned().ok_or_else(|| {
                            VeyraError::runtime_error(format!(
                                "Key '{}' not found in dictionary",
                                key
                            ))
                        })
                    }
                    (Value::Tuple(tuple), Value::Integer(i)) => {
                        if i < 0 || i as usize >= tuple.len() {
                            return Err(VeyraError::runtime_error("Tuple index out of bounds"));
                        }
                        Ok(tuple[i as usize].clone())
                    }
                    (Value::String(s), Value::Integer(i)) => {
                        if i < 0 || i as usize >= s.len() {
                            return Err(VeyraError::runtime_error("String index out of bounds"));
                        }
                        Ok(Value::String(
                            s.chars().nth(i as usize).unwrap().to_string(),
                        ))
                    }
                    _ => Err(VeyraError::runtime_error("Invalid indexing operation")),
                }
            }
            Expression::Range(range) => {
                let start = self.evaluate_expression(&range.start)?;
                let end = self.evaluate_expression(&range.end)?;

                if let (Value::Integer(s), Value::Integer(e)) = (start, end) {
                    let mut elements = Vec::new();
                    let end_val = if range.inclusive { e + 1 } else { e };
                    for i in s..end_val {
                        elements.push(Value::Integer(i));
                    }
                    Ok(Value::Array(elements))
                } else {
                    Err(VeyraError::runtime_error(
                        "Range expressions require integer bounds",
                    ))
                }
            }
            Expression::ModuleAccess(module_access) => {
                // For now, we'll implement basic std library access
                let full_name = format!("{}::{}", module_access.module, module_access.item);

                // Check for constants first
                match full_name.as_str() {
                    "std::PI" | "math::PI" => Ok(Value::Float(std::f64::consts::PI)),
                    "std::E" | "math::E" => Ok(Value::Float(std::f64::consts::E)),
                    _ => {
                        // Try to find the item in the global environment (stdlib items are loaded globally)
                        self.environment
                            .get(&module_access.item)
                            .cloned()
                            .ok_or_else(|| {
                                VeyraError::runtime_error(format!(
                                    "Undefined module item '{}::{}'",
                                    module_access.module, module_access.item
                                ))
                            })
                    }
                }
            }
            Expression::Cast(cast) => {
                let value = self.evaluate_expression(&cast.expression)?;
                self.cast_value(value, &cast.target_type)
            }
            _ => Err(VeyraError::runtime_error("Expression type not implemented")),
        }
    }

    fn literal_to_value(&self, literal: &Literal) -> Value {
        match literal {
            Literal::Integer(n) => Value::Integer(*n),
            Literal::Float(f) => Value::Float(*f),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Char(c) => Value::Char(*c),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::None => Value::None,
        }
    }

    fn apply_binary_operator(
        &self,
        op: &BinaryOperator,
        left: &Value,
        right: &Value,
    ) -> Result<Value> {
        match (op, left, right) {
            // Arithmetic
            (BinaryOperator::Add, Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Integer(a + b))
            }
            (BinaryOperator::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (BinaryOperator::Add, Value::Integer(a), Value::Float(b)) => {
                Ok(Value::Float(*a as f64 + b))
            }
            (BinaryOperator::Add, Value::Float(a), Value::Integer(b)) => {
                Ok(Value::Float(a + *b as f64))
            }
            (BinaryOperator::Add, Value::String(a), Value::String(b)) => {
                Ok(Value::String(format!("{}{}", a, b)))
            }

            (BinaryOperator::Subtract, Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Integer(a - b))
            }
            (BinaryOperator::Subtract, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (BinaryOperator::Subtract, Value::Integer(a), Value::Float(b)) => {
                Ok(Value::Float(*a as f64 - b))
            }
            (BinaryOperator::Subtract, Value::Float(a), Value::Integer(b)) => {
                Ok(Value::Float(a - *b as f64))
            }

            (BinaryOperator::Multiply, Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Integer(a * b))
            }
            (BinaryOperator::Multiply, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (BinaryOperator::Multiply, Value::Integer(a), Value::Float(b)) => {
                Ok(Value::Float(*a as f64 * b))
            }
            (BinaryOperator::Multiply, Value::Float(a), Value::Integer(b)) => {
                Ok(Value::Float(a * *b as f64))
            }

            (BinaryOperator::Divide, Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    Err(VeyraError::runtime_error("Division by zero"))
                } else {
                    Ok(Value::Integer(a / b))
                }
            }
            (BinaryOperator::Divide, Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(VeyraError::runtime_error("Division by zero"))
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (BinaryOperator::Divide, Value::Integer(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(VeyraError::runtime_error("Division by zero"))
                } else {
                    Ok(Value::Float(*a as f64 / b))
                }
            }
            (BinaryOperator::Divide, Value::Float(a), Value::Integer(b)) => {
                if *b == 0 {
                    Err(VeyraError::runtime_error("Division by zero"))
                } else {
                    Ok(Value::Float(a / *b as f64))
                }
            }

            (BinaryOperator::Modulo, Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    Err(VeyraError::runtime_error("Modulo by zero"))
                } else {
                    Ok(Value::Integer(a % b))
                }
            }

            // Comparison
            (BinaryOperator::Equal, a, b) => Ok(Value::Boolean(self.values_equal(a, b))),
            (BinaryOperator::NotEqual, a, b) => Ok(Value::Boolean(!self.values_equal(a, b))),

            (BinaryOperator::Less, Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Boolean(a < b))
            }
            (BinaryOperator::Less, Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
            (BinaryOperator::Less, Value::String(a), Value::String(b)) => Ok(Value::Boolean(a < b)),
            (BinaryOperator::Less, Value::Char(a), Value::Char(b)) => Ok(Value::Boolean(a < b)),
            (BinaryOperator::Less, Value::String(a), Value::Char(b)) => {
                Ok(Value::Boolean(*a < b.to_string()))
            }
            (BinaryOperator::Less, Value::Char(a), Value::String(b)) => {
                Ok(Value::Boolean(a.to_string() < *b))
            }
            (BinaryOperator::LessEqual, Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Boolean(a <= b))
            }
            (BinaryOperator::LessEqual, Value::Float(a), Value::Float(b)) => {
                Ok(Value::Boolean(a <= b))
            }
            (BinaryOperator::LessEqual, Value::String(a), Value::String(b)) => {
                Ok(Value::Boolean(a <= b))
            }
            (BinaryOperator::LessEqual, Value::Char(a), Value::Char(b)) => {
                Ok(Value::Boolean(a <= b))
            }
            (BinaryOperator::LessEqual, Value::String(a), Value::Char(b)) => {
                Ok(Value::Boolean(*a <= b.to_string()))
            }
            (BinaryOperator::LessEqual, Value::Char(a), Value::String(b)) => {
                Ok(Value::Boolean(a.to_string() <= *b))
            }
            (BinaryOperator::Greater, Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Boolean(a > b))
            }
            (BinaryOperator::Greater, Value::Float(a), Value::Float(b)) => {
                Ok(Value::Boolean(a > b))
            }
            (BinaryOperator::Greater, Value::String(a), Value::String(b)) => {
                Ok(Value::Boolean(a > b))
            }
            (BinaryOperator::Greater, Value::Char(a), Value::Char(b)) => Ok(Value::Boolean(a > b)),
            (BinaryOperator::Greater, Value::String(a), Value::Char(b)) => {
                Ok(Value::Boolean(*a > b.to_string()))
            }
            (BinaryOperator::Greater, Value::Char(a), Value::String(b)) => {
                Ok(Value::Boolean(a.to_string() > *b))
            }
            (BinaryOperator::GreaterEqual, Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Boolean(a >= b))
            }
            (BinaryOperator::GreaterEqual, Value::Float(a), Value::Float(b)) => {
                Ok(Value::Boolean(a >= b))
            }
            (BinaryOperator::GreaterEqual, Value::String(a), Value::String(b)) => {
                Ok(Value::Boolean(a >= b))
            }
            (BinaryOperator::GreaterEqual, Value::Char(a), Value::Char(b)) => {
                Ok(Value::Boolean(a >= b))
            }
            (BinaryOperator::GreaterEqual, Value::String(a), Value::Char(b)) => {
                Ok(Value::Boolean(*a >= b.to_string()))
            }
            (BinaryOperator::GreaterEqual, Value::Char(a), Value::String(b)) => {
                Ok(Value::Boolean(a.to_string() >= *b))
            }

            // Logical (with short-circuiting)
            (BinaryOperator::And, a, b) => Ok(Value::Boolean(a.is_truthy() && b.is_truthy())),
            (BinaryOperator::Or, a, b) => Ok(Value::Boolean(a.is_truthy() || b.is_truthy())),

            // Power operation
            (BinaryOperator::Power, Value::Integer(base), Value::Integer(exp)) => {
                if *exp < 0 {
                    Ok(Value::Float((*base as f64).powf(*exp as f64)))
                } else {
                    let mut result = 1i64;
                    for _ in 0..*exp {
                        result *= base;
                    }
                    Ok(Value::Integer(result))
                }
            }
            (BinaryOperator::Power, Value::Float(base), Value::Float(exp)) => {
                Ok(Value::Float(base.powf(*exp)))
            }
            (BinaryOperator::Power, Value::Integer(base), Value::Float(exp)) => {
                Ok(Value::Float((*base as f64).powf(*exp)))
            }
            (BinaryOperator::Power, Value::Float(base), Value::Integer(exp)) => {
                Ok(Value::Float(base.powf(*exp as f64)))
            }

            // Bitwise operations (integers only)
            (BinaryOperator::BitwiseAnd, Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Integer(a & b))
            }
            (BinaryOperator::BitwiseOr, Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Integer(a | b))
            }
            (BinaryOperator::BitwiseXor, Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Integer(a ^ b))
            }
            (BinaryOperator::LeftShift, Value::Integer(a), Value::Integer(b)) => {
                if *b < 0 {
                    Err(VeyraError::runtime_error("Cannot shift by negative amount"))
                } else {
                    Ok(Value::Integer(a << b))
                }
            }
            (BinaryOperator::RightShift, Value::Integer(a), Value::Integer(b)) => {
                if *b < 0 {
                    Err(VeyraError::runtime_error("Cannot shift by negative amount"))
                } else {
                    Ok(Value::Integer(a >> b))
                }
            }

            _ => Err(VeyraError::runtime_error(format!(
                "Unsupported operation: {} {:?} {}",
                left.type_name(),
                op,
                right.type_name()
            ))),
        }
    }

    fn apply_unary_operator(&self, op: &UnaryOperator, operand: &Value) -> Result<Value> {
        match (op, operand) {
            (UnaryOperator::Minus, Value::Integer(n)) => Ok(Value::Integer(-n)),
            (UnaryOperator::Minus, Value::Float(f)) => Ok(Value::Float(-f)),
            (UnaryOperator::Not, val) => Ok(Value::Boolean(!val.is_truthy())),
            (UnaryOperator::BitwiseNot, Value::Integer(n)) => Ok(Value::Integer(!n)),
            (UnaryOperator::Reference, val) => {
                // Create an immutable reference
                Ok(Value::Reference(Reference {
                    value: Rc::new(RefCell::new(val.clone())),
                    mutable: false,
                }))
            }
            (UnaryOperator::MutableReference, val) => {
                // Create a mutable reference
                Ok(Value::Reference(Reference {
                    value: Rc::new(RefCell::new(val.clone())),
                    mutable: true,
                }))
            }
            (UnaryOperator::Dereference, Value::Reference(r)) => {
                // Dereference: get the value the reference points to
                Ok(r.value.borrow().clone())
            }
            (UnaryOperator::Dereference, val) => {
                // Attempting to dereference a non-reference is an error
                Err(VeyraError::runtime_error(format!(
                    "Cannot dereference non-reference type: {}",
                    val.type_name()
                )))
            }
            _ => Err(VeyraError::runtime_error(format!(
                "Unsupported unary operation: {:?} {}",
                op,
                operand.type_name()
            ))),
        }
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::String(a), Value::Char(b)) => *a == b.to_string(),
            (Value::Char(a), Value::String(b)) => a.to_string() == *b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::None, Value::None) => true,
            (Value::Reference(a), Value::Reference(b)) => {
                // References are equal if they point to the same location
                Rc::ptr_eq(&a.value, &b.value)
            }
            _ => false,
        }
    }

    fn add_values(&self, a: &Value, b: &Value) -> Result<Value> {
        self.apply_binary_operator(&BinaryOperator::Add, a, b)
    }

    fn cast_value(&self, value: Value, target_type: &Type) -> Result<Value> {
        match target_type {
            Type::Primitive(prim_type) => match prim_type {
                PrimitiveType::Int | PrimitiveType::I32 | PrimitiveType::I64 => match value {
                    Value::Integer(n) => Ok(Value::Integer(n)),
                    Value::Float(f) => Ok(Value::Integer(f as i64)),
                    Value::Boolean(b) => Ok(Value::Integer(if b { 1 } else { 0 })),
                    Value::String(s) => s.parse::<i64>().map(Value::Integer).map_err(|_| {
                        VeyraError::runtime_error(format!("Cannot cast '{}' to int", s))
                    }),
                    Value::Char(c) => Ok(Value::Integer(c as i64)),
                    _ => Err(VeyraError::runtime_error(format!(
                        "Cannot cast {} to int",
                        value.type_name()
                    ))),
                },
                PrimitiveType::F32 | PrimitiveType::F64 => match value {
                    Value::Integer(n) => Ok(Value::Float(n as f64)),
                    Value::Float(f) => Ok(Value::Float(f)),
                    Value::Boolean(b) => Ok(Value::Float(if b { 1.0 } else { 0.0 })),
                    Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| {
                        VeyraError::runtime_error(format!("Cannot cast '{}' to float", s))
                    }),
                    _ => Err(VeyraError::runtime_error(format!(
                        "Cannot cast {} to float",
                        value.type_name()
                    ))),
                },
                PrimitiveType::Bool => Ok(Value::Boolean(value.is_truthy())),
                PrimitiveType::String => Ok(Value::String(Self::value_to_string(&value))),
                PrimitiveType::Char => match value {
                    Value::Char(c) => Ok(Value::Char(c)),
                    Value::Integer(n) => {
                        if (0..=0x10FFFF).contains(&n) {
                            std::char::from_u32(n as u32)
                                .map(Value::Char)
                                .ok_or_else(|| {
                                    VeyraError::runtime_error(format!(
                                        "Invalid Unicode code point: {}",
                                        n
                                    ))
                                })
                        } else {
                            Err(VeyraError::runtime_error(format!(
                                "Invalid Unicode code point: {}",
                                n
                            )))
                        }
                    }
                    Value::String(s) => {
                        if s.len() == 1 {
                            Ok(Value::Char(s.chars().next().unwrap()))
                        } else {
                            Err(VeyraError::runtime_error(format!(
                                "Cannot cast string of length {} to char",
                                s.len()
                            )))
                        }
                    }
                    _ => Err(VeyraError::runtime_error(format!(
                        "Cannot cast {} to char",
                        value.type_name()
                    ))),
                },
                _ => Err(VeyraError::runtime_error(format!(
                    "Type casting to {:?} not supported",
                    prim_type
                ))),
            },
            _ => Err(VeyraError::runtime_error(format!(
                "Type casting to {:?} not supported",
                target_type
            ))),
        }
    }

    fn value_to_string(value: &Value) -> String {
        match value {
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Char(c) => c.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::None => "None".to_string(),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(Self::value_to_string).collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Dictionary(map) => {
                let mut pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, Self::value_to_string(v)))
                    .collect();
                pairs.sort(); // For consistent output
                format!("{{{}}}", pairs.join(", "))
            }
            Value::Set(set) => {
                let mut elements: Vec<String> = set.iter().map(|s| format!("\"{}\"", s)).collect();
                elements.sort(); // For consistent output
                format!("{{{}}}", elements.join(", "))
            }
            Value::Tuple(tuple) => {
                let elements: Vec<String> = tuple.iter().map(Self::value_to_string).collect();
                format!("({})", elements.join(", "))
            }
            Value::Reference(r) => {
                let prefix = if r.mutable { "&mut " } else { "&" };
                format!("{}{}", prefix, Self::value_to_string(&r.value.borrow()))
            }
        }
    }

    fn handle_import(&mut self, import: &Import) -> Result<()> {
        let module_path = import.path.join(".");

        // Built-in modules
        match module_path.as_str() {
            "std.math" => {
                self.load_math_module()?;
            }
            "std.collections" => {
                self.load_collections_module()?;
            }
            "std.string" => {
                self.load_string_module()?;
            }
            _ => {
                // Try to load from filesystem
                self.load_external_module(&module_path)?;
            }
        }

        Ok(())
    }

    fn load_math_module(&mut self) -> Result<()> {
        // Add math constants and functions to environment
        self.environment
            .define("PI".to_string(), Value::Float(std::f64::consts::PI));
        self.environment
            .define("E".to_string(), Value::Float(std::f64::consts::E));

        // Math functions would be added here
        Ok(())
    }

    fn load_collections_module(&mut self) -> Result<()> {
        // Collections functions are already built-in (len, push, pop, etc.)
        Ok(())
    }

    fn load_string_module(&mut self) -> Result<()> {
        // String functions would be added here
        Ok(())
    }

    fn load_external_module(&mut self, _module_path: &str) -> Result<()> {
        // TODO: Load external .vey files
        // For now, just ignore unknown modules
        Ok(())
    }

    fn parse_return_value(&self, value_str: &str) -> Result<Value> {
        // Simple return value parsing - in real implementation would be more sophisticated
        if let Ok(n) = value_str.parse::<i64>() {
            Ok(Value::Integer(n))
        } else if let Ok(f) = value_str.parse::<f64>() {
            Ok(Value::Float(f))
        } else if value_str == "true" {
            Ok(Value::Boolean(true))
        } else if value_str == "false" {
            Ok(Value::Boolean(false))
        } else if value_str == "none" {
            Ok(Value::None)
        } else {
            Ok(Value::String(value_str.to_string()))
        }
    }
}

impl Interpreter {
    fn load_stdlib(&mut self) -> Result<()> {
        // Temporarily disable stdlib loading to test basic functionality
        // TODO: Re-enable when stdlib parsing issues are fixed
        /*
        self.load_stdlib_file("stdlib/string.vey")?;  // Load string first since core depends on it
        self.load_stdlib_file("stdlib/math.vey")?;    // Load math next
        self.load_stdlib_file("stdlib/core.vey")?;    // Core depends on string and math
        self.load_stdlib_file("stdlib/collections.vey")?; // Collections depends on core
        self.load_stdlib_file("stdlib/io.vey")?;
        self.load_stdlib_file("stdlib/net.vey")?;
        self.load_stdlib_file("stdlib/datetime.vey")?;
        */

        Ok(())
    }

    fn _load_stdlib_file(&mut self, path: &str) -> Result<()> {
        use crate::lexer;
        use crate::parser;
        use std::fs;

        // Get the directory containing the executable
        let exe_path = std::env::current_exe()
            .map_err(|e| VeyraError::IoError(format!("Failed to get executable path: {}", e)))?;
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));

        // Go up from compiler/target/debug to veyra directory
        let project_root = exe_dir
            .parent() // up from debug
            .and_then(|d| d.parent()) // up from target
            .and_then(|d| d.parent()) // up from compiler
            .unwrap_or(exe_dir);

        let stdlib_path = project_root.join(path);

        let source = fs::read_to_string(&stdlib_path).map_err(|e| {
            VeyraError::IoError(format!(
                "Failed to load stdlib file '{}': {}",
                stdlib_path.display(),
                e
            ))
        })?;

        let tokens = lexer::tokenize(&source)?;
        let ast = parser::parse(tokens)?;

        // Load the stdlib program (this will add functions to our function map)
        self.interpret_program(&ast)?;

        Ok(())
    }
}

pub fn interpret(program: &Program) -> Result<()> {
    let mut interpreter = Interpreter::new();

    // Load standard library modules
    interpreter.load_stdlib()?;

    interpreter.interpret_program(program)
}
