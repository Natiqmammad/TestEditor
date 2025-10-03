//! Abstract Syntax Tree for ApexForge NightScript
//! 
//! This module defines the AST nodes that represent the parsed AFNS source code.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Position information for AST nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }
}

/// Type annotations for AFNS
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    // Primitives
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    F32,
    F64,
    Bool,
    String,
    Byte,
    Char,
    
    // Math types
    Decimal,
    BigInt,
    Complex,
    Rational,
    
    // Collections
    Array(Box<Type>),
    Map(Box<Type>, Box<Type>), // Map<K, V>
    Set(Box<Type>),
    Queue(Box<Type>),
    Stack(Box<Type>),
    LinkedList(Box<Type>),
    RingBuffer(Box<Type>),
    CircularBuffer(Box<Type>),
    
    // Struct types
    Tuple(Vec<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>), // Result<T, E>
    
    // Special types
    UUID,
    Email,
    URL,
    IPAddress,
    MACAddress,
    Date,
    Duration,
    
    // Special AFNS types
    Timeline(Box<Type>),
    Holo(Box<Type>),
    Chain(Box<Type>),
    Echo(Box<Type>),
    Portal(Box<Type>),
    Mirror(Box<Type>),
    Trace(Box<Type>),
    Dream(Box<Type>),
    Fractal(Box<Type>),
    Paradox(Box<Type>),
    Anchor(Box<Type>),
    CVar(Box<Type>),
    Reactiv(Box<Type>),
    
    // Function types
    Function(Vec<Type>, Box<Type>), // Function(parameters, return_type)
    Closure(Vec<Type>, Box<Type>),
    Actor(Vec<Type>, Box<Type>),
    
    // Generic types
    Generic(String),
    
    // User-defined types
    UserDefined(String),
    
    // Reference types
    Reference(Box<Type>),
    MutableReference(Box<Type>),
    RawPointer(Box<Type>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::I128 => write!(f, "i128"),
            Type::Isize => write!(f, "isize"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::U128 => write!(f, "u128"),
            Type::Usize => write!(f, "usize"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Byte => write!(f, "byte"),
            Type::Char => write!(f, "char"),
            Type::Decimal => write!(f, "Decimal"),
            Type::BigInt => write!(f, "BigInt"),
            Type::Complex => write!(f, "Complex"),
            Type::Rational => write!(f, "Rational"),
            Type::Array(inner) => write!(f, "Array<{}>", inner),
            Type::Map(key, value) => write!(f, "Map<{}, {}>", key, value),
            Type::Set(inner) => write!(f, "Set<{}>", inner),
            Type::Queue(inner) => write!(f, "Queue<{}>", inner),
            Type::Stack(inner) => write!(f, "Stack<{}>", inner),
            Type::LinkedList(inner) => write!(f, "LinkedList<{}>", inner),
            Type::RingBuffer(inner) => write!(f, "RingBuffer<{}>", inner),
            Type::CircularBuffer(inner) => write!(f, "CircularBuffer<{}>", inner),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Type::Option(inner) => write!(f, "Option<{}>", inner),
            Type::Result(ok, err) => write!(f, "Result<{}, {}>", ok, err),
            Type::UUID => write!(f, "UUID"),
            Type::Email => write!(f, "Email"),
            Type::URL => write!(f, "URL"),
            Type::IPAddress => write!(f, "IPAddress"),
            Type::MACAddress => write!(f, "MACAddress"),
            Type::Date => write!(f, "Date"),
            Type::Duration => write!(f, "Duration"),
            Type::Timeline(inner) => write!(f, "Timeline<{}>", inner),
            Type::Holo(inner) => write!(f, "Holo<{}>", inner),
            Type::Chain(inner) => write!(f, "Chain<{}>", inner),
            Type::Echo(inner) => write!(f, "Echo<{}>", inner),
            Type::Portal(inner) => write!(f, "Portal<{}>", inner),
            Type::Mirror(inner) => write!(f, "Mirror<{}>", inner),
            Type::Trace(inner) => write!(f, "Trace<{}>", inner),
            Type::Dream(inner) => write!(f, "Dream<{}>", inner),
            Type::Fractal(inner) => write!(f, "Fractal<{}>", inner),
            Type::Paradox(inner) => write!(f, "Paradox<{}>", inner),
            Type::Anchor(inner) => write!(f, "Anchor<{}>", inner),
            Type::CVar(inner) => write!(f, "CVar<{}>", inner),
            Type::Reactiv(inner) => write!(f, "Reactiv<{}>", inner),
            Type::Function(params, ret) => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Closure(params, ret) => {
                write!(f, "closure(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Actor(params, ret) => {
                write!(f, "actor(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Generic(name) => write!(f, "{}", name),
            Type::UserDefined(name) => write!(f, "{}", name),
            Type::Reference(inner) => write!(f, "&{}", inner),
            Type::MutableReference(inner) => write!(f, "&mut {}", inner),
            Type::RawPointer(inner) => write!(f, "*const {}", inner),
        }
    }
}

/// Literal values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Integer(i64),
    UnsignedInteger(u64),
    Float(f64),
    Boolean(bool),
    String(String),
    Char(char),
    Byte(u8),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Integer(i) => write!(f, "{}", i),
            Literal::UnsignedInteger(u) => write!(f, "{}", u),
            Literal::Float(fl) => write!(f, "{}", fl),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Char(c) => write!(f, "'{}'", c),
            Literal::Byte(b) => write!(f, "{}", b),
        }
    }
}

/// Expressions in AFNS
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },
    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Option<Box<Expression>>,
    },
    Match {
        value: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    Block(Vec<Statement>),
    Lambda {
        params: Vec<(String, Type)>,
        body: Box<Expression>,
    },
    Actor {
        params: Vec<(String, Type)>,
        body: Box<Expression>,
    },
}

/// Binary operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Negate,
    Dereference,
    Reference,
    MutableReference,
}

/// Match arm for pattern matching
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Expression,
}

/// Patterns for pattern matching
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    Wildcard,
    Tuple(Vec<Pattern>),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
    Enum {
        name: String,
        variant: String,
        fields: Vec<Pattern>,
    },
}

/// Statements in AFNS
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Expression(Expression),
    VariableDeclaration {
        name: String,
        type_annotation: Option<Type>,
        value: Expression,
    },
    Assignment {
        target: Expression,
        value: Expression,
    },
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    For {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Loop {
        body: Vec<Statement>,
    },
    Break,
    Continue,
    Return(Option<Expression>),
    Match {
        value: Expression,
        arms: Vec<MatchArm>,
    },
    Block(Vec<Statement>),
    Unsafe(Vec<Statement>),
}

/// Function definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub is_public: bool,
    pub span: Span,
}

/// Function overload group for function overloading
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionOverload {
    pub name: String,
    pub overloads: Vec<Function>,
    pub span: Span,
}

impl FunctionOverload {
    /// Create a new function overload group
    pub fn new(name: String, span: Span) -> Self {
        Self {
            name,
            overloads: Vec::new(),
            span,
        }
    }

    /// Add an overload to the group
    pub fn add_overload(&mut self, function: Function) {
        self.overloads.push(function);
    }

    /// Find the best matching overload for given parameter types
    pub fn find_best_match(&self, param_types: &[Type]) -> Option<&Function> {
        // Simple exact match for now
        for overload in &self.overloads {
            if overload.parameters.len() == param_types.len() {
                let mut matches = true;
                for (i, (_, param_type)) in overload.parameters.iter().enumerate() {
                    if param_type != &param_types[i] {
                        matches = false;
                        break;
                    }
                }
                if matches {
                    return Some(overload);
                }
            }
        }
        None
    }

    /// Get all overloads
    pub fn get_overloads(&self) -> &Vec<Function> {
        &self.overloads
    }
}

/// Struct definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub is_public: bool,
    pub span: Span,
}

/// Enum definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub is_public: bool,
    pub span: Span,
}

/// Enum variant
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

/// Implementation block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Implementation {
    pub target: String,
    pub trait_name: Option<String>,
    pub methods: Vec<Function>,
    pub span: Span,
}

/// Trait definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trait {
    pub name: String,
    pub methods: Vec<TraitMethod>,
    pub is_public: bool,
    pub span: Span,
}

/// Trait method signature
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitMethod {
    pub name: String,
    pub parameters: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub is_async: bool,
    pub is_unsafe: bool,
}

/// Module definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub items: Vec<Item>,
    pub is_public: bool,
    pub span: Span,
}

/// Top-level items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Function(Function),
    FunctionOverload(FunctionOverload),
    Struct(Struct),
    Enum(Enum),
    Implementation(Implementation),
    Trait(Trait),
    Module(Module),
    Import(String),
    TypeAlias {
        name: String,
        type_: Type,
    },
}

/// Complete program AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

impl Program {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            span: Span::new(0, 0, 1, 1),
        }
    }
    
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", Type::I32), "i32");
        assert_eq!(format!("{}", Type::String), "string");
        assert_eq!(format!("{}", Type::Array(Box::new(Type::I32))), "Array<i32>");
        assert_eq!(format!("{}", Type::Map(Box::new(Type::String), Box::new(Type::I32))), "Map<string, i32>");
    }
    
    #[test]
    fn test_literal_display() {
        assert_eq!(format!("{}", Literal::Integer(42)), "42");
        assert_eq!(format!("{}", Literal::String("hello".to_string())), "\"hello\"");
        assert_eq!(format!("{}", Literal::Boolean(true)), "true");
    }
    
    #[test]
    fn test_program_creation() {
        let mut program = Program::new();
        let function = Function {
            name: "main".to_string(),
            parameters: Vec::new(),
            return_type: None,
            body: Vec::new(),
            is_async: false,
            is_unsafe: false,
            is_public: false,
            span: Span::new(0, 0, 1, 1),
        };
        
        program.add_item(Item::Function(function));
        assert_eq!(program.items.len(), 1);
    }
}

