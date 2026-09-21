use crate::ast::Expr;
use crate::token::{Token, TokenType};
use std::fmt::{self, Display, Formatter};

// A syntax error encountered while parsing.
pub struct ParseError {
    pub line: u32,
    pub message: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error at line {}: {}", self.line, self.message)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    // Parses a single expression.
    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expression()
    }

    // expression -> term
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.term()
    }

    // term -> primary ( ( "+" | "-" | "++" ) primary )*
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            let is_operator = matches!(
                self.peek().token_type(),
                TokenType::Add | TokenType::Subtract | TokenType::StringConcat
            );
            if !is_operator {
                break;
            }

            let operator = self.advance().clone();
            let right = self.primary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    // primary -> INTEGER | FLOAT | STRING | "true" | "false" | "null"
    fn primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.peek().clone();

        let is_literal = matches!(
            token.token_type(),
            TokenType::Integer(_)
                | TokenType::Float(_)
                | TokenType::String(_)
                | TokenType::True
                | TokenType::False
                | TokenType::Null
        );

        if is_literal {
            self.advance();
            return Ok(Expr::Literal(token));
        }

        Err(self.error(token, "Expect expression."))
    }

    // peeks at the current token without consuming it
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    // consumes and returns the current token
    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.current];
        if !self.is_at_end() {
            self.current += 1;
        }
        token
    }

    // True once `current` has reached the EOF token
    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type(), TokenType::Eof)
    }

    // Builds a syntax error pointing at the given token
    fn error(&self, token: Token, message: &str) -> ParseError {
        ParseError {
            line: token.line(),
            message: message.to_string(),
        }
    }
}
