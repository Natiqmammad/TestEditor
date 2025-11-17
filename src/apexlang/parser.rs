use crate::apexlang::ast::{
    BinaryOp, Expr, Function, Import, ImportKind, Path, Program, Statement, UnaryOp, Value,
};
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
        let mut imports = Vec::new();
        while self.match_kind(TokenKind::Import) {
            imports.push(self.parse_import_clause()?);
        }

        let mut functions = Vec::new();
        while !self.is_at_end() {
            functions.push(self.parse_function()?);
        }

        Ok(Program { imports, functions })
    }

    fn parse_import_clause(&mut self) -> Result<Import, ApexError> {
        let module = self.consume_identifier("Expected module name after 'import'")?;

        if self.match_kind(TokenKind::Dot) {
            let symbol = self.consume_identifier("Expected symbol name after '.' in import")?;
            let alias = if self.match_kind(TokenKind::As) {
                Some(self.consume_identifier("Expected alias name after 'as'")?)
            } else {
                None
            };
            self.consume(TokenKind::Semicolon, "Expected ';' after import statement")?;
            Ok(Import {
                kind: ImportKind::Symbol {
                    module,
                    symbol,
                    alias,
                },
            })
        } else {
            let alias = if self.match_kind(TokenKind::As) {
                Some(self.consume_identifier("Expected alias name after 'as'")?)
            } else {
                None
            };
            self.consume(TokenKind::Semicolon, "Expected ';' after import statement")?;
            Ok(Import {
                kind: ImportKind::Module {
                    name: module,
                    alias,
                },
            })
        }
    }

    fn parse_function(&mut self) -> Result<Function, ApexError> {
        self.consume(TokenKind::Fn, "Expected 'fn' at the start of a function")?;
        let name = self.consume_identifier("Expected function name after 'fn'")?;
        self.consume(TokenKind::LParen, "Expected '(' after function name")?;

        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                params.push(self.consume_identifier("Expected parameter name")?);
                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RParen, "Expected ')' after function parameters")?;
        self.consume(TokenKind::LBrace, "Expected '{' to start function body")?;

        let mut body = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }

        self.consume(TokenKind::RBrace, "Expected '}' to end function body")?;

        Ok(Function { name, params, body })
    }

    fn parse_statement(&mut self) -> Result<Statement, ApexError> {
        if self.match_kind(TokenKind::Return) {
            let expr = self.parse_expression()?;
            self.consume(TokenKind::Semicolon, "Expected ';' after return expression")?;
            return Ok(Statement::Return(expr));
        }

        if self.match_kind(TokenKind::Let) {
            return self.parse_variable_declaration(false);
        }

        if self.match_kind(TokenKind::Var) {
            return self.parse_variable_declaration(true);
        }

        if self.is_assignment_start() {
            let name = if let TokenKind::Identifier(name) = self.advance().kind.clone() {
                name
            } else {
                unreachable!("assignment start guarantees identifier");
            };
            self.consume(TokenKind::Equal, "Expected '=' in assignment")?;
            let value = self.parse_expression()?;
            self.consume(TokenKind::Semicolon, "Expected ';' after assignment")?;
            return Ok(Statement::Assignment { name, value });
        }

        let expr = self.parse_expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after expression")?;
        Ok(Statement::Expr(expr))
    }

    fn parse_variable_declaration(&mut self, mutable: bool) -> Result<Statement, ApexError> {
        let name = self.consume_identifier("Expected variable name")?;
        self.consume(TokenKind::Equal, "Expected '=' after variable name")?;
        let value = self.parse_expression()?;
        self.consume(
            TokenKind::Semicolon,
            "Expected ';' after variable declaration",
        )?;
        Ok(Statement::Let {
            name,
            mutable,
            value,
        })
    }

    fn parse_expression(&mut self) -> Result<Expr, ApexError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, ApexError> {
        let mut expr = self.parse_and()?;

        while self.match_kind(TokenKind::OrOr) {
            let right = self.parse_and()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::Or, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, ApexError> {
        let mut expr = self.parse_equality()?;

        while self.match_kind(TokenKind::AndAnd) {
            let right = self.parse_equality()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::And, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, ApexError> {
        let mut expr = self.parse_comparison()?;

        loop {
            if self.match_kind(TokenKind::EqualEqual) {
                let right = self.parse_comparison()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Eq, Box::new(right));
            } else if self.match_kind(TokenKind::BangEqual) {
                let right = self.parse_comparison()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Ne, Box::new(right));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ApexError> {
        let mut expr = self.parse_term()?;

        loop {
            if self.match_kind(TokenKind::Less) {
                let right = self.parse_term()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Lt, Box::new(right));
            } else if self.match_kind(TokenKind::LessEqual) {
                let right = self.parse_term()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Le, Box::new(right));
            } else if self.match_kind(TokenKind::Greater) {
                let right = self.parse_term()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Gt, Box::new(right));
            } else if self.match_kind(TokenKind::GreaterEqual) {
                let right = self.parse_term()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Ge, Box::new(right));
            } else {
                break;
            }
        }

        Ok(expr)
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
        if self.match_kind(TokenKind::Bang) {
            return Ok(Expr::Unary(UnaryOp::Not, Box::new(self.parse_unary()?)));
        }
        if self.match_kind(TokenKind::Plus) {
            return Ok(Expr::Unary(UnaryOp::Plus, Box::new(self.parse_unary()?)));
        }
        if self.match_kind(TokenKind::Minus) {
            return Ok(Expr::Unary(UnaryOp::Minus, Box::new(self.parse_unary()?)));
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expr, ApexError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_kind(TokenKind::LParen) {
                let mut arguments = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RParen, "Expected ')' after arguments")?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    arguments,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ApexError> {
        if let Some(token) = self.advance_if(|t| {
            matches!(
                t.kind,
                TokenKind::Number(_) | TokenKind::Integer(_) | TokenKind::StringLiteral(_)
            )
        }) {
            return Ok(match token.kind {
                TokenKind::Number(value) => Expr::Literal(Value::Number(value)),
                TokenKind::Integer(value) => Expr::Literal(Value::Int(value)),
                TokenKind::StringLiteral(value) => Expr::Literal(Value::String(value)),
                _ => unreachable!(),
            });
        }

        if self.match_kind(TokenKind::True) {
            return Ok(Expr::Literal(Value::Bool(true)));
        }

        if self.match_kind(TokenKind::False) {
            return Ok(Expr::Literal(Value::Bool(false)));
        }

        if self.match_kind(TokenKind::LParen) {
            let expr = self.parse_expression()?;
            self.consume(TokenKind::RParen, "Expected ')' after expression")?;
            return Ok(expr);
        }

        if let Some(ident) = self.match_identifier() {
            let mut segments = vec![ident];
            while self.match_kind(TokenKind::Dot) {
                segments.push(self.consume_identifier("Expected identifier after '.'")?);
            }
            return Ok(Expr::Path(Path::join(segments)));
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

    fn match_identifier(&mut self) -> Option<String> {
        if let TokenKind::Identifier(name) = &self.peek().kind {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        }
    }

    fn is_assignment_start(&self) -> bool {
        if self.is_at_end() {
            return false;
        }
        matches!(self.peek().kind, TokenKind::Identifier(_))
            && matches!(self.peek_next().map(|t| &t.kind), Some(TokenKind::Equal))
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

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1)
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
            TokenKind::Let => "let".to_string(),
            TokenKind::Var => "var".to_string(),
            TokenKind::Import => "import".to_string(),
            TokenKind::As => "as".to_string(),
            TokenKind::True => "true".to_string(),
            TokenKind::False => "false".to_string(),
            TokenKind::Identifier(_) => "identifier".to_string(),
            TokenKind::Integer(_) => "integer".to_string(),
            TokenKind::Number(_) => "number".to_string(),
            TokenKind::StringLiteral(_) => "string literal".to_string(),
            TokenKind::LParen => "'('".to_string(),
            TokenKind::RParen => "')'".to_string(),
            TokenKind::LBrace => "'{'".to_string(),
            TokenKind::RBrace => "'}'".to_string(),
            TokenKind::Comma => "','".to_string(),
            TokenKind::Semicolon => "';'".to_string(),
            TokenKind::Dot => "'.'".to_string(),
            TokenKind::Plus => "'+'".to_string(),
            TokenKind::Minus => "'-'".to_string(),
            TokenKind::Star => "'*'".to_string(),
            TokenKind::Slash => "'/'".to_string(),
            TokenKind::Percent => "'%'".to_string(),
            TokenKind::Bang => "'!'".to_string(),
            TokenKind::Equal => "'='".to_string(),
            TokenKind::EqualEqual => "'=='".to_string(),
            TokenKind::BangEqual => "'!='".to_string(),
            TokenKind::Less => "'<'".to_string(),
            TokenKind::LessEqual => "'<='".to_string(),
            TokenKind::Greater => "'>'".to_string(),
            TokenKind::GreaterEqual => "'>='".to_string(),
            TokenKind::AndAnd => "'&&'".to_string(),
            TokenKind::OrOr => "'||'".to_string(),
            TokenKind::Eof => "end of file".to_string(),
        }
    }
}
