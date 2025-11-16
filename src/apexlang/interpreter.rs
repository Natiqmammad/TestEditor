use std::collections::HashMap;

use num_bigint::BigInt;
use num_traits::Zero;

use crate::apexlang::ast::{
    BinaryOp, Expr, Function, ImportKind, Path, Program, Statement, UnaryOp, Value,
};
use crate::apexlang::error::ApexError;
use crate::apexlang::parser::Parser;
use crate::apexlang::stdlib::{NativeCallable, NativeRegistry};

pub fn evaluate_source(source: &str) -> Result<Value, ApexError> {
    let program = Parser::parse(source)?;
    let mut interpreter = Interpreter::new(&program)?;
    interpreter.evaluate()
}

struct Interpreter<'a> {
    program: &'a Program,
    functions: HashMap<&'a str, &'a Function>,
    registry: NativeRegistry,
    module_aliases: HashMap<String, String>,
    symbol_aliases: HashMap<String, NativeCallable>,
    frames: Vec<Frame>,
}

#[derive(Default)]
struct Frame {
    variables: HashMap<String, Variable>,
}

struct Variable {
    mutable: bool,
    value: Value,
}

enum Callable<'a> {
    User(&'a Function),
    Native(NativeCallable),
}

impl<'a> Interpreter<'a> {
    fn new(program: &'a Program) -> Result<Self, ApexError> {
        let mut functions = HashMap::new();
        for function in &program.functions {
            functions.insert(function.name.as_str(), function);
        }

        let mut interpreter = Self {
            program,
            functions,
            registry: NativeRegistry::with_standard_library(),
            module_aliases: HashMap::new(),
            symbol_aliases: HashMap::new(),
            frames: Vec::new(),
        };
        interpreter.initialize_imports()?;
        Ok(interpreter)
    }

    fn initialize_imports(&mut self) -> Result<(), ApexError> {
        for import in &self.program.imports {
            match &import.kind {
                ImportKind::Module { name, alias } => {
                    let module_name = name.as_str();
                    if self.registry.get_module(module_name).is_none() {
                        return Err(ApexError::new(format!("Unknown module '{}'", name)));
                    }
                    let alias_name = alias.clone().unwrap_or_else(|| name.clone());
                    self.module_aliases.insert(alias_name, name.clone());
                }
                ImportKind::Symbol {
                    module,
                    symbol,
                    alias,
                } => {
                    let callable = self.registry.get_callable(module, symbol).ok_or_else(|| {
                        ApexError::new(format!("Unknown symbol '{}::{}'", module, symbol))
                    })?;
                    let alias_name = alias.clone().unwrap_or_else(|| symbol.clone());
                    self.symbol_aliases.insert(alias_name, callable.clone());
                }
            }
        }
        Ok(())
    }

    fn evaluate(&mut self) -> Result<Value, ApexError> {
        let apex_fn = self
            .functions
            .get("apex")
            .ok_or_else(|| ApexError::new("Expected an 'apex' entry-point function"))?
            .clone();
        self.call_function(Callable::User(apex_fn), Vec::new())
    }

    fn call_function(
        &mut self,
        callable: Callable<'a>,
        arguments: Vec<Value>,
    ) -> Result<Value, ApexError> {
        match callable {
            Callable::Native(native) => native.call(&arguments),
            Callable::User(function) => {
                if function.params.len() != arguments.len() {
                    return Err(ApexError::new(format!(
                        "Function '{}' expected {} argument(s) but received {}",
                        function.name,
                        function.params.len(),
                        arguments.len()
                    )));
                }
                let mut frame = Frame::default();
                for (param, value) in function.params.iter().zip(arguments.into_iter()) {
                    if frame.variables.contains_key(param) {
                        return Err(ApexError::new(format!(
                            "Duplicate parameter '{}' in function '{}'",
                            param, function.name
                        )));
                    }
                    frame.variables.insert(
                        param.clone(),
                        Variable {
                            mutable: false,
                            value,
                        },
                    );
                }
                self.frames.push(frame);
                let result = self.execute_block(&function.body);
                self.frames.pop();
                result
            }
        }
    }

    fn execute_block(&mut self, statements: &[Statement]) -> Result<Value, ApexError> {
        for statement in statements {
            if let Some(value) = self.execute_statement(statement)? {
                return Ok(value);
            }
        }
        Err(ApexError::new("Function must return a value"))
    }

    fn execute_statement(&mut self, statement: &Statement) -> Result<Option<Value>, ApexError> {
        match statement {
            Statement::Let {
                name,
                mutable,
                value,
            } => {
                let value = self.evaluate_expr(value)?;
                self.declare_variable(name, *mutable, value)?;
                Ok(None)
            }
            Statement::Assignment { name, value } => {
                let new_value = self.evaluate_expr(value)?;
                let variable = self
                    .lookup_variable_mut(name)
                    .ok_or_else(|| ApexError::new(format!("Unknown variable '{}'", name)))?;
                if !variable.mutable {
                    return Err(ApexError::new(format!(
                        "Cannot assign to immutable binding '{}'",
                        name
                    )));
                }
                variable.value = new_value;
                Ok(None)
            }
            Statement::Expr(expr) => {
                self.evaluate_expr(expr)?;
                Ok(None)
            }
            Statement::Return(expr) => {
                let value = self.evaluate_expr(expr)?;
                Ok(Some(value))
            }
        }
    }

    fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, ApexError> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Path(path) => self.evaluate_path(path),
            Expr::Unary(op, inner) => {
                let value = self.evaluate_expr(inner)?;
                self.apply_unary(op, value)
            }
            Expr::Binary(lhs, op, rhs) => match op {
                BinaryOp::And => {
                    let left = self.evaluate_expr(lhs)?;
                    let left_bool = self.expect_bool(left, "left operand of '&&'")?;
                    if !left_bool {
                        return Ok(Value::Bool(false));
                    }
                    let right = self.evaluate_expr(rhs)?;
                    let right_bool = self.expect_bool(right, "right operand of '&&'")?;
                    Ok(Value::Bool(right_bool))
                }
                BinaryOp::Or => {
                    let left = self.evaluate_expr(lhs)?;
                    let left_bool = self.expect_bool(left, "left operand of '||'")?;
                    if left_bool {
                        return Ok(Value::Bool(true));
                    }
                    let right = self.evaluate_expr(rhs)?;
                    let right_bool = self.expect_bool(right, "right operand of '||'")?;
                    Ok(Value::Bool(right_bool))
                }
                _ => {
                    let left = self.evaluate_expr(lhs)?;
                    let right = self.evaluate_expr(rhs)?;
                    self.apply_binary(op, left, right)
                }
            },
            Expr::Call { callee, arguments } => {
                let args = self.evaluate_arguments(arguments)?;
                if let Expr::Path(path) = &**callee {
                    let callable = self.resolve_callable(path)?;
                    self.call_function(callable, args)
                } else {
                    Err(ApexError::new(
                        "Only named functions can be called in ApexLang MVP",
                    ))
                }
            }
        }
    }

    fn evaluate_arguments(&mut self, arguments: &[Expr]) -> Result<Vec<Value>, ApexError> {
        let mut values = Vec::with_capacity(arguments.len());
        for argument in arguments {
            values.push(self.evaluate_expr(argument)?);
        }
        Ok(values)
    }

    fn resolve_callable(&self, path: &Path) -> Result<Callable<'a>, ApexError> {
        if path.segments.is_empty() {
            return Err(ApexError::new("Invalid callable path"));
        }
        if path.segments.len() == 1 {
            let name = &path.segments[0];
            if self.lookup_variable(name).is_some() {
                return Err(ApexError::new(format!("'{}' is not callable", name)));
            }
            if let Some(native) = self.symbol_aliases.get(name) {
                return Ok(Callable::Native(native.clone()));
            }
            if let Some(function) = self.functions.get(name.as_str()) {
                return Ok(Callable::User(function));
            }
            return Err(ApexError::new(format!("Unknown function '{}'", name)));
        }
        if path.segments.len() == 2 {
            let module_alias = &path.segments[0];
            let symbol = &path.segments[1];
            let module_name = self.module_aliases.get(module_alias).ok_or_else(|| {
                ApexError::new(format!("Unknown module alias '{}'", module_alias))
            })?;
            let callable = self
                .registry
                .get_callable(module_name, symbol)
                .ok_or_else(|| {
                    ApexError::new(format!("Unknown symbol '{}::{}'", module_name, symbol))
                })?;
            return Ok(Callable::Native(callable.clone()));
        }
        Err(ApexError::new("Nested module paths are not supported"))
    }

    fn evaluate_path(&self, path: &Path) -> Result<Value, ApexError> {
        if path.segments.len() != 1 {
            return Err(ApexError::new(
                "Module-qualified identifiers must be called as functions",
            ));
        }
        let name = &path.segments[0];
        if let Some(variable) = self.lookup_variable(name) {
            return Ok(variable.value.clone());
        }
        if self.symbol_aliases.contains_key(name) || self.functions.contains_key(name.as_str()) {
            return Err(ApexError::new(format!(
                "'{}' is a function and must be called with parentheses",
                name
            )));
        }
        Err(ApexError::new(format!("Unknown identifier '{}'", name)))
    }

    fn declare_variable(
        &mut self,
        name: &str,
        mutable: bool,
        value: Value,
    ) -> Result<(), ApexError> {
        let frame = self
            .frames
            .last_mut()
            .ok_or_else(|| ApexError::new("Internal error: missing execution frame"))?;
        if frame.variables.contains_key(name) {
            return Err(ApexError::new(format!(
                "Variable '{}' is already defined in this scope",
                name
            )));
        }
        frame
            .variables
            .insert(name.to_string(), Variable { mutable, value });
        Ok(())
    }

    fn lookup_variable(&self, name: &str) -> Option<&Variable> {
        self.frames
            .iter()
            .rev()
            .find_map(|frame| frame.variables.get(name))
    }

    fn lookup_variable_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.frames
            .iter_mut()
            .rev()
            .find_map(|frame| frame.variables.get_mut(name))
    }

    fn apply_unary(&self, op: &UnaryOp, value: Value) -> Result<Value, ApexError> {
        match op {
            UnaryOp::Plus => match value {
                Value::Int(_) | Value::Number(_) => Ok(value),
                Value::Bool(_) => Err(ApexError::new("Unary '+' expects a numeric operand")),
            },
            UnaryOp::Minus => match value {
                Value::Int(v) => Ok(Value::Int(-v)),
                Value::Number(v) => Ok(Value::Number(-v)),
                Value::Bool(_) => Err(ApexError::new("Unary '-' expects a numeric operand")),
            },
            UnaryOp::Not => {
                let b = self.expect_bool(value, "operand of '!'")?;
                Ok(Value::Bool(!b))
            }
        }
    }

    fn apply_binary(&self, op: &BinaryOp, left: Value, right: Value) -> Result<Value, ApexError> {
        use BinaryOp::*;
        match op {
            Add | Sub | Mul | Div | Mod => self.numeric_binary(op, left, right),
            Eq | Ne => self.equality_binary(op, left, right),
            Lt | Le | Gt | Ge => self.comparison_binary(op, left, right),
            And | Or => unreachable!("handled earlier"),
        }
    }

    fn numeric_binary(&self, op: &BinaryOp, left: Value, right: Value) -> Result<Value, ApexError> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => match op {
                BinaryOp::Add => Ok(Value::Int(l + r)),
                BinaryOp::Sub => Ok(Value::Int(l - r)),
                BinaryOp::Mul => Ok(Value::Int(l * r)),
                BinaryOp::Div => {
                    if r.is_zero() {
                        Err(ApexError::new("Division by zero"))
                    } else {
                        Ok(Value::Int(l / r))
                    }
                }
                BinaryOp::Mod => {
                    if r.is_zero() {
                        Err(ApexError::new("Modulo by zero"))
                    } else {
                        Ok(Value::Int(l % r))
                    }
                }
                _ => unreachable!(),
            },
            (Value::Number(l), Value::Number(r)) => match op {
                BinaryOp::Add => Ok(Value::Number(l + r)),
                BinaryOp::Sub => Ok(Value::Number(l - r)),
                BinaryOp::Mul => Ok(Value::Number(l * r)),
                BinaryOp::Div => {
                    if r == 0.0 {
                        Err(ApexError::new("Division by zero"))
                    } else {
                        Ok(Value::Number(l / r))
                    }
                }
                BinaryOp::Mod => {
                    if r == 0.0 {
                        Err(ApexError::new("Modulo by zero"))
                    } else {
                        Ok(Value::Number(l % r))
                    }
                }
                _ => unreachable!(),
            },
            (Value::Int(l), Value::Number(r)) => {
                self.numeric_binary(op, Value::Number(l.to_f64()), Value::Number(r))
            }
            (Value::Number(l), Value::Int(r)) => {
                self.numeric_binary(op, Value::Number(l), Value::Number(r.to_f64()))
            }
            (Value::Bool(_), _) | (_, Value::Bool(_)) => Err(ApexError::new(
                "Numeric operations require numeric operands",
            )),
        }
    }

    fn equality_binary(
        &self,
        op: &BinaryOp,
        left: Value,
        right: Value,
    ) -> Result<Value, ApexError> {
        let result = match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Int(l), Value::Int(r)) => l == r,
            (Value::Number(l), Value::Number(r)) => (l - r).abs() <= f64::EPSILON,
            (Value::Int(l), Value::Number(r)) => (l.to_f64() - r).abs() <= f64::EPSILON,
            (Value::Number(l), Value::Int(r)) => (l - r.to_f64()).abs() <= f64::EPSILON,
            _ => {
                return Err(ApexError::new(
                    "Equality comparison requires compatible operands",
                ))
            }
        };
        Ok(Value::Bool(match op {
            BinaryOp::Eq => result,
            BinaryOp::Ne => !result,
            _ => unreachable!(),
        }))
    }

    fn comparison_binary(
        &self,
        op: &BinaryOp,
        left: Value,
        right: Value,
    ) -> Result<Value, ApexError> {
        let value = match (left, right) {
            (Value::Int(l), Value::Int(r)) => compare_bigints(op, &l, &r),
            (Value::Number(l), Value::Number(r)) => compare_f64(op, l, r)?,
            (Value::Int(l), Value::Number(r)) => compare_f64(op, l.to_f64(), r)?,
            (Value::Number(l), Value::Int(r)) => compare_f64(op, l, r.to_f64())?,
            _ => {
                return Err(ApexError::new("Comparison requires numeric operands"));
            }
        };
        Ok(Value::Bool(value))
    }

    fn expect_bool(&self, value: Value, context: &str) -> Result<bool, ApexError> {
        match value {
            Value::Bool(b) => Ok(b),
            _ => Err(ApexError::new(format!(
                "{} expects a boolean value",
                context
            ))),
        }
    }
}

trait ToFloat {
    fn to_f64(self) -> f64;
}

impl ToFloat for BigInt {
    fn to_f64(self) -> f64 {
        use num_traits::ToPrimitive;
        let sign = self.sign();
        ToPrimitive::to_f64(&self).unwrap_or_else(|| {
            if sign == num_bigint::Sign::Minus {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        })
    }
}

fn compare_bigints(op: &BinaryOp, left: &BigInt, right: &BigInt) -> bool {
    match op {
        BinaryOp::Lt => left < right,
        BinaryOp::Le => left <= right,
        BinaryOp::Gt => left > right,
        BinaryOp::Ge => left >= right,
        _ => unreachable!(),
    }
}

fn compare_f64(op: &BinaryOp, left: f64, right: f64) -> Result<bool, ApexError> {
    use std::cmp::Ordering;
    match left.partial_cmp(&right) {
        Some(ordering) => Ok(match (op, ordering) {
            (BinaryOp::Lt, Ordering::Less) => true,
            (BinaryOp::Le, Ordering::Less | Ordering::Equal) => true,
            (BinaryOp::Gt, Ordering::Greater) => true,
            (BinaryOp::Ge, Ordering::Greater | Ordering::Equal) => true,
            (BinaryOp::Lt, _) | (BinaryOp::Le, _) | (BinaryOp::Gt, _) | (BinaryOp::Ge, _) => false,
            _ => unreachable!(),
        }),
        None => Err(ApexError::new("Comparison involving NaN is undefined")),
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate_source;
    use crate::apexlang::ast::Value;

    #[test]
    fn evaluates_simple_expression() {
        let source = r#"
            fn apex() {
                return (1 + 2) * 3 - 4 / 2;
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        assert_eq!(value, Value::Int(7.into()));
    }

    #[test]
    fn supports_boolean_logic() {
        let source = r#"
            fn apex() {
                let a = true;
                let b = false;
                return (a && !b) || false;
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn calls_stdlib_function() {
        let source = r#"
            import nats;

            fn apex() {
                return nats.gcd(270, 192);
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        assert_eq!(value, Value::Int(6.into()));
    }

    #[test]
    fn uses_symbol_alias() {
        let source = r#"
            import nats.is_prime as prime;

            fn apex() {
                var flag = prime(97);
                flag = flag && !prime(95);
                return flag;
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn supports_mutable_bindings() {
        let source = r#"
            fn double_then_add(a, b) {
                var total = a * 2;
                total = total + b;
                return total;
            }

            fn apex() {
                return double_then_add(3, 4);
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        assert_eq!(value, Value::Int(10.into()));
    }

    #[test]
    fn mixes_math_and_nats_modules() {
        let source = r#"
            import math;
            import nats;
            import nats.btoi;

            fn apex() {
                let curvature = math.sqrt(144);
                let trig = math.sin(math.pi() / 4);
                let digits = nats.sum_digits(270);
                let hyp = math.hypot(3, 4);
                let bonus = btoi(nats.is_prime(97));
                return curvature + math.pow(trig, 2) + digits + hyp + bonus;
            }
        "#;

        let value = evaluate_source(source).expect("evaluation succeeded");
        match value {
            Value::Number(v) => assert!((v - 27.5).abs() < 1e-9),
            other => panic!("expected floating point result, got {:?}", other),
        }
    }
}
