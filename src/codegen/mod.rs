//! Code generation for ApexForge NightScript
//! 
//! This module provides code generation for different backends:
//! - LLVM IR for native compilation
//! - WASM for web deployment
//! - Bytecode for the AFNS virtual machine

use crate::ast::{Expression, Literal, Statement, BinaryOperator, Program, Item, Function, Pattern};
use crate::ast::Type;
use anyhow::Result;
use std::fmt::Write;

/// Trait for code generators
pub trait CodeGenerator {
    fn generate(&mut self, program: &Program) -> Result<String>;
}

/// LLVM IR Code Generator
pub struct LLVMCodeGenerator;

impl LLVMCodeGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate_ir(&mut self, program: &Program) -> Result<String> {
        let mut output = String::new();
        
        // LLVM header
        output.push_str("declare i32 @printf(i8*, ...)\n");
        output.push_str("declare i8* @string_concat(i8*, i8*)\n");
        output.push_str("declare void @show(i8*)\n");
        output.push_str("declare void @println(i8*)\n\n");
        
        // Generate string constants
        self.collect_string_literals(program, &mut output)?;
        
        // Generate functions
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    output.push_str(&self.generate_function(func)?);
                    output.push('\n');
                }
                Item::FunctionOverload(overload_group) => {
                    // Generate all overloads with unique names
                    for (i, func) in overload_group.get_overloads().iter().enumerate() {
                        let mut func_with_unique_name = func.clone();
                        func_with_unique_name.name = format!("{}_{}", func.name, i);
                        output.push_str(&self.generate_function(&func_with_unique_name)?);
                        output.push('\n');
                    }
                }
                _ => {}
            }
        }
        
        Ok(output)
    }
    
    fn collect_string_literals(&mut self, program: &Program, output: &mut String) -> Result<()> {
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    self.collect_strings_in_function(func, output)?;
                }
                Item::FunctionOverload(overload_group) => {
                    for func in overload_group.get_overloads() {
                        self.collect_strings_in_function(func, output)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    fn collect_strings_in_function(&self, func: &Function, output: &mut String) -> Result<()> {
        for stmt in &func.body {
            self.collect_strings_in_statement(stmt, output)?;
        }
        Ok(())
    }
    
    fn collect_strings_in_statement(&self, stmt: &Statement, output: &mut String) -> Result<()> {
        match stmt {
            Statement::Expression(expr) => {
                self.collect_strings_in_expression(expr, output)?;
            }
            Statement::VariableDeclaration { name: _, type_annotation: _, value } => {
                self.collect_strings_in_expression(value, output)?;
            }
            Statement::Return(Some(expr)) => {
                self.collect_strings_in_expression(expr, output)?;
            }
            _ => {}
        }
        Ok(())
    }
    
    fn collect_strings_in_expression(&self, expr: &Expression, output: &mut String) -> Result<()> {
        match expr {
            Expression::Literal(Literal::String(s)) => {
                let const_name = format!("@.str.{}", s.len());
                writeln!(output, "{} = private unnamed_addr constant [{} x i8] c\"{}\00\"", 
                    const_name, s.len() + 1, s)?;
            }
            Expression::BinaryOp { left, op: _, right } => {
                self.collect_strings_in_expression(left, output)?;
                self.collect_strings_in_expression(right, output)?;
            }
            Expression::FunctionCall { name: _, args } => {
                for arg in args {
                    self.collect_strings_in_expression(arg, output)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn generate_function(&mut self, func: &Function) -> Result<String> {
        let mut output = String::new();
        
        // Function signature
        let return_type = match &func.return_type {
            Some(t) => self.type_to_llvm(t),
            None => "void".to_string(),
        };
        
        output.push_str(&format!("define {} @{}(\n", return_type, func.name));
        
        // Parameters
        for (i, (name, type_)) in func.parameters.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("{} %{}", self.type_to_llvm(type_), name));
        }
        output.push_str(") {\n");
        
        // Function body
        for stmt in &func.body {
            output.push_str(&self.generate_statement(stmt, &func.parameters)?);
            output.push('\n');
        }
        
        output.push_str("}\n");
        Ok(output)
    }
    
    fn generate_statement(&mut self, stmt: &Statement, params: &[(String, Type)]) -> Result<String> {
        match stmt {
            Statement::Expression(expr) => {
                self.generate_expression(expr, params)
            }
            Statement::VariableDeclaration { name, type_annotation, value } => {
                let value_code = self.generate_expression(value, params)?;
                let type_str = match type_annotation {
                    Some(t) => self.type_to_llvm(t),
                    None => "i32".to_string(),
                };
                Ok(format!("  %{} = {}", name, value_code))
            }
            Statement::Return(expr) => {
                match expr {
                    Some(expr) => {
                        let expr_code = self.generate_expression(expr, params)?;
                        Ok(format!("  ret {}", expr_code))
                    }
                    None => Ok("  ret void".to_string()),
                }
            }
            Statement::Match { value, arms } => {
                let value_code = self.generate_expression(value, params)?;
                let mut output = String::new();
                
                if arms.is_empty() {
                    return Ok(output);
                }
                
                // Generate comparison for first pattern
                if let Some(first_arm) = arms.first() {
                    match &first_arm.pattern {
                        Pattern::Literal(Literal::Integer(i)) => {
                            output.push_str(&format!("  %cmp0 = eq i32 {}, {}\n", value_code, i));
                            
                            // Generate if-then-else structure
                            output.push_str("  br i1 %cmp0, label %match0, label %next0\n");
                            output.push_str("\n");
                            output.push_str("match0:\n");
                            
                            // Generate body expression for this arm
                            let body_code = self.generate_expression(&first_arm.body, params)?;
                            output.push_str(&format!("  {}\n", body_code));
                            output.push_str("  ret void\n");
                            output.push_str("\n");
                            output.push_str("next0:\n");
                            
                            // Handle remaining arms (simplified)
                            if arms.len() > 1 {
                                if let Some(second_arm) = arms.get(1) {
                                    match &second_arm.pattern {
                                        Pattern::Wildcard => {
                                            output.push_str("  ; Wildcard pattern - default case\n");
                                            let body_code = self.generate_expression(&second_arm.body, params)?;
                                            output.push_str(&format!("  {}\n", body_code));
                                            output.push_str("  ret void\n");
                                        }
                                        Pattern::Literal(Literal::Integer(i)) => {
                                            output.push_str(&format!("  %cmp1 = eq i32 {}, {}\n", value_code, i));
                                            output.push_str("  br i1 %cmp1, label %match1, label %next1\n");
                                            output.push_str("\n");
                                            output.push_str("match1:\n");
                                            let body_code = self.generate_expression(&second_arm.body, params)?;
                                            output.push_str(&format!("  {}\n", body_code));
                                            output.push_str("  ret void\n");
                                            output.push_str("\n");
                                            output.push_str("next1:\n");
                                            output.push_str("  ; TODO: Handle more patterns\n");
                                            output.push_str("  ret i8* null\n");
                                        }
                                        _ => output.push_str("  ; TODO: Handle pattern\n"),
                                    }
                                }
                            } else {
                                output.push_str("  ; No more patterns\n");
                                output.push_str("  ret i8* null\n");
                            }
                        }
                        Pattern::Wildcard => {
                            output.push_str("  ; Wildcard pattern - always matches\n");
                            let body_code = self.generate_expression(&first_arm.body, params)?;
                            output.push_str(&format!("  %result = {}\n", body_code));
                            output.push_str("  ret i8* %result\n");
                        }
                        _ => output.push_str("  ; TODO: Implement pattern matching"),
                    }
                }
                
                Ok(output)
            }
            _ => {
                Ok("  ; TODO: Implement statement".to_string())
            }
        }
    }
    
    fn generate_expression(&mut self, expr: &Expression, params: &[(String, Type)]) -> Result<String> {
        match expr {
            Expression::Literal(literal) => {
                match literal {
                    Literal::Integer(i) => Ok(format!("i32 {}", i)),
                    Literal::UnsignedInteger(u) => Ok(format!("i32 {}", u)),
                    Literal::Float(f) => Ok(format!("double {}", f)),
                    Literal::Boolean(b) => Ok(format!("i1 {}", if *b { 1 } else { 0 })),
                    Literal::String(s) => {
                        let const_name = self.get_string_constant_name(s);
                        Ok(format!("getelementptr inbounds ([{} x i8], [{} x i8]* {}, i32 0, i32 0)", s.len() + 1, s.len() + 1, const_name))
                    }
                    Literal::Char(c) => Ok(format!("i8 {}", *c as u8)),
                    Literal::Byte(b) => Ok(format!("i8 {}", b)),
                }
            }
            Expression::Identifier(name) => {
                Ok(format!("%{}", name))
            }
            Expression::BinaryOp { left, op, right } => {
                let left_code = self.generate_expression(left, params)?;
                let right_code = self.generate_expression(right, params)?;
                
                match op {
                    BinaryOperator::Add => {
                        // Check for string concatenation by examining operands
                        let left_is_string = match **left {
                            Expression::Literal(Literal::String(_)) => true,
                            Expression::Identifier(ref name) => {
                                params.iter().any(|(param_name, param_type)| {
                                    param_name == name && matches!(param_type, Type::String)
                                })
                            },
                            _ => false,
                        };
                        
                        let right_is_string = match **right {
                            Expression::Literal(Literal::String(_)) => true,
                            Expression::Identifier(ref name) => {
                                params.iter().any(|(param_name, param_type)| {
                                    param_name == name && matches!(param_type, Type::String)
                                })
                            },
                            _ => false,
                        };
                        
                        if left_is_string || right_is_string {
                            Ok(format!("call i8* @string_concat(i8* {}, i8* {})", left_code, right_code))
                        } else {
                            // Numeric addition
                            Ok(format!("add i32 {}, {}", left_code, right_code))
                        }
                    }
                    BinaryOperator::Subtract => {
                        Ok(format!("sub i32 {}, {}", left_code, right_code))
                    }
                    BinaryOperator::Multiply => {
                        Ok(format!("mul i32 {}, {}", left_code, right_code))
                    }
                    BinaryOperator::Divide => {
                        Ok(format!("sdiv i32 {}, {}", left_code, right_code))
                    }
                    BinaryOperator::Equal => {
                        Ok(format!("eq i32 {}, {}", left_code, right_code))
                    }
                    BinaryOperator::NotEqual => {
                        Ok(format!("ne i32 {}, {}", left_code, right_code))
                    }
                    _ => Ok(format!("add i32 {}, {}", left_code, right_code)),
                }
            }
            Expression::FunctionCall { name, args } => {
                let mut arg_codes = Vec::new();
                for arg in args {
                    arg_codes.push(self.generate_expression(arg, params)?);
                }
                
                // Built-in functions handling
                let return_type = if name == "show" || name == "println" {
                    "void"
                } else if name.starts_with("print_string_") {
                    "i8*"
                } else if name.starts_with("add_") {
                    "i32"  // Default to i32 for function overloading
                } else {
                    "void"
                };
                
            Ok(format!("call {} @{}({})", return_type, name, arg_codes.join(", ")))
            }
            _ => Ok("  ; TODO: Implement expression".to_string()),
        }
    }
    
    fn type_to_llvm(&self, type_: &Type) -> String {
        match type_ {
            Type::I32 => "i32".to_string(),
            Type::F64 => "double".to_string(),
            Type::Bool => "i1".to_string(),
            Type::String => "i8*".to_string(),
            Type::Char => "i8".to_string(),
            Type::Byte => "i8".to_string(),
            _ => "i32".to_string(),
        }
    }
    
    fn get_string_constant_name(&self, s: &str) -> String {
        format!("@.str.{}", s.len())
    }
}

impl CodeGenerator for LLVMCodeGenerator {
    fn generate(&mut self, program: &Program) -> Result<String> {
        self.generate_ir(program)
    }
}

/// WASM Code Generator (simplified)
pub struct WASMCodeGenerator;

impl CodeGenerator for WASMCodeGenerator {
    fn generate(&mut self, program: &Program) -> Result<String> {
        Ok(";; WASM code generation - TODO".to_string())
    }
}

/// Bytecode Generator (simplified)
pub struct BytecodeGenerator;

impl CodeGenerator for BytecodeGenerator {
    fn generate(&mut self, program: &Program) -> Result<String> {
        Ok(";; Bytecode generation - TODO".to_string())
    }
}