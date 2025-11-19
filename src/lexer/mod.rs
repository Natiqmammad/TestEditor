//! Lexer module for ApexForge NightScript
//!
//! This module provides lexical analysis for AFNS source code,
//! tokenizing the unique syntax with keywords like `fun`, `apex`, `var`, `::`, `check`, `import`.

use logos::Logos;
use std::fmt;

/// Token types for ApexForge NightScript
#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords - AFNS specific
    #[token("fun")]
    Fun,

    #[token("apex")]
    Apex,

    #[token("var")]
    Var,

    #[token("check")]
    Check,

    #[token("in")]
    In,

    #[token("_", priority = 1)]
    Wildcard,

    #[token("import")]
    Import,

    #[token("struct")]
    Struct,

    #[token("enum")]
    Enum,

    #[token("impl")]
    Impl,

    #[token("trait")]
    Trait,

    #[token("mod")]
    Mod,

    #[token("pub")]
    Pub,

    #[token("priv")]
    Priv,

    #[token("unsafe")]
    Unsafe,

    #[token("async")]
    Async,

    #[token("await")]
    Await,

    #[token("actor")]
    Actor,

    #[token("lambda")]
    Lambda,

    #[token("define")]
    Define,

    #[token("type")]
    Type,

    #[token("typedef")]
    Typedef,

    // Control flow
    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("while")]
    While,

    #[token("for")]
    For,

    #[token("loop")]
    Loop,

    #[token("break")]
    Break,

    #[token("continue")]
    Continue,

    #[token("return")]
    Return,

    #[token("match")]
    Match,

    // Type annotations - AFNS uses :: instead of :
    #[token("::")]
    TypeAnnotation,

    // Operators
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("=")]
    Assign,

    #[token("==")]
    Equal,

    #[token("!=")]
    NotEqual,

    #[token("<")]
    Less,

    #[token(">")]
    Greater,

    #[token("<=")]
    LessEqual,

    #[token(">=")]
    GreaterEqual,

    #[token("&&")]
    And,

    #[token("||")]
    Or,

    #[token("!")]
    Not,

    #[token("&")]
    BitAnd,

    #[token("|")]
    BitOr,

    #[token("^")]
    BitXor,

    #[token("<<")]
    LeftShift,

    #[token(">>")]
    RightShift,

    // Delimiters
    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    #[token("{")]
    LeftBrace,

    #[token("}")]
    RightBrace,

    #[token(",")]
    Comma,

    #[token(";")]
    Semicolon,

    #[token(".")]
    Dot,

    #[token("->")]
    Arrow,

    #[token("=>")]
    FatArrow,

    #[token("?")]
    Question,

    #[token(":")]
    Colon,

    // Literals
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice().trim_matches('"').to_string())]
    StringLiteral(String),

    #[regex(r"'([^'\\]|\\.)'", |lex| lex.slice().trim_matches('\'').chars().next().unwrap_or('\0'))]
    CharLiteral(char),

    #[regex(r"[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?", |lex| lex.slice().to_string())]
    NumberLiteral(String),

    #[regex(r"true|false", |lex| lex.slice() == "true")]
    BooleanLiteral(bool),

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Comments
    #[regex(r"//[^\n]*")]
    LineComment,

    #[regex(r"/\*([^*]|\*[^/])*\*/")]
    BlockComment,

    // Whitespace (filtered out by the lexer helpers)
    #[regex(r"[ \t\n\r]+")]
    Whitespace,
    // Error token (removed - no longer needed in newer logos versions)
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Fun => write!(f, "fun"),
            Token::Apex => write!(f, "apex"),
            Token::Var => write!(f, "var"),
            Token::Check => write!(f, "check"),
            Token::Import => write!(f, "import"),
            Token::TypeAnnotation => write!(f, "::"),
            Token::StringLiteral(s) => write!(f, "\"{}\"", s),
            Token::CharLiteral(c) => write!(f, "'{}'", c),
            Token::NumberLiteral(n) => write!(f, "{}", n),
            Token::BooleanLiteral(b) => write!(f, "{}", b),
            Token::Identifier(i) => write!(f, "{}", i),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Lexer for ApexForge NightScript
pub struct Lexer<'a> {
    inner: logos::Lexer<'a, Token>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: Token::lexer(source),
        }
    }

    fn next_token_internal(&mut self) -> Option<Token> {
        while let Some(result) = self.inner.next() {
            match result {
                Ok(Token::Whitespace) | Ok(Token::LineComment) | Ok(Token::BlockComment) => {
                    continue;
                }
                Ok(token) => return Some(token),
                Err(_) => continue,
            }
        }
        None
    }

    /// Get the next token from the source
    pub fn next(&mut self) -> Option<Token> {
        self.next_token_internal()
    }

    /// Get the current token span
    pub fn span(&self) -> std::ops::Range<usize> {
        self.inner.span()
    }

    /// Get the source slice for the current token
    pub fn slice(&self) -> &'a str {
        self.inner.slice()
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token_internal()
    }
}

/// Token with position information
#[derive(Debug, Clone)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: std::ops::Range<usize>,
    pub line: usize,
    pub column: usize,
}

/// Enhanced lexer that provides position information
pub struct PositionalLexer<'a> {
    inner: logos::Lexer<'a, Token>,
    source: &'a str,
    line: usize,
    column: usize,
    last_pos: usize,
}

impl<'a> PositionalLexer<'a> {
    /// Create a new positional lexer
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: Token::lexer(source),
            source,
            line: 1,
            column: 1,
            last_pos: 0,
        }
    }

    /// Get the next token with position information
    fn next_token_internal(&mut self) -> Option<TokenWithSpan> {
        while let Some(result) = self.inner.next() {
            let span = self.inner.span();

            // Account for the text between the previous token and this one
            let prefix = &self.source[self.last_pos..span.start];
            for ch in prefix.chars() {
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
            }

            self.last_pos = span.start;

            match result {
                Ok(Token::Whitespace) | Ok(Token::LineComment) | Ok(Token::BlockComment) => {
                    let slice = &self.source[span.start..span.end];
                    for ch in slice.chars() {
                        if ch == '\n' {
                            self.line += 1;
                            self.column = 1;
                        } else {
                            self.column += 1;
                        }
                    }
                    self.last_pos = span.end;
                    continue;
                }
                Ok(token) => {
                    let start_line = self.line;
                    let start_column = self.column;

                    let slice = &self.source[span.start..span.end];
                    for ch in slice.chars() {
                        if ch == '\n' {
                            self.line += 1;
                            self.column = 1;
                        } else {
                            self.column += 1;
                        }
                    }
                    self.last_pos = span.end;

                    return Some(TokenWithSpan {
                        token,
                        span,
                        line: start_line,
                        column: start_column,
                    });
                }
                Err(_) => {
                    let slice = &self.source[span.start..span.end];
                    for ch in slice.chars() {
                        if ch == '\n' {
                            self.line += 1;
                            self.column = 1;
                        } else {
                            self.column += 1;
                        }
                    }
                    self.last_pos = span.end;
                }
            }
        }

        None
    }

    pub fn next(&mut self) -> Option<TokenWithSpan> {
        self.next_token_internal()
    }
}

impl<'a> Iterator for PositionalLexer<'a> {
    type Item = TokenWithSpan;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token_internal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let source = "fun apex() { var x::i32 = 42; }";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next(), Some(Token::Fun));
        assert_eq!(lexer.next(), Some(Token::Apex));
        assert_eq!(lexer.next(), Some(Token::LeftParen));
        assert_eq!(lexer.next(), Some(Token::RightParen));
        assert_eq!(lexer.next(), Some(Token::LeftBrace));
        assert_eq!(lexer.next(), Some(Token::Var));
        assert_eq!(lexer.next(), Some(Token::Identifier("x".to_string())));
        assert_eq!(lexer.next(), Some(Token::TypeAnnotation));
        assert_eq!(lexer.next(), Some(Token::Identifier("i32".to_string())));
        assert_eq!(lexer.next(), Some(Token::Assign));
        assert_eq!(lexer.next(), Some(Token::NumberLiteral("42".to_string())));
        assert_eq!(lexer.next(), Some(Token::Semicolon));
        assert_eq!(lexer.next(), Some(Token::RightBrace));
    }

    #[test]
    fn test_string_literals() {
        let source = r#""Hello, ApexForge!""#;
        let mut lexer = Lexer::new(source);

        let token = lexer.next().unwrap();
        match token {
            Token::StringLiteral(s) => assert_eq!(s, "Hello, ApexForge!"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_type_annotation() {
        let source = "var name::string = \"test\";";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next(), Some(Token::Var));
        assert_eq!(lexer.next(), Some(Token::Identifier("name".to_string())));
        assert_eq!(lexer.next(), Some(Token::TypeAnnotation));
        assert_eq!(lexer.next(), Some(Token::Identifier("string".to_string())));
        assert_eq!(lexer.next(), Some(Token::Assign));
    }

    #[test]
    fn test_positional_lexer() {
        let source = "fun test() {\n    var x::i32 = 42;\n}";
        let mut lexer = PositionalLexer::new(source);

        let token = lexer.next().unwrap();
        assert_eq!(token.token, Token::Fun);
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 1);

        // Skip to the var declaration
        while let Some(t) = lexer.next() {
            if matches!(t.token, Token::Var) {
                assert_eq!(t.line, 2);
                assert_eq!(t.column, 5);
                break;
            }
        }
    }
}
