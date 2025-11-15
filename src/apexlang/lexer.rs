use crate::apexlang::error::ApexError;

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
    Identifier(String),
    Number(f64),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Plus,
    Minus,
    Star,
    Slash,
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
            }
            '(' => {
                tokens.push(Token {
                    kind: TokenKind::LParen,
                    lexeme: ch.to_string(),
                    line,
                    column,
                });
                i += 1;
                column += 1;
            }
            ')' => {
                tokens.push(Token {
                    kind: TokenKind::RParen,
                    lexeme: ch.to_string(),
                    line,
                    column,
                });
                i += 1;
                column += 1;
            }
            '{' => {
                tokens.push(Token {
                    kind: TokenKind::LBrace,
                    lexeme: ch.to_string(),
                    line,
                    column,
                });
                i += 1;
                column += 1;
            }
            '}' => {
                tokens.push(Token {
                    kind: TokenKind::RBrace,
                    lexeme: ch.to_string(),
                    line,
                    column,
                });
                i += 1;
                column += 1;
            }
            ';' => {
                tokens.push(Token {
                    kind: TokenKind::Semicolon,
                    lexeme: ch.to_string(),
                    line,
                    column,
                });
                i += 1;
                column += 1;
            }
            '+' => {
                tokens.push(Token {
                    kind: TokenKind::Plus,
                    lexeme: ch.to_string(),
                    line,
                    column,
                });
                i += 1;
                column += 1;
            }
            '-' => {
                tokens.push(Token {
                    kind: TokenKind::Minus,
                    lexeme: ch.to_string(),
                    line,
                    column,
                });
                i += 1;
                column += 1;
            }
            '*' => {
                tokens.push(Token {
                    kind: TokenKind::Star,
                    lexeme: ch.to_string(),
                    line,
                    column,
                });
                i += 1;
                column += 1;
            }
            '/' => {
                // Handle comments starting with //
                if i + 1 < chars.len() && chars[i + 1] == '/' {
                    // consume until newline or end
                    i += 2;
                    column += 2;
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                        column += 1;
                    }
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Slash,
                        lexeme: ch.to_string(),
                        line,
                        column,
                    });
                    i += 1;
                    column += 1;
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
