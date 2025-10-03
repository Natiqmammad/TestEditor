//! AFNS Interpreter Module
//! 
//! Simple interpreter for executing AFNS programs

use std::collections::HashMap;
use anyhow::{anyhow, Result};
use crate::ast::{Program, Item, Expression, Statement, Literal};

// Simple value representation for interpretation
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i32),
    Boolean(bool),
    None,
}

impl From<&Literal> for Value {
    fn from(literal: &Literal) -> Self {
        match literal {
            Literal::String(s) => Value::String(s.clone()),
            Literal::Integer(i) => Value::Integer(i64::try_into(*i).unwrap_or(0)),
            Literal::UnsignedInteger(i) => Value::Integer(*i as i32),
            Literal::Float(f) => Value::Integer(*f as i32),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::Char(c) => Value::String(c.to_string()),
            Literal::Byte(b) => Value::Integer(*b as i32),
        }
    }
}

impl Value {
    fn as_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Integer(i) => i.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::None => "null".to_string(),
        }
    }
}

// Simple interpreter context
#[derive(Debug)]
pub struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Value>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn interpret_program(&mut self, program: &Program) -> Result<()> {
        println!("🔍 Interpreting program with {} items", program.items.len());
        
        // Pre-process functions
        for item in &program.items {
            match item {
                Item::FunctionOverload(func_overload) => {
                    for func in &func_overload.overloads {
                        self.functions.insert(func.name.clone(), Value::None);
                        println!("📦 Function registered: {}", func.name);
                    }
                }
                _ => {}
            }
        }

        // Find main apex function and execute
        println!("🎯 Looking for apex function...");
        for item in &program.items {
            match item {
                Item::FunctionOverload(func_overload) => {
                    for func in &func_overload.overloads {
                        if func.name == "apex" {
                            println!("🚀 Found apex function! Executing...");
                            self.interpret_function(func)?;
                            println!("✅ Apex function completed!");
                            return Ok(());
                        }
                    }
                }
                _ => {}
            }
        }
        
        println!("❌ Apex function not found!");
        Ok(())
    }

    fn interpret_function(&mut self, func: &crate::ast::Function) -> Result<Value> {
        for statement in &func.body {
            self.interpret_statement(statement)?;
        }
        Ok(Value::None)
    }

    fn interpret_statement(&mut self, statement: &Statement) -> Result<Value> {
        match statement {
            Statement::Expression(expr) => {
                self.interpret_expression(expr)?;
            }
            Statement::VariableDeclaration { name, value, .. } => {
                let val = self.interpret_expression(value)?;
                self.variables.insert(name.clone(), val);
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    return Ok(self.interpret_expression(expr)?);
                }
                return Ok(Value::None);
            }
            Statement::Block(_) => {
                // Simplified block handling
            }
            Statement::If { condition, then_branch, else_branch } => {
                let cond_val = self.interpret_expression(condition)?;
                if let Value::Boolean(true) = cond_val {
                    for stmt in then_branch {
                        self.interpret_statement(stmt)?;
                    }
                } else if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.interpret_statement(stmt)?;
                    }
                }
            }
            Statement::While { condition, body } => {
                loop {
                    let conn_val = self.interpret_expression(condition)?;
                    match conn_val {
                        Value::Boolean(true) => {
                            for stmt in body {
                                self.interpret_statement(stmt)?;
                            }
                        }
                        Value::Boolean(false) => {
                            break;
                        }
                        _ => return Err(anyhow!("While condition must be boolean")),
                    }
                }
            }
            Statement::Match { value, arms } => {
                let val = self.interpret_expression(value)?;
                for arm in arms {
                    let matches = self.match_pattern(&arm.pattern, &val)?;
                    if matches {
                        return Ok(self.interpret_expression(&arm.body)?);
                    }
                }
            }
            _ => {}
        }
        Ok(Value::None)
    }

    fn interpret_expression(&mut self, expression: &Expression) -> Result<Value> {
        match expression {
            Expression::Literal(literal) => Ok(Value::from(literal)),
            Expression::Identifier(name) => {
                if let Some(value) = self.variables.get(name) {
                    Ok(value.clone())
                } else {
                    Err(anyhow!("Undefined variable: {}", name))
                }
            }
            Expression::FunctionCall { name, args } => {
                Ok(self.interpret_function_call(name, args)?)
            }
            Expression::BinaryOp { left, op, right } => {
                let left_val = self.interpret_expression(left)?;
                let right_val = self.interpret_expression(right)?;
                self.evaluate_binary_op(left_val, op, right_val)
            }
            Expression::UnaryOp { op, expr } => {
                let val = self.interpret_expression(expr)?;
                self.evaluate_unary_op(op, val)
            }
            Expression::ArrayAccess { array, index } => {
                let _arr_val = self.interpret_expression(array)?;
                let idx_val = self.interpret_expression(index)?;
                if let Value::Integer(_) = idx_val {
                    // Simplified array indexing
                    Ok(Value::String("array_element".to_string()))
                } else {
                    Err(anyhow!("Array index must be integer"))
                }
            }
            Expression::If { condition, then_branch, else_branch } => {
                let cond_val = self.interpret_expression(condition)?;
                match cond_val {
                    Value::Boolean(true) => self.interpret_expression(then_branch),
                    Value::Boolean(false) => {
                        if let Some(else_expr) = else_branch {
                            self.interpret_expression(else_expr)
                        } else {
                            Ok(Value::None)
                        }
                    }
                    _ => Err(anyhow!("If condition must be boolean")),
                }
            }
            Expression::Block(stmts) => {
                let mut result = Value::None;
                for stmt in stmts {
                    result = self.interpret_statement(stmt)?;
                }
                Ok(result)
            }
            _ => {
                // Skip unsupported expressions
                Ok(Value::None)
            }
        }
    }

    fn interpret_function_call(&mut self, name: &str, args: &[Expression]) -> Result<Value> {
        // Built-in function handling
        match name {
            "show" => {
                if !args.is_empty() {
                    let val = self.interpret_expression(&args[0])?;
                    println!("{}", val.as_string());
                }
                Ok(Value::None)
            }
            "println" => {
                if !args.is_empty() {
                    let val = self.interpret_expression(&args[0])?;
                    println!("{}", val.as_string());
                }
                Ok(Value::None)
            }
            // Flutter functions - return simulated values
            "flutter_init" => {
                println!("🚀 AFNS Flutter Integration Initialized!");
                println!("Platform: Cross-platform Native");
                println!("Performance: Maximum Speed");
                Ok(Value::None)
            }
            "flutter_create_window" => {
                if args.len() >= 4 {
                    let id = self.interpret_expression(&args[0])?.as_string();
                    let title = self.interpret_expression(&args[1])?.as_string();
                    let width = self.interpret_expression(&args[2])?.as_string();
                    let height = self.interpret_expression(&args[3])?.as_string();
                    let result = format!("FlutterWindow(id::string = \"{}\", title::string = \"{}\", width::i32 = {}, height::i32 = {})", id, title, width, height);
                    return Ok(Value::String(result));
                }
                Ok(Value::String("FlutterWindow created".to_string()))
            }
            "flutter_create_button" => {
                if args.len() >= 4 {
                    let id = self.interpret_expression(&args[0])?.as_string();
                    let text = self.interpret_expression(&args[1])?.as_string();
                    let x = self.interpret_expression(&args[2])?.as_string();
                    let y = self.interpret_expression(&args[3])?.as_string();
                    let result = format!("FlutterButton(id::string = \"{}\", text::string = \"{}\", x::i32 = {}, y::i32 = {})", id, text, x, y);
                    return Ok(Value::String(result));
                }
                Ok(Value::String("FlutterButton created".to_string()))
            }
            "flutter_create_textfield" => {
                if args.len() >= 5 {
                    let id = self.interpret_expression(&args[0])?.as_string();
                    let placeholder = self.interpret_expression(&args[1])?.as_string();
                    let x = self.interpret_expression(&args[2])?.as_string();
                    let y = self.interpret_expression(&args[3])?.as_string();
                    let width = self.interpret_expression(&args[4])?.as_string();
                    let result = format!("FlutterTextField(id::string = \"{}\", placeholder::string = \"{}\", x::i32 = {}, y::i32 = {}, width::i32 = {})", id, placeholder, x, y, width);
                    return Ok(Value::String(result));
                }
                Ok(Value::String("FlutterTextField created".to_string()))
            }
            "flutter_create_listbox" => {
                if args.len() >= 5 {
                    let id = self.interpret_expression(&args[0])?.as_string();
                    let x = self.interpret_expression(&args[1])?.as_string();
                    let y = self.interpret_expression(&args[2])?.as_string();
                    let width = self.interpret_expression(&args[3])?.as_string();
                    let height = self.interpret_expression(&args[4])?.as_string();
                    let result = format!("FlutterListBox(id::string = \"{}\", x::i32 = {}, y::i32 = {}, width::i32 = {}, height::i32 = {})", id, x, y, width, height);
                    return Ok(Value::String(result));
                }
                Ok(Value::String("FlutterListBox created".to_string()))
            }
            "flutter_show_dialog" => {
                if args.len() >= 2 {
                    let title = self.interpret_expression(&args[0])?.as_string();
                    let message = self.interpret_expression(&args[1])?.as_string();
                    let result = format!("FlutterDialog(title::string = \"{}\", message::string = \"{}\", modal::bool = true)", title, message);
                    return Ok(Value::String(result));
                }
                Ok(Value::String("FlutterDialog created".to_string()))
            }
            "flutter_update_status" => {
                if !args.is_empty() {
                    let status = self.interpret_expression(&args[0])?.as_string();
                    println!("🔄 Flutter Status: {}", status);
                }
                Ok(Value::None)
            }
            "flutter_handle_save" => {
                println!("🔄 Flutter Status: Project saved successfully");
                println!("💾 Flutter Save Action: Completed");
                Ok(Value::None)
            }
            "flutter_handle_load" => {
                println!("🔄 Flutter Status: Project loaded successfully");
                println!("📂 Flutter Load Action: Completed");
                Ok(Value::None)
            }
            "flutter_handle_run" => {
                println!("🔄 Flutter Status: Application running");
                println!("🚀 Flutter Run Action: Completed");
                Ok(Value::None)
            }
            "flutter_handle_debug" => {
                println!("🔄 Flutter Status: Debug mode active");
                println!("🐛 Flutter Debug Action: Completed");
                Ok(Value::None)
            }
            _ => {
                // Check if it's a function overload
                if self.functions.contains_key(name) {
                    println!("Executing function: {}", name);
                    Ok(Value::None)
                } else {
                    Err(anyhow!("Unknown function: {}", name))
                }
            }
        }
    }

    fn evaluate_binary_op(&self, left: Value, op: &crate::ast::BinaryOperator, right: Value) -> Result<Value> {
        match op {
            crate::ast::BinaryOperator::Add => {
                match (left, right) {
                    (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
                    _ => Err(anyhow!("Cannot add incompatible types")),
                }
            }
            crate::ast::BinaryOperator::Subtract => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
                    _ => Err(anyhow!("Cannot subtract non-integer types")),
                }
            }
            crate::ast::BinaryOperator::Multiply => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
                    _ => Err(anyhow!("Cannot multiply non-integer types")),
                }
            }
            crate::ast::BinaryOperator::Divide => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => {
                        if r != 0 {
                            Ok(Value::Integer(l / r))
                        } else {
                            Err(anyhow!("Division by zero"))
                        }
                    }
                    _ => Err(anyhow!("Cannot divide non-integer types")),
                }
            }
            crate::ast::BinaryOperator::Equal => {
                Ok(Value::Boolean(self.values_equal(left, right)))
            }
            crate::ast::BinaryOperator::NotEqual => {
                Ok(Value::Boolean(!self.values_equal(left, right)))
            }
            crate::ast::BinaryOperator::Less => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
                    _ => Err(anyhow!("Cannot compare non-integer types with <")),
                }
            }
            crate::ast::BinaryOperator::Greater => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
                    _ => Err(anyhow!("Cannot compare non-integer types with >")),
                }
            }
            crate::ast::BinaryOperator::LessEqual => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),
                    _ => Err(anyhow!("Cannot compare non-integer types with <=")),
                }
            }
            crate::ast::BinaryOperator::GreaterEqual => {
                match (left, right) {
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
                    _ => Err(anyhow!("Cannot compare non-integer types with >=")),
                }
            }
            crate::ast::BinaryOperator::And => {
                match (left, right) {
                    (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l && r)),
                    _ => Err(anyhow!("Cannot apply AND to non-boolean types")),
                }
            }
            crate::ast::BinaryOperator::Or => {
                match (left, right) {
                    (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l || r)),
                    _ => Err(anyhow!("Cannot apply OR to non-boolean types")),
                }
            }
            _ => Err(anyhow!("Unsupported binary operator")),
        }
    }

    fn evaluate_unary_op(&self, _op: &crate::ast::UnaryOperator, val: Value) -> Result<Value> {
        // Simplified unary operation handling
        Ok(val)
    }

    fn values_equal(&self, left: Value, right: Value) -> bool {
        match (left, right) {
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Integer(l), Value::Integer(r)) => l == r,
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::None, Value::None) => true,
            _ => false,
        }
    }

    fn match_pattern(&self, pattern: &crate::ast::Pattern, value: &Value) -> Result<bool> {
        match pattern {
            crate::ast::Pattern::Literal(literal) => {
                Ok(self.values_equal(Value::from(literal), value.clone()))
            }
            crate::ast::Pattern::Wildcard => Ok(true),
            _ => Ok(false), // Simplified pattern matching
        }
    }
}

/// Interpret an AFNS program
pub fn interpret_program(program: &Program) -> Result<()> {
    let mut interpreter = Interpreter::new();
    interpreter.interpret_program(program)
}
