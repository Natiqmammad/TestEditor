//! Parser for ApexForge NightScript
//! 
//! This module provides parsing functionality for AFNS source code,
//! converting tokens into an Abstract Syntax Tree.

use crate::ast::*;
use crate::lexer::{Token, TokenWithSpan};
use anyhow::{anyhow, Result};
use std::collections::VecDeque;

/// Parser for ApexForge NightScript
pub struct Parser {
    tokens: VecDeque<TokenWithSpan>,
    current: usize,
    function_overloads: std::collections::HashMap<String, FunctionOverload>,
}

impl Parser {
    /// Create a new parser with tokens
    pub fn new(tokens: Vec<TokenWithSpan>) -> Self {
        Self {
            tokens: tokens.into(),
            current: 0,
            function_overloads: std::collections::HashMap::new(),
        }
    }
    
    /// Parse the tokens into a program AST
    pub fn parse(&mut self) -> Result<Program> {
        let mut program = Program::new();
        
        while !self.is_at_end() {
            if let Some(item) = self.parse_item()? {
                program.add_item(item);
            }
        }
        
        // Add all function overloads to the program
        for (_, overload_group) in self.function_overloads.drain() {
            program.add_item(Item::FunctionOverload(overload_group));
        }
        
        Ok(program)
    }
    
    /// Parse a top-level item
    fn parse_item(&mut self) -> Result<Option<Item>> {
        // Skip whitespace and comments
        while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
            self.advance();
        }
        match self.peek() {
            Some(Token::Import) => {
                self.advance();
                let mut module_name = String::new();
                
                // Parse the module path (e.g., forge::collections::*)
                loop {
                    let part = self.expect_identifier()?;
                    module_name.push_str(&part);
                    
                    // Check if there's a :: separator
                    if self.check(&Token::TypeAnnotation) {
                        self.advance();
                        module_name.push_str("::");
                    } else {
                        break;
                    }
                }
                
                self.expect_semicolon()?;
                Ok(Some(Item::Import(module_name)))
            }
            Some(Token::Fun) => {
                let function = self.parse_function()?;
                
                // Check if this function name already exists
                if let Some(existing_overload) = self.function_overloads.get_mut(&function.name) {
                    // Add to existing overload group
                    existing_overload.add_overload(function);
                    Ok(None) // Don't add as separate item
                } else {
                    // Create new overload group
                    let mut overload_group = FunctionOverload::new(function.name.clone(), function.span);
                    overload_group.add_overload(function);
                    self.function_overloads.insert(overload_group.name.clone(), overload_group);
                    Ok(None) // Don't add as separate item, will be added at the end
                }
            }
            Some(Token::Struct) => {
                let struct_def = self.parse_struct()?;
                Ok(Some(Item::Struct(struct_def)))
            }
            Some(Token::Enum) => {
                let enum_def = self.parse_enum()?;
                Ok(Some(Item::Enum(enum_def)))
            }
            Some(Token::Impl) => {
                let impl_block = self.parse_implementation()?;
                Ok(Some(Item::Implementation(impl_block)))
            }
            Some(Token::Trait) => {
                let trait_def = self.parse_trait()?;
                Ok(Some(Item::Trait(trait_def)))
            }
            Some(Token::Mod) => {
                let module = self.parse_module()?;
                Ok(Some(Item::Module(module)))
            }
            Some(Token::Type) => {
                let type_alias = self.parse_type_alias()?;
                Ok(Some(Item::TypeAlias {
                    name: type_alias.0,
                    type_: type_alias.1,
                }))
            }
            Some(_) => {
                // Skip unrecognized tokens to avoid infinite loops
                self.advance();
                Ok(None)
            }
            None => Ok(None),
        }
    }
    
    /// Parse a function definition
    fn parse_function(&mut self) -> Result<Function> {
        let span = self.current_span();
        
        // Parse function keyword
        self.expect(Token::Fun)?;
        
        // Parse function name
        let name = self.expect_identifier()?;
        
        // Parse parameters
        let parameters = self.parse_parameters()?;
        
        // Parse return type
        // Skip whitespace before checking for arrow
        while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
            self.advance();
        }
        
        let return_type = if self.check(&Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        // Parse function body
        let body = self.parse_block()?;
        
        Ok(Function {
            name,
            parameters,
            return_type,
            body,
            is_async: false, // TODO: Parse async keyword
            is_unsafe: false, // TODO: Parse unsafe keyword
            is_public: false, // TODO: Parse pub keyword
            span,
        })
    }
    
    /// Parse function parameters
    fn parse_parameters(&mut self) -> Result<Vec<(String, Type)>> {
        self.expect(Token::LeftParen)?;
        
        let mut parameters = Vec::new();
        
        if !self.check(&Token::RightParen) {
            loop {
                let name = self.expect_identifier()?;
                self.expect(Token::TypeAnnotation)?; // Expect ::
                let type_ = self.parse_type()?;
                
                parameters.push((name, type_));
                
                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }
        
        self.expect(Token::RightParen)?;
        Ok(parameters)
    }
    
    /// Parse a type
    fn parse_type(&mut self) -> Result<Type> {
        // Skip whitespace before parsing type
        while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
            self.advance();
        }
        
        match self.peek() {
            Some(Token::Identifier(name)) => {
                let name_clone = name.clone();
                self.advance();
                self.parse_type_from_name(name_clone)
            }
            Some(Token::LeftBracket) => {
                self.parse_array_type()
            }
            Some(Token::LeftParen) => {
                self.parse_tuple_type()
            }
            _ => Err(anyhow!("Expected type")),
        }
    }
    
    /// Parse type from identifier
    fn parse_type_from_name(&mut self, name: String) -> Result<Type> {
        match name.as_str() {
            "i8" => Ok(Type::I8),
            "i16" => Ok(Type::I16),
            "i32" => Ok(Type::I32),
            "i64" => Ok(Type::I64),
            "i128" => Ok(Type::I128),
            "isize" => Ok(Type::Isize),
            "u8" => Ok(Type::U8),
            "u16" => Ok(Type::U16),
            "u32" => Ok(Type::U32),
            "u64" => Ok(Type::U64),
            "u128" => Ok(Type::U128),
            "usize" => Ok(Type::Usize),
            "f32" => Ok(Type::F32),
            "f64" => Ok(Type::F64),
            "bool" => Ok(Type::Bool),
            "string" => Ok(Type::String),
            "byte" => Ok(Type::Byte),
            "char" => Ok(Type::Char),
            "Decimal" => Ok(Type::Decimal),
            "BigInt" => Ok(Type::BigInt),
            "Complex" => Ok(Type::Complex),
            "Rational" => Ok(Type::Rational),
            "UUID" => Ok(Type::UUID),
            "Email" => Ok(Type::Email),
            "URL" => Ok(Type::URL),
            "IPAddress" => Ok(Type::IPAddress),
            "MACAddress" => Ok(Type::MACAddress),
            "Date" => Ok(Type::Date),
            "Duration" => Ok(Type::Duration),
            "Timeline" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Timeline(Box::new(inner)))
            }
            "Holo" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Holo(Box::new(inner)))
            }
            "Chain" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Chain(Box::new(inner)))
            }
            "Echo" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Echo(Box::new(inner)))
            }
            "Portal" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Portal(Box::new(inner)))
            }
            "Mirror" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Mirror(Box::new(inner)))
            }
            "Trace" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Trace(Box::new(inner)))
            }
            "Dream" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Dream(Box::new(inner)))
            }
            "Fractal" => {
                self.expect(Token::LeftBracket)?;
                let inner = self.parse_type()?;
                self.expect(Token::RightBracket)?;
                Ok(Type::Fractal(Box::new(inner)))
            }
            "Paradox" => {
                self.expect(Token::LeftBracket)?;
                let inner = self.parse_type()?;
                self.expect(Token::RightBracket)?;
                Ok(Type::Paradox(Box::new(inner)))
            }
            "Anchor" => {
                self.expect(Token::LeftBracket)?;
                let inner = self.parse_type()?;
                self.expect(Token::RightBracket)?;
                Ok(Type::Anchor(Box::new(inner)))
            }
            "CVar" => {
                self.expect(Token::LeftBracket)?;
                let inner = self.parse_type()?;
                self.expect(Token::RightBracket)?;
                Ok(Type::CVar(Box::new(inner)))
            }
            "Reactiv" => {
                self.expect(Token::LeftBracket)?;
                let inner = self.parse_type()?;
                self.expect(Token::RightBracket)?;
                Ok(Type::Reactiv(Box::new(inner)))
            }
            "Array" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Array(Box::new(inner)))
            }
            "Map" => {
                self.expect(Token::Less)?;
                let key = self.parse_type()?;
                self.expect(Token::Comma)?;
                let value = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Map(Box::new(key), Box::new(value)))
            }
            "Set" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Set(Box::new(inner)))
            }
            "Queue" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Queue(Box::new(inner)))
            }
            "Stack" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Stack(Box::new(inner)))
            }
            "LinkedList" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::LinkedList(Box::new(inner)))
            }
            "RingBuffer" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::RingBuffer(Box::new(inner)))
            }
            "CircularBuffer" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::CircularBuffer(Box::new(inner)))
            }
            "Option" => {
                self.expect(Token::Less)?;
                let inner = self.parse_type()?;
                self.expect(Token::Greater)?;
                Ok(Type::Option(Box::new(inner)))
            }
            "Result" => {
                self.expect(Token::LeftBracket)?;
                let ok = self.parse_type()?;
                self.expect(Token::Comma)?;
                let err = self.parse_type()?;
                self.expect(Token::RightBracket)?;
                Ok(Type::Result(Box::new(ok), Box::new(err)))
            }
            _ => Ok(Type::UserDefined(name)),
        }
    }
    
    /// Parse array type
    fn parse_array_type(&mut self) -> Result<Type> {
        self.expect(Token::LeftBracket)?;
        let inner = self.parse_type()?;
        self.expect(Token::RightBracket)?;
        Ok(Type::Array(Box::new(inner)))
    }
    
    /// Parse tuple type
    fn parse_tuple_type(&mut self) -> Result<Type> {
        self.expect(Token::LeftParen)?;
        
        let mut types = Vec::new();
        
        if !self.check(&Token::RightParen) {
            loop {
                let type_ = self.parse_type()?;
                types.push(type_);
                
                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }
        
        self.expect(Token::RightParen)?;
        Ok(Type::Tuple(types))
    }
    
    /// Parse a block of statements
    fn parse_block(&mut self) -> Result<Vec<Statement>> {
        // Skip whitespace and comments before expecting LeftBrace
        while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
            self.advance();
        }
        
        self.expect(Token::LeftBrace)?;
        
        let mut statements = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.parse_statement()? {
                statements.push(stmt);
            }
        }
        
        self.expect(Token::RightBrace)?;
        Ok(statements)
    }
    
    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Option<Statement>> {
        // Skip whitespace and comments
        while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
            self.advance();
        }
        
        
        match self.peek() {
            Some(Token::Var) => {
                self.parse_variable_declaration().map(Some)
            }
            Some(Token::If) => {
                self.parse_if_statement().map(Some)
            }
            Some(Token::While) => {
                self.parse_while_statement().map(Some)
            }
            Some(Token::For) => {
                self.parse_for_statement().map(Some)
            }
            Some(Token::Loop) => {
                self.parse_loop_statement().map(Some)
            }
            Some(Token::Return) => {
                self.parse_return_statement().map(Some)
            }
            Some(Token::Break) => {
                self.advance();
                self.expect_semicolon()?;
                Ok(Some(Statement::Break))
            }
            Some(Token::Continue) => {
                self.advance();
                self.expect_semicolon()?;
                Ok(Some(Statement::Continue))
            }
            Some(Token::Check) => {
                self.parse_match_statement().map(Some)
            }
            Some(Token::LeftBrace) => {
                let statements = self.parse_block()?;
                Ok(Some(Statement::Block(statements)))
            }
            _ => {
                // Try to parse as expression statement
                if let Some(expr) = self.parse_expression()? {
                    self.expect_semicolon()?;
                    Ok(Some(Statement::Expression(expr)))
                } else {
                    Ok(None)
                }
            }
        }
    }
    
    /// Parse variable declaration
    fn parse_variable_declaration(&mut self) -> Result<Statement> {
        self.expect(Token::Var)?;
        
        let name = self.expect_identifier()?;
        
        let type_annotation = if self.check(&Token::TypeAnnotation) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        // Skip whitespace before expecting Assign
        while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
            self.advance();
        }
        
        self.expect(Token::Assign)?;
        let value = self.parse_expression()?.ok_or_else(|| anyhow!("Expected expression"))?;
        
        self.expect_semicolon()?;
        
        Ok(Statement::VariableDeclaration {
            name,
            type_annotation,
            value,
        })
    }
    
    /// Parse if statement
    fn parse_if_statement(&mut self) -> Result<Statement> {
        self.expect(Token::If)?;
        
        let condition = self.parse_expression()?.ok_or_else(|| anyhow!("Expected condition"))?;
        
        let then_branch = self.parse_block()?;
        
        let else_branch = if self.check(&Token::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }
    
    /// Parse while statement
    fn parse_while_statement(&mut self) -> Result<Statement> {
        self.expect(Token::While)?;
        
        let condition = self.parse_expression()?.ok_or_else(|| anyhow!("Expected condition"))?;
        let body = self.parse_block()?;
        
        Ok(Statement::While { condition, body })
    }
    
    /// Parse for statement
    fn parse_for_statement(&mut self) -> Result<Statement> {
        self.expect(Token::For)?;
        
        let variable = self.expect_identifier()?;
        self.expect(Token::In)?;
        let iterable = self.parse_expression()?.ok_or_else(|| anyhow!("Expected iterable"))?;
        let body = self.parse_block()?;
        
        Ok(Statement::For {
            variable,
            iterable,
            body,
        })
    }
    
    /// Parse loop statement
    fn parse_loop_statement(&mut self) -> Result<Statement> {
        self.expect(Token::Loop)?;
        let body = self.parse_block()?;
        Ok(Statement::Loop { body })
    }
    
    /// Parse return statement
    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.expect(Token::Return)?;
        
        // Skip whitespace before checking for semicolon
        while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
            self.advance();
        }
        
        
        let value = if !self.check(&Token::Semicolon) {
            Some(self.parse_expression()?.ok_or_else(|| anyhow!("Expected expression"))?)
        } else {
            None
        };
        
        self.expect_semicolon()?;
        Ok(Statement::Return(value))
    }
    
    /// Parse match statement
    fn parse_match_statement(&mut self) -> Result<Statement> {
        self.expect(Token::Check)?;
        
        let value = self.parse_expression()?.ok_or_else(|| anyhow!("Expected expression"))?;
        
        self.expect(Token::LeftBrace)?;
        
        let mut arms = Vec::new();
        
            while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            
            let guard = if self.check(&Token::If) {
                self.advance();
                Some(self.parse_expression()?.ok_or_else(|| anyhow!("Expected guard expression"))?)
            } else {
                None
            };
            
            // Skip whitespace before FatArrow
            while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
                self.advance();
            }
            
            self.expect(Token::FatArrow)?;
            let body = self.parse_expression()?.ok_or_else(|| anyhow!("Expected match arm body"))?;
            
            arms.push(MatchArm { pattern, guard, body });
            
            if self.check(&Token::Comma) {
                self.advance();
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(Statement::Match { value, arms })
    }
    
    /// Parse pattern
    fn parse_pattern(&mut self) -> Result<Pattern> {
        // Skip whitespace before parsing pattern
        while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
            self.advance();
        }
        
        match self.peek() {
            Some(Token::Identifier(name)) => {
                let name_clone = name.clone();
                self.advance();
                Ok(Pattern::Identifier(name_clone))
            }
            Some(Token::BooleanLiteral(b)) => {
                let b_clone = *b;
                self.advance();
                Ok(Pattern::Literal(Literal::Boolean(b_clone)))
            }
            Some(Token::NumberLiteral(n)) => {
                let n_clone = n.clone();
                self.advance();
                // Try to parse as integer first
                if let Ok(i) = n_clone.parse::<i64>() {
                    Ok(Pattern::Literal(Literal::Integer(i)))
                } else if let Ok(f) = n_clone.parse::<f64>() {
                    Ok(Pattern::Literal(Literal::Float(f)))
                } else {
                    Err(anyhow!("Invalid number literal"))
                }
            }
            Some(Token::StringLiteral(s)) => {
                let s_clone = s.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(s_clone)))
            }
            Some(Token::CharLiteral(c)) => {
                let c_clone = *c;
                self.advance();
                Ok(Pattern::Literal(Literal::Char(c_clone)))
            }
            Some(Token::Wildcard) => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            _ => Err(anyhow!("Expected pattern")),
        }
    }
    
    /// Parse expression
    fn parse_expression(&mut self) -> Result<Option<Expression>> {
        // Skip whitespace and comments before parsing expression
        while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
            self.advance();
        }
        
        self.parse_assignment()
    }
    
    /// Parse assignment expression
    fn parse_assignment(&mut self) -> Result<Option<Expression>> {
        let expr = self.parse_equality()?;
        
        if let Some(ref expr) = expr {
            if self.check(&Token::Assign) {
                self.advance();
                let value = self.parse_assignment()?.ok_or_else(|| anyhow!("Expected assignment value"))?;
                return Ok(Some(Expression::BinaryOp {
                    left: Box::new(expr.clone()),
                    op: BinaryOperator::Equal,
                    right: Box::new(value),
                }));
            }
        }
        
        Ok(expr)
    }
    
    /// Parse equality expression
    fn parse_equality(&mut self) -> Result<Option<Expression>> {
        let mut expr = self.parse_comparison()?;
        
        while let Some(ref left) = expr {
            // Skip whitespace before checking for operator
            while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
                self.advance();
            }
            
            if self.check(&Token::Equal) {
                self.advance();
                let right = self.parse_comparison()?.ok_or_else(|| anyhow!("Expected right operand"))?;
                expr = Some(Expression::BinaryOp {
                    left: Box::new(left.clone()),
                    op: BinaryOperator::Equal,
                    right: Box::new(right),
                });
            } else if self.check(&Token::NotEqual) {
                self.advance();
                let right = self.parse_comparison()?.ok_or_else(|| anyhow!("Expected right operand"))?;
                expr = Some(Expression::BinaryOp {
                    left: Box::new(left.clone()),
                    op: BinaryOperator::NotEqual,
                    right: Box::new(right),
                });
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    /// Parse comparison expression
    fn parse_comparison(&mut self) -> Result<Option<Expression>> {
        let mut expr = self.parse_term()?;
        
        while let Some(ref left) = expr {
            // Skip whitespace before checking for operator
            while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
                self.advance();
            }
            
            if self.check(&Token::Greater) {
                self.advance();
                let right = self.parse_term()?.ok_or_else(|| anyhow!("Expected right operand"))?;
                expr = Some(Expression::BinaryOp {
                    left: Box::new(left.clone()),
                    op: BinaryOperator::Greater,
                    right: Box::new(right),
                });
            } else if self.check(&Token::GreaterEqual) {
                self.advance();
                let right = self.parse_term()?.ok_or_else(|| anyhow!("Expected right operand"))?;
                expr = Some(Expression::BinaryOp {
                    left: Box::new(left.clone()),
                    op: BinaryOperator::GreaterEqual,
                    right: Box::new(right),
                });
            } else if self.check(&Token::Less) {
                self.advance();
                let right = self.parse_term()?.ok_or_else(|| anyhow!("Expected right operand"))?;
                expr = Some(Expression::BinaryOp {
                    left: Box::new(left.clone()),
                    op: BinaryOperator::Less,
                    right: Box::new(right),
                });
            } else if self.check(&Token::LessEqual) {
                self.advance();
                let right = self.parse_term()?.ok_or_else(|| anyhow!("Expected right operand"))?;
                expr = Some(Expression::BinaryOp {
                    left: Box::new(left.clone()),
                    op: BinaryOperator::LessEqual,
                    right: Box::new(right),
                });
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    /// Parse term expression (addition, subtraction)
    fn parse_term(&mut self) -> Result<Option<Expression>> {
        let mut expr = self.parse_factor()?;
        
        while let Some(ref left) = expr {
            // Skip whitespace before checking for operator
            while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
                self.advance();
            }
            
            if self.check(&Token::Plus) {
                self.advance();
                let right = self.parse_factor()?.ok_or_else(|| anyhow!("Expected right operand"))?;
                expr = Some(Expression::BinaryOp {
                    left: Box::new(left.clone()),
                    op: BinaryOperator::Add,
                    right: Box::new(right),
                });
            } else if self.check(&Token::Minus) {
                self.advance();
                let right = self.parse_factor()?.ok_or_else(|| anyhow!("Expected right operand"))?;
                expr = Some(Expression::BinaryOp {
                    left: Box::new(left.clone()),
                    op: BinaryOperator::Subtract,
                    right: Box::new(right),
                });
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    /// Parse factor expression (multiplication, division)
    fn parse_factor(&mut self) -> Result<Option<Expression>> {
        let mut expr = self.parse_unary()?;
        
        while let Some(ref left) = expr {
            // Skip whitespace before checking for operator
            while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
                self.advance();
            }
            
            if self.check(&Token::Star) {
                self.advance();
                let right = self.parse_unary()?.ok_or_else(|| anyhow!("Expected right operand"))?;
                expr = Some(Expression::BinaryOp {
                    left: Box::new(left.clone()),
                    op: BinaryOperator::Multiply,
                    right: Box::new(right),
                });
            } else if self.check(&Token::Slash) {
                self.advance();
                let right = self.parse_unary()?.ok_or_else(|| anyhow!("Expected right operand"))?;
                expr = Some(Expression::BinaryOp {
                    left: Box::new(left.clone()),
                    op: BinaryOperator::Divide,
                    right: Box::new(right),
                });
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    /// Parse unary expression
    fn parse_unary(&mut self) -> Result<Option<Expression>> {
        // Skip whitespace before parsing unary expression
        while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
            self.advance();
        }
        
        if self.check(&Token::Not) {
            self.advance();
            let expr = self.parse_unary()?.ok_or_else(|| anyhow!("Expected operand"))?;
            return Ok(Some(Expression::UnaryOp {
                op: UnaryOperator::Not,
                expr: Box::new(expr),
            }));
        }
        
        if self.check(&Token::Minus) {
            self.advance();
            let expr = self.parse_unary()?.ok_or_else(|| anyhow!("Expected operand"))?;
            return Ok(Some(Expression::UnaryOp {
                op: UnaryOperator::Negate,
                expr: Box::new(expr),
            }));
        }
        
        self.parse_primary()
    }
    
    /// Parse primary expression
    fn parse_primary(&mut self) -> Result<Option<Expression>> {
        let mut expr = self.parse_atom()?;
        
        // Handle method calls and field access
        while let Some(ref current_expr) = expr {
            // Skip whitespace before checking for dot
            while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
                self.advance();
            }
            
            if self.check(&Token::Dot) {
                self.advance();
                let field_name = self.expect_identifier()?;
                
                // Check if this is a method call
                if self.check(&Token::LeftParen) {
                    expr = Some(self.parse_method_call(current_expr.clone(), field_name)?);
                } else {
                    expr = Some(Expression::FieldAccess {
                        object: Box::new(current_expr.clone()),
                        field: field_name,
                    });
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    /// Parse atom expression (literals, identifiers, parentheses)
    fn parse_atom(&mut self) -> Result<Option<Expression>> {
        match self.peek() {
            Some(Token::NumberLiteral(n)) => {
                let n_clone = n.clone();
                self.advance();
                // Try to parse as integer first
                if let Ok(i) = n_clone.parse::<i64>() {
                    Ok(Some(Expression::Literal(Literal::Integer(i))))
                } else if let Ok(f) = n_clone.parse::<f64>() {
                    Ok(Some(Expression::Literal(Literal::Float(f))))
                } else {
                    Err(anyhow!("Invalid number literal"))
                }
            }
            Some(Token::StringLiteral(s)) => {
                let s_clone = s.clone();
                self.advance();
                Ok(Some(Expression::Literal(Literal::String(s_clone))))
            }
            Some(Token::CharLiteral(c)) => {
                let c_clone = *c;
                self.advance();
                Ok(Some(Expression::Literal(Literal::Char(c_clone))))
            }
            Some(Token::BooleanLiteral(b)) => {
                let b_clone = *b;
                self.advance();
                Ok(Some(Expression::Literal(Literal::Boolean(b_clone))))
            }
            Some(Token::Identifier(name)) => {
                let name_clone = name.clone();
                self.advance();
                
                // Check if this is a static method call (e.g., Array::new)
                if self.check(&Token::TypeAnnotation) {
                    self.advance();
                    let method_name = self.expect_identifier()?;
                    
                    // Check if this is a method call
                    if self.check(&Token::LeftParen) {
                        self.parse_function_call(format!("{}::{}", name_clone, method_name)).map(Some)
                    } else {
                        Ok(Some(Expression::Identifier(format!("{}::{}", name_clone, method_name))))
                    }
                } else if self.check(&Token::LeftParen) {
                    // Regular function call
                    self.parse_function_call(name_clone).map(Some)
                } else {
                    Ok(Some(Expression::Identifier(name_clone)))
                }
            }
            Some(Token::LeftParen) => {
                self.advance();
                let expr = self.parse_expression()?.ok_or_else(|| anyhow!("Expected expression"))?;
                self.expect(Token::RightParen)?;
                Ok(Some(expr))
            }
            _ => Ok(None),
        }
    }
    
    /// Parse function call
    fn parse_function_call(&mut self, name: String) -> Result<Expression> {
        self.expect(Token::LeftParen)?;
        
        let mut args = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                args.push(self.parse_expression()?.ok_or_else(|| anyhow!("Expected expression"))?);
                if self.check(&Token::RightParen) {
                    break;
                }
                self.expect(Token::Comma)?;
            }
        }
        
        self.expect(Token::RightParen)?;
        
        Ok(Expression::FunctionCall { name, args })
    }
    
    /// Parse method call
    fn parse_method_call(&mut self, object: Expression, method_name: String) -> Result<Expression> {
        self.expect(Token::LeftParen)?;
        
        let mut args = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                args.push(self.parse_expression()?.ok_or_else(|| anyhow!("Expected expression"))?);
                if self.check(&Token::RightParen) {
                    break;
                }
                self.expect(Token::Comma)?;
            }
        }
        
        self.expect(Token::RightParen)?;
        
        Ok(Expression::MethodCall {
            object: Box::new(object),
            method: method_name,
            args,
        })
    }
    
    // Helper methods
    
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current).map(|t| &t.token)
    }
    
    fn advance(&mut self) -> Option<&TokenWithSpan> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn previous(&self) -> Option<&TokenWithSpan> {
        if self.current > 0 {
            Some(&self.tokens[self.current - 1])
        } else {
            None
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
    
    fn check(&self, token: &Token) -> bool {
        self.peek() == Some(token)
    }
    
    fn expect(&mut self, token: Token) -> Result<()> {
        if self.check(&token) {
            self.advance();
            Ok(())
        } else {
            Err(anyhow!("Expected {:?}", token))
        }
    }
    
    fn expect_identifier(&mut self) -> Result<String> {
        // Skip whitespace and comments
        while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
            self.advance();
        }
        
        
        match self.peek() {
            Some(Token::Identifier(name)) => {
                let name_clone = name.clone();
                self.advance();
                Ok(name_clone)
            }
            Some(Token::Apex) => {
                self.advance();
                Ok("apex".to_string())
            }
            Some(Token::Star) => {
                self.advance();
                Ok("*".to_string())
            }
            _ => Err(anyhow!("Expected identifier")),
        }
    }
    
    fn expect_semicolon(&mut self) -> Result<()> {
        // Skip whitespace before expecting semicolon
        while let Some(Token::Whitespace) | Some(Token::LineComment) | Some(Token::BlockComment) = self.peek() {
            self.advance();
        }
        
        self.expect(Token::Semicolon)
    }
    
    fn current_span(&self) -> Span {
        if let Some(token) = self.tokens.get(self.current) {
            Span::new(token.span.start, token.span.end, token.line, token.column)
        } else {
            Span::new(0, 0, 1, 1)
        }
    }
    
    // Placeholder methods for unimplemented features
    fn parse_struct(&mut self) -> Result<Struct> {
        Err(anyhow!("Struct parsing not implemented"))
    }
    
    fn parse_enum(&mut self) -> Result<Enum> {
        Err(anyhow!("Enum parsing not implemented"))
    }
    
    fn parse_implementation(&mut self) -> Result<Implementation> {
        Err(anyhow!("Implementation parsing not implemented"))
    }
    
    fn parse_trait(&mut self) -> Result<Trait> {
        Err(anyhow!("Trait parsing not implemented"))
    }
    
    fn parse_module(&mut self) -> Result<Module> {
        Err(anyhow!("Module parsing not implemented"))
    }
    
    fn parse_type_alias(&mut self) -> Result<(String, Type)> {
        Err(anyhow!("Type alias parsing not implemented"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::PositionalLexer;
    
    #[test]
    fn test_parse_simple_function() {
        let source = "fun main() { var x::i32 = 42; }";
        let tokens: Vec<TokenWithSpan> = PositionalLexer::new(source).collect();
        let mut parser = Parser::new(tokens);
        
        let program = parser.parse().unwrap();
        assert_eq!(program.items.len(), 1);
        
        if let Item::Function(func) = &program.items[0] {
            assert_eq!(func.name, "main");
            assert_eq!(func.parameters.len(), 0);
            assert_eq!(func.body.len(), 1);
        } else {
            panic!("Expected function");
        }
    }
    
    #[test]
    fn test_parse_variable_declaration() {
        let source = "var name::string = \"Hello\";";
        let tokens: Vec<TokenWithSpan> = PositionalLexer::new(source).collect();
        let mut parser = Parser::new(tokens);
        
        let program = parser.parse().unwrap();
        // This should parse as a statement, not a top-level item
        // For now, we'll just ensure it doesn't crash
        assert!(true);
    }
}
