use crate::apexlang::ast::{BinaryOp, Expr, Function, Program, Statement, UnaryOp, Value};
use crate::apexlang::error::ApexError;
use crate::apexlang::lexer::{self, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn parse(source: &str) -> Result<Program, ApexError> {
        let tokens = lexer::lex(source)?;
        let mut parser = Parser { tokens, current: 0 };
        parser.parse_program()
    }

    fn parse_program(&mut self) -> Result<Program, ApexError> {
        let mut functions = Vec::new();
        while !self.is_at_end() {
            functions.push(self.parse_function()?);
        }
        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<Function, ApexError> {
        self.consume(TokenKind::Fn, "Expected 'fn' at the start of a function")?;
        let name = self.consume_identifier("Expected function name after 'fn'")?;
        self.consume(TokenKind::LParen, "Expected '(' after function name")?;
        self.consume(TokenKind::RParen, "Expected ')' after function parameters")?;
        self.consume(TokenKind::LBrace, "Expected '{' to start function body")?;

        let mut body = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }

        self.consume(TokenKind::RBrace, "Expected '}' to end function body")?;

        Ok(Function { name, body })
    }

    fn parse_statement(&mut self) -> Result<Statement, ApexError> {
        if self.match_kind(TokenKind::Return) {
            let expr = self.parse_expression()?;
            self.consume(TokenKind::Semicolon, "Expected ';' after return expression")?;
            Ok(Statement::Return(expr))
        } else {
            Err(self.error_here("Only return statements are supported in ApexLang MVP"))
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, ApexError> {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Result<Expr, ApexError> {
        let mut expr = self.parse_factor()?;

        loop {
            if self.match_kind(TokenKind::Plus) {
                let right = self.parse_factor()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Add, Box::new(right));
            } else if self.match_kind(TokenKind::Minus) {
                let right = self.parse_factor()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Sub, Box::new(right));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, ApexError> {
        let mut expr = self.parse_unary()?;

        loop {
            if self.match_kind(TokenKind::Star) {
                let right = self.parse_unary()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Mul, Box::new(right));
            } else if self.match_kind(TokenKind::Slash) {
                let right = self.parse_unary()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Div, Box::new(right));
            } else if self.match_kind(TokenKind::Percent) {
                let right = self.parse_unary()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Mod, Box::new(right));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ApexError> {
        if self.match_kind(TokenKind::Plus) {
            Ok(Expr::Unary(UnaryOp::Plus, Box::new(self.parse_unary()?)))
        } else if self.match_kind(TokenKind::Minus) {
            Ok(Expr::Unary(UnaryOp::Minus, Box::new(self.parse_unary()?)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ApexError> {
        if let Some(token) =
            self.advance_if(|t| matches!(t.kind, TokenKind::Number(_) | TokenKind::Integer(_)))
        {
            return Ok(match token.kind {
                TokenKind::Number(value) => Expr::Literal(Value::Number(value)),
                TokenKind::Integer(value) => Expr::Literal(Value::Int(value)),
                _ => unreachable!(),
            });
        }

        if self.match_kind(TokenKind::LParen) {
            let expr = self.parse_expression()?;
            self.consume(TokenKind::RParen, "Expected ')' after expression")?;
            return Ok(expr);
        }

        Err(self.error_here("Expected expression"))
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<Token, ApexError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(self.error_here(message))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, ApexError> {
        if let Some(token) = self.advance_if(|t| matches!(t.kind, TokenKind::Identifier(_))) {
            if let TokenKind::Identifier(name) = token.kind {
                return Ok(name);
            }
        }
        Err(self.error_here(message))
    }

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.check(&kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return matches!(kind, TokenKind::Eof);
        }
        match (&self.peek().kind, kind) {
            (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,
            (TokenKind::Number(_), TokenKind::Number(_)) => true,
            (TokenKind::Integer(_), TokenKind::Integer(_)) => true,
            (left, right) => left == right,
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
            return self.tokens[self.current - 1].clone();
        }
        self.tokens[self.current].clone()
    }

    fn advance_if<F>(&mut self, predicate: F) -> Option<Token>
    where
        F: Fn(&Token) -> bool,
    {
        if predicate(self.peek()) {
            Some(self.advance())
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn error_here(&self, message: &str) -> ApexError {
        let token = self.peek();
        let details = if matches!(token.kind, TokenKind::Eof) {
            "end of file".to_string()
        } else if token.lexeme.is_empty() {
            token.kind_name()
        } else {
            format!("'{}'", token.lexeme)
        };
        ApexError::new(format!(
            "{} at line {}, column {} near {}",
            message, token.line, token.column, details
        ))
    }
}

trait TokenExt {
    fn kind_name(&self) -> String;
}

impl TokenExt for Token {
    fn kind_name(&self) -> String {
        match &self.kind {
            TokenKind::Fn => "fn".to_string(),
            TokenKind::Return => "return".to_string(),
            TokenKind::Identifier(_) => "identifier".to_string(),
            TokenKind::Integer(_) => "integer".to_string(),
            TokenKind::Number(_) => "number".to_string(),
            TokenKind::LParen => "'('".to_string(),
            TokenKind::RParen => "')'".to_string(),
            TokenKind::LBrace => "'{'".to_string(),
            TokenKind::RBrace => "'}'".to_string(),
            TokenKind::Semicolon => "';'".to_string(),
            TokenKind::Plus => "'+'".to_string(),
            TokenKind::Minus => "'-'".to_string(),
            TokenKind::Star => "'*'".to_string(),
            TokenKind::Slash => "'/'".to_string(),
            TokenKind::Percent => "'%'".to_string(),
            TokenKind::Eof => "end of file".to_string(),
        }
    }
}
