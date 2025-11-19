use crate::apexlang::error::ApexError;
use num_bigint::BigInt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Fn,
    Return,
    Let,
    Var,
    Import,
    As,
    True,
    False,
    Identifier(String),
    Integer(BigInt),
    Number(f64),
    StringLiteral(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Dot,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    AndAnd,
    OrOr,
    Eof,
}

pub fn lex(source: &str) -> Result<Vec<Token>, ApexError> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    let mut line = 1;
    let mut column = 1;

    while i < chars.len() {
        let ch = chars[i];
        match ch {
            ' ' | '\t' | '\r' => {
                i += 1;
                column += 1;
            }
            '\n' => {
                i += 1;
                line += 1;
                column = 1;
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start_col = column;
                let mut ident = String::new();
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    ident.push(chars[i]);
                    i += 1;
                    column += 1;
                }

                let kind = match ident.as_str() {
                    "fn" => TokenKind::Fn,
                    "return" => TokenKind::Return,
                    "let" => TokenKind::Let,
                    "var" => TokenKind::Var,
                    "import" => TokenKind::Import,
                    "as" => TokenKind::As,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    _ => TokenKind::Identifier(ident.clone()),
                };

                tokens.push(Token {
                    kind,
                    lexeme: ident,
                    line,
                    column: start_col,
                });
            }
            '0'..='9' => {
                let start_col = column;
                let mut number = String::new();
                let mut has_dot = false;
                while i < chars.len() {
                    let c = chars[i];
                    if c.is_ascii_digit() {
                        number.push(c);
                        i += 1;
                        column += 1;
                    } else if c == '.' && !has_dot {
                        has_dot = true;
                        number.push(c);
                        i += 1;
                        column += 1;
                    } else {
                        break;
                    }
                }

                if has_dot {
                    let value: f64 = number.parse().map_err(|_| {
                        ApexError::new(format!(
                            "Invalid number literal '{}' at line {}, column {}",
                            number, line, start_col
                        ))
                    })?;

                    tokens.push(Token {
                        kind: TokenKind::Number(value),
                        lexeme: number,
                        line,
                        column: start_col,
                    });
                } else {
                    let value = BigInt::parse_bytes(number.as_bytes(), 10).ok_or_else(|| {
                        ApexError::new(format!(
                            "Invalid integer literal '{}' at line {}, column {}",
                            number, line, start_col
                        ))
                    })?;

                    tokens.push(Token {
                        kind: TokenKind::Integer(value),
                        lexeme: number,
                        line,
                        column: start_col,
                    });
                }
            }
            '"' => {
                let start_col = column;
                i += 1;
                column += 1;
                let mut value = String::new();
                let mut terminated = false;
                while i < chars.len() {
                    let c = chars[i];
                    match c {
                        '"' => {
                            terminated = true;
                            i += 1;
                            column += 1;
                            break;
                        }
                        '\\' => {
                            if i + 1 >= chars.len() {
                                return Err(ApexError::new(format!(
                                    "Unterminated string literal at line {}, column {}",
                                    line, start_col
                                )));
                            }
                            let escape = chars[i + 1];
                            let translated = match escape {
                                '"' => '"',
                                '\\' => '\\',
                                'n' => '\n',
                                't' => '\t',
                                'r' => '\r',
                                '0' => '\0',
                                other => {
                                    return Err(ApexError::new(format!(
                                        "Unsupported escape '\\{}' at line {}, column {}",
                                        other, line, column
                                    )));
                                }
                            };
                            value.push(translated);
                            i += 2;
                            column += 2;
                        }
                        '\n' => {
                            return Err(ApexError::new(format!(
                                "Unterminated string literal at line {}, column {}",
                                line, start_col
                            )));
                        }
                        other => {
                            value.push(other);
                            i += 1;
                            column += 1;
                        }
                    }
                }
                if !terminated {
                    return Err(ApexError::new(format!(
                        "Unterminated string literal at line {}, column {}",
                        line, start_col
                    )));
                }
                tokens.push(Token {
                    kind: TokenKind::StringLiteral(value.clone()),
                    lexeme: value,
                    line,
                    column: start_col,
                });
            }
            '(' => {
                tokens.push(simple_token(TokenKind::LParen, ch, line, column));
                i += 1;
                column += 1;
            }
            ')' => {
                tokens.push(simple_token(TokenKind::RParen, ch, line, column));
                i += 1;
                column += 1;
            }
            '{' => {
                tokens.push(simple_token(TokenKind::LBrace, ch, line, column));
                i += 1;
                column += 1;
            }
            '}' => {
                tokens.push(simple_token(TokenKind::RBrace, ch, line, column));
                i += 1;
                column += 1;
            }
            ';' => {
                tokens.push(simple_token(TokenKind::Semicolon, ch, line, column));
                i += 1;
                column += 1;
            }
            ',' => {
                tokens.push(simple_token(TokenKind::Comma, ch, line, column));
                i += 1;
                column += 1;
            }
            '.' => {
                tokens.push(simple_token(TokenKind::Dot, ch, line, column));
                i += 1;
                column += 1;
            }
            '+' => {
                tokens.push(simple_token(TokenKind::Plus, ch, line, column));
                i += 1;
                column += 1;
            }
            '-' => {
                tokens.push(simple_token(TokenKind::Minus, ch, line, column));
                i += 1;
                column += 1;
            }
            '*' => {
                tokens.push(simple_token(TokenKind::Star, ch, line, column));
                i += 1;
                column += 1;
            }
            '/' => {
                if i + 1 < chars.len() && chars[i + 1] == '/' {
                    i += 2;
                    column += 2;
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                        column += 1;
                    }
                } else {
                    tokens.push(simple_token(TokenKind::Slash, ch, line, column));
                    i += 1;
                    column += 1;
                }
            }
            '%' => {
                tokens.push(simple_token(TokenKind::Percent, ch, line, column));
                i += 1;
                column += 1;
            }
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token {
                        kind: TokenKind::BangEqual,
                        lexeme: "!=".to_string(),
                        line,
                        column,
                    });
                    i += 2;
                    column += 2;
                } else {
                    tokens.push(simple_token(TokenKind::Bang, ch, line, column));
                    i += 1;
                    column += 1;
                }
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token {
                        kind: TokenKind::EqualEqual,
                        lexeme: "==".to_string(),
                        line,
                        column,
                    });
                    i += 2;
                    column += 2;
                } else {
                    tokens.push(simple_token(TokenKind::Equal, ch, line, column));
                    i += 1;
                    column += 1;
                }
            }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token {
                        kind: TokenKind::LessEqual,
                        lexeme: "<=".to_string(),
                        line,
                        column,
                    });
                    i += 2;
                    column += 2;
                } else {
                    tokens.push(simple_token(TokenKind::Less, ch, line, column));
                    i += 1;
                    column += 1;
                }
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token {
                        kind: TokenKind::GreaterEqual,
                        lexeme: ">=".to_string(),
                        line,
                        column,
                    });
                    i += 2;
                    column += 2;
                } else {
                    tokens.push(simple_token(TokenKind::Greater, ch, line, column));
                    i += 1;
                    column += 1;
                }
            }
            '&' => {
                if i + 1 < chars.len() && chars[i + 1] == '&' {
                    tokens.push(Token {
                        kind: TokenKind::AndAnd,
                        lexeme: "&&".to_string(),
                        line,
                        column,
                    });
                    i += 2;
                    column += 2;
                } else {
                    return Err(ApexError::new(format!(
                        "Unexpected character '&' at line {}, column {}",
                        line, column
                    )));
                }
            }
            '|' => {
                if i + 1 < chars.len() && chars[i + 1] == '|' {
                    tokens.push(Token {
                        kind: TokenKind::OrOr,
                        lexeme: "||".to_string(),
                        line,
                        column,
                    });
                    i += 2;
                    column += 2;
                } else {
                    return Err(ApexError::new(format!(
                        "Unexpected character '|' at line {}, column {}",
                        line, column
                    )));
                }
            }
            _ => {
                return Err(ApexError::new(format!(
                    "Unexpected character '{}' at line {}, column {}",
                    ch, line, column
                )))
            }
        }
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        lexeme: String::new(),
        line,
        column,
    });

    Ok(tokens)
}

fn simple_token(kind: TokenKind, ch: char, line: usize, column: usize) -> Token {
    Token {
        kind,
        lexeme: ch.to_string(),
        line,
        column,
    }
}
