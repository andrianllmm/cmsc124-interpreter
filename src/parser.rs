//! Recursive descent parser that turns tokens into expressions.
//!
//! Each rule method follows its rule from the grammar in the README.

use crate::ast::Expr;
use crate::token::{Token, TokenType};
use std::fmt::{self, Display, Formatter};

/// A syntax error encountered while parsing.
pub struct ParseError {
    pub line: u32,
    pub message: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error at line {}: {}", self.line, self.message)
    }
}

/// Parses the scanner's tokens, which must end with `Eof`.
pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    expressions: Vec<Expr<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Parser<'a> {
        Parser {
            tokens,
            expressions: Vec::new(),
            current: 0,
        }
    }

    /// Parses each `;`-terminated expression, stopping at the first error.
    pub fn parse(&mut self) -> Result<&Vec<Expr<'a>>, ParseError> {
        while !self.is_at_end() {
            let expr: Expr<'_> = self.expression()?;
            let is_terminator: bool = matches!(self.peek().token_type(), TokenType::Terminator);

            if is_terminator {
                self.expressions.push(expr);
                self.advance();
            } else {
                return Err(self.error(self.peek().clone(), "Missing terminator ';'"));
            }
        }

        Ok(&self.expressions)
    }

    /// `expression → assignment`
    fn expression(&mut self) -> Result<Expr<'a>, ParseError> {
        self.assignment()
    }

    /// `assignment → IDENTIFIER ( "=" | "+=" | "-=" | "*=" | "/=" | "|=" ) assignment
    /// | pipe`
    fn assignment(&mut self) -> Result<Expr<'a>, ParseError> {
        // TODO: Parse assignments once identifiers are supported.
        self.pipe()
    }

    /// `pipe → logicOr ( "|>" logicOr )*`
    fn pipe(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr: Expr<'_> = self.logic_or()?;

        loop {
            let is_pipe: bool = matches!(self.peek().token_type(), TokenType::Pipe,);

            if !is_pipe {
                break;
            }

            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.logic_or()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    /// `logicOr → logicAnd ( "or" logicAnd )*`
    fn logic_or(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr: Expr<'_> = self.logic_and()?;

        loop {
            let is_or: bool = matches!(self.peek().token_type(), TokenType::Or);

            if !is_or {
                break;
            }

            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.logic_and()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    /// `logicAnd → logicNot ( "and" logicNot )*`
    fn logic_and(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr: Expr<'_> = self.logic_not()?;

        loop {
            let is_and: bool = matches!(self.peek().token_type(), TokenType::And);

            if !is_and {
                break;
            }

            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.logic_not()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    /// `logicNot → "not" logicNot | comparison`
    fn logic_not(&mut self) -> Result<Expr<'a>, ParseError> {
        let is_logic_not: bool = matches!(self.peek().token_type(), TokenType::Not);

        if is_logic_not {
            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.logic_not()?;
            let expr: Expr<'_> = Expr::Unary(operator, Box::new(right));
            Ok(expr)
        } else {
            self.comparison()
        }
    }

    /// `comparison → term ( ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) term )*`
    fn comparison(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr: Expr<'_> = self.term()?;

        loop {
            let is_operator: bool = matches!(
                self.peek().token_type(),
                TokenType::Equals
                    | TokenType::NotEquals
                    | TokenType::Less
                    | TokenType::LessEquals
                    | TokenType::More
                    | TokenType::MoreEquals
            );

            if !is_operator {
                break;
            }

            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    /// `term → factor ( ( "+" | "-" | "++" ) factor )*`
    fn term(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr = self.factor()?;

        loop {
            let is_operator = matches!(
                self.peek().token_type(),
                TokenType::Add | TokenType::Subtract | TokenType::StringConcat
            );
            if !is_operator {
                break;
            }

            let operator = self.advance().clone();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    /// `factor → unary ( ( "*" | "/" | "%" ) unary )*`
    fn factor(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr: Expr<'_> = self.unary()?;

        loop {
            let is_operator: bool = matches!(
                self.peek().token_type(),
                TokenType::Multiply | TokenType::Divide | TokenType::Modulo
            );

            if !is_operator {
                break;
            }

            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    /// `unary → "-" unary | exponent`
    fn unary(&mut self) -> Result<Expr<'a>, ParseError> {
        let is_unary_operator: bool = matches!(self.peek().token_type(), TokenType::Subtract);

        if is_unary_operator {
            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.unary()?;
            let expr: Expr<'_> = Expr::Unary(operator, Box::new(right));
            Ok(expr)
        } else {
            self.exponent()
        }
    }

    /// `exponent → primary ( "^" exponent )?`
    fn exponent(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr: Expr<'_> = self.primary()?;

        let is_exponent: bool = matches!(self.peek().token_type(), TokenType::Exponent);

        if is_exponent {
            let exponent: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.exponent()?;
            expr = Expr::Binary(Box::new(expr), exponent, Box::new(right));
        }

        Ok(expr)
    }

    /// `primary → INTEGER | FLOAT | STRING | "true" | "false" | "null"
    /// | IDENTIFIER | "(" expression ")"`
    fn primary(&mut self) -> Result<Expr<'a>, ParseError> {
        // TODO: Parse IDENTIFIER.
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

        if matches!(token.token_type(), TokenType::LParen) {
            self.advance();
            let expr: Expr<'_> = self.expression()?;

            if !matches!(self.peek().token_type(), TokenType::RParen) {
                return Err(self.error(token, "Missing ')'"));
            } else {
                self.advance();
                return Ok(Expr::Grouping(Box::new(expr)));
            }
        }

        Err(self.error(token, "Expect expression."))
    }

    /// Returns the current token without consuming it.
    fn peek(&self) -> &Token<'a> {
        &self.tokens[self.current]
    }

    /// Consumes and returns the current token.
    ///
    /// Stays on EOF so `peek` never goes out of bounds.
    fn advance(&mut self) -> &Token<'a> {
        let token = &self.tokens[self.current];
        if !self.is_at_end() {
            self.current += 1;
        }
        token
    }

    /// Returns `true` once `current` has reached the EOF token.
    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type(), TokenType::Eof)
    }

    /// Builds a syntax error pointing at the given token.
    fn error(&self, token: Token<'a>, message: &str) -> ParseError {
        ParseError {
            line: token.line(),
            message: message.to_string(),
        }
    }
}
