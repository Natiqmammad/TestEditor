//! Type system for ApexForge NightScript
//!
//! This module provides type checking and inference for AFNS.

use crate::ast::*;
use anyhow::Result;
use std::collections::HashMap;

/// Type checker for AFNS
pub struct TypeChecker {
    /// Type environment for variable bindings
    environment: HashMap<String, Type>,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        Self {
            environment: HashMap::new(),
        }
    }

    /// Type check a program
    pub fn check_program(&mut self, program: &Program) -> Result<()> {
        for item in &program.items {
            self.check_item(item)?;
        }
        Ok(())
    }

    /// Type check an item
    fn check_item(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::Function(func) => self.check_function(func)?,
            Item::FunctionOverload(overload_group) => {
                // Check all overloads in the group
                for func in overload_group.get_overloads() {
                    self.check_function(func)?;
                }
            }
            Item::Struct(struct_def) => self.check_struct(struct_def)?,
            Item::Enum(enum_def) => self.check_enum(enum_def)?,
            Item::Implementation(impl_block) => self.check_implementation(impl_block)?,
            Item::Trait(trait_def) => self.check_trait(trait_def)?,
            Item::Module(module) => self.check_module(module)?,
            Item::Import(_) => {
                // Imports are handled during parsing
            }
            Item::TypeAlias { name, type_ } => {
                self.environment.insert(name.clone(), type_.clone());
            }
        }
        Ok(())
    }

    /// Type check a function
    fn check_function(&mut self, func: &Function) -> Result<()> {
        // Add parameters to environment
        for (name, type_) in &func.parameters {
            self.environment.insert(name.clone(), type_.clone());
        }

        // Type check function body
        for stmt in &func.body {
            self.check_statement(stmt)?;
        }

        // Remove parameters from environment
        for (name, _) in &func.parameters {
            self.environment.remove(name);
        }

        Ok(())
    }

    /// Type check a statement
    fn check_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Expression(expr) => {
                self.check_expression(expr)?;
            }
            Statement::VariableDeclaration {
                name,
                type_annotation,
                value,
            } => {
                let value_type = self.check_expression(value)?;

                if let Some(annotation) = type_annotation {
                    if !self.types_compatible(&value_type, annotation) {
                        return Err(anyhow::anyhow!(
                            "Type mismatch: expected {:?}, got {:?}",
                            annotation,
                            value_type
                        ));
                    }
                }

                self.environment.insert(name.clone(), value_type);
            }
            Statement::Assignment { target, value } => {
                let target_type = self.check_expression(target)?;
                let value_type = self.check_expression(value)?;

                if !self.types_compatible(&target_type, &value_type) {
                    return Err(anyhow::anyhow!(
                        "Assignment type mismatch: expected {:?}, got {:?}",
                        target_type,
                        value_type
                    ));
                }
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_type = self.check_expression(condition)?;
                if !matches!(cond_type, Type::Bool) {
                    return Err(anyhow::anyhow!("If condition must be boolean"));
                }

                for stmt in then_branch {
                    self.check_statement(stmt)?;
                }

                if let Some(else_branch) = else_branch {
                    for stmt in else_branch {
                        self.check_statement(stmt)?;
                    }
                }
            }
            Statement::While { condition, body } => {
                let cond_type = self.check_expression(condition)?;
                if !matches!(cond_type, Type::Bool) {
                    return Err(anyhow::anyhow!("While condition must be boolean"));
                }

                for stmt in body {
                    self.check_statement(stmt)?;
                }
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                let iterable_type = self.check_expression(iterable)?;
                // TODO: Check if iterable is actually iterable

                for stmt in body {
                    self.check_statement(stmt)?;
                }
            }
            Statement::Loop { body } => {
                for stmt in body {
                    self.check_statement(stmt)?;
                }
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.check_expression(expr)?;
                }
            }
            Statement::Match { value, arms } => {
                let value_type = self.check_expression(value)?;

                for arm in arms {
                    self.check_pattern(&arm.pattern, &value_type)?;

                    if let Some(guard) = &arm.guard {
                        let guard_type = self.check_expression(guard)?;
                        if !matches!(guard_type, Type::Bool) {
                            return Err(anyhow::anyhow!("Match guard must be boolean"));
                        }
                    }

                    self.check_expression(&arm.body)?;
                }
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
            }
            Statement::Break | Statement::Continue => {
                // These are always valid in loops
            }
            Statement::Unsafe(statements) => {
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
            }
        }
        Ok(())
    }

    /// Type check an expression and return its type
    fn check_expression(&mut self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Literal(literal) => match literal {
                Literal::Integer(_) => Ok(Type::I32),
                Literal::UnsignedInteger(_) => Ok(Type::U32),
                Literal::Float(_) => Ok(Type::F64),
                Literal::Boolean(_) => Ok(Type::Bool),
                Literal::String(_) => Ok(Type::String),
                Literal::Char(_) => Ok(Type::Char),
                Literal::Byte(_) => Ok(Type::Byte),
            },
            Expression::Identifier(name) => self
                .environment
                .get(name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Undefined variable: {}", name)),
            Expression::BinaryOp { left, op: _, right } => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;

                // TODO: Implement proper binary operation type checking
                if self.types_compatible(&left_type, &right_type) {
                    Ok(left_type)
                } else {
                    Err(anyhow::anyhow!("Binary operation type mismatch"))
                }
            }
            Expression::UnaryOp { op: _, expr } => self.check_expression(expr),
            Expression::FunctionCall { name: _, args } => {
                // TODO: Implement function call type checking
                for arg in args {
                    self.check_expression(arg)?;
                }
                Ok(Type::I32) // Placeholder
            }
            Expression::MethodCall {
                object,
                method: _,
                args,
            } => {
                let object_type = self.check_expression(object)?;
                for arg in args {
                    self.check_expression(arg)?;
                }
                Ok(object_type) // Placeholder
            }
            Expression::ArrayAccess { array, index } => {
                let array_type = self.check_expression(array)?;
                let index_type = self.check_expression(index)?;

                if !matches!(index_type, Type::I32 | Type::U32 | Type::Usize) {
                    return Err(anyhow::anyhow!("Array index must be integer"));
                }

                match array_type {
                    Type::Array(element_type) => Ok(*element_type),
                    _ => Err(anyhow::anyhow!("Cannot index non-array type")),
                }
            }
            Expression::FieldAccess { object, field: _ } => self.check_expression(object),
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_type = self.check_expression(condition)?;
                if !matches!(cond_type, Type::Bool) {
                    return Err(anyhow::anyhow!("If condition must be boolean"));
                }

                let then_type = self.check_expression(then_branch)?;

                if let Some(else_branch) = else_branch {
                    let else_type = self.check_expression(else_branch)?;
                    if !self.types_compatible(&then_type, &else_type) {
                        return Err(anyhow::anyhow!("If branches must have compatible types"));
                    }
                }

                Ok(then_type)
            }
            Expression::Match { value, arms } => {
                let value_type = self.check_expression(value)?;

                if arms.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Match expression must have at least one arm"
                    ));
                }

                let mut return_type = None;
                for arm in arms {
                    self.check_pattern(&arm.pattern, &value_type)?;

                    if let Some(guard) = &arm.guard {
                        let guard_type = self.check_expression(guard)?;
                        if !matches!(guard_type, Type::Bool) {
                            return Err(anyhow::anyhow!("Match guard must be boolean"));
                        }
                    }

                    let arm_type = self.check_expression(&arm.body)?;

                    if let Some(ref expected_type) = return_type {
                        if !self.types_compatible(&arm_type, expected_type) {
                            return Err(anyhow::anyhow!("Match arms must have compatible types"));
                        }
                    } else {
                        return_type = Some(arm_type);
                    }
                }

                Ok(return_type.unwrap())
            }
            Expression::Block(statements) => {
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
                Ok(Type::I32) // Placeholder - blocks should return the last expression
            }
            Expression::Lambda { params, body } => {
                // Add parameters to environment
                for (name, type_) in params {
                    self.environment.insert(name.clone(), type_.clone());
                }

                let body_type = self.check_expression(body)?;

                // Remove parameters from environment
                for (name, _) in params {
                    self.environment.remove(name);
                }

                Ok(Type::Closure(
                    params.iter().map(|(_, t)| t.clone()).collect(),
                    Box::new(body_type),
                ))
            }
            Expression::Actor { params, body } => {
                // Similar to lambda
                for (name, type_) in params {
                    self.environment.insert(name.clone(), type_.clone());
                }

                let body_type = self.check_expression(body)?;

                for (name, _) in params {
                    self.environment.remove(name);
                }

                Ok(Type::Actor(
                    params.iter().map(|(_, t)| t.clone()).collect(),
                    Box::new(body_type),
                ))
            }
        }
    }

    /// Check if two types are compatible
    fn types_compatible(&self, t1: &Type, t2: &Type) -> bool {
        match (t1, t2) {
            (Type::I32, Type::I32) => true,
            (Type::U32, Type::U32) => true,
            (Type::F64, Type::F64) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::String, Type::String) => true,
            (Type::Char, Type::Char) => true,
            (Type::Byte, Type::Byte) => true,
            (Type::Array(t1), Type::Array(t2)) => self.types_compatible(t1, t2),
            (Type::Map(k1, v1), Type::Map(k2, v2)) => {
                self.types_compatible(k1, k2) && self.types_compatible(v1, v2)
            }
            (Type::Set(t1), Type::Set(t2)) => self.types_compatible(t1, t2),
            (Type::Option(t1), Type::Option(t2)) => self.types_compatible(t1, t2),
            (Type::Result(t1, e1), Type::Result(t2, e2)) => {
                self.types_compatible(t1, t2) && self.types_compatible(e1, e2)
            }
            (Type::Tuple(t1), Type::Tuple(t2)) => {
                t1.len() == t2.len()
                    && t1
                        .iter()
                        .zip(t2.iter())
                        .all(|(a, b)| self.types_compatible(a, b))
            }
            (Type::Reference(t1), Type::Reference(t2)) => self.types_compatible(t1, t2),
            (Type::MutableReference(t1), Type::MutableReference(t2)) => {
                self.types_compatible(t1, t2)
            }
            (Type::UserDefined(n1), Type::UserDefined(n2)) => n1 == n2,
            _ => false,
        }
    }

    /// Check a pattern against a type
    fn check_pattern(&self, pattern: &Pattern, type_: &Type) -> Result<()> {
        match (pattern, type_) {
            (Pattern::Literal(literal), expected_type) => {
                let literal_type = match literal {
                    Literal::Integer(_) => Type::I32,
                    Literal::UnsignedInteger(_) => Type::U32,
                    Literal::Float(_) => Type::F64,
                    Literal::Boolean(_) => Type::Bool,
                    Literal::String(_) => Type::String,
                    Literal::Char(_) => Type::Char,
                    Literal::Byte(_) => Type::Byte,
                };

                if !self.types_compatible(&literal_type, expected_type) {
                    return Err(anyhow::anyhow!("Pattern literal type mismatch"));
                }
            }
            (Pattern::Identifier(_), _) => {
                // Variable patterns match any type
            }
            (Pattern::Wildcard, _) => {
                // Wildcard patterns match any type
            }
            (Pattern::Tuple(patterns), Type::Tuple(types)) => {
                if patterns.len() != types.len() {
                    return Err(anyhow::anyhow!("Tuple pattern length mismatch"));
                }

                for (pattern, type_) in patterns.iter().zip(types.iter()) {
                    self.check_pattern(pattern, type_)?;
                }
            }
            (Pattern::Struct { name: _, fields }, Type::UserDefined(name)) => {
                // TODO: Look up struct definition and check fields
                for (field_name, field_pattern) in fields {
                    // TODO: Check field exists and type matches
                    self.check_pattern(field_pattern, &Type::I32)?; // Placeholder
                }
            }
            (
                Pattern::Enum {
                    name: _,
                    variant: _,
                    fields,
                },
                Type::UserDefined(name),
            ) => {
                // TODO: Look up enum definition and check variant
                for field_pattern in fields {
                    self.check_pattern(field_pattern, &Type::I32)?; // Placeholder
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Pattern type mismatch"));
            }
        }
        Ok(())
    }

    // Placeholder methods for unimplemented features
    fn check_struct(&mut self, _struct_def: &Struct) -> Result<()> {
        Ok(())
    }

    fn check_enum(&mut self, _enum_def: &Enum) -> Result<()> {
        Ok(())
    }

    fn check_implementation(&mut self, _impl_block: &Implementation) -> Result<()> {
        Ok(())
    }

    fn check_trait(&mut self, _trait_def: &Trait) -> Result<()> {
        Ok(())
    }

    fn check_module(&mut self, _module: &Module) -> Result<()> {
        Ok(())
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_checker_creation() {
        let checker = TypeChecker::new();
        assert!(checker.environment.is_empty());
    }

    #[test]
    fn test_types_compatible() {
        let checker = TypeChecker::new();

        assert!(checker.types_compatible(&Type::I32, &Type::I32));
        assert!(checker.types_compatible(&Type::String, &Type::String));
        assert!(!checker.types_compatible(&Type::I32, &Type::String));
    }
}
