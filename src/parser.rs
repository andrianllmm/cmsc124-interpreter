//! Recursive descent parser that turns tokens into expressions.
//!
//! Each rule method follows its rule from the grammar in the README.

use crate::ast::{Expr, LiteralValue};
use crate::token::{Token, TokenType};
use std::fmt::{self, Display, Formatter};
use std::mem::discriminant;

/// A syntax error encountered while parsing.
pub struct ParseError {
    pub line: u32,
    pub kind: ParseErrorKind,
    /// Lexeme of the token where parsing failed, or `None` at end of file.
    pub found: Option<String>,
}

/// What went wrong while parsing.
pub enum ParseErrorKind {
    /// An expression isn't followed by `;`.
    MissingTerminator,
    /// A `(` has no matching `)`.
    UnclosedParen,
    /// A token that can't start an expression, e.g. `*` or `;`.
    UnexpectedToken,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            ParseErrorKind::MissingTerminator => "Missing ';'",
            ParseErrorKind::UnclosedParen => "Missing ')'",
            ParseErrorKind::UnexpectedToken => "Missing expression",
        };
        let location = match &self.found {
            Some(lexeme) => format!("'{}'", lexeme),
            None => "end".to_string(),
        };
        write!(
            f,
            "Error at line {}: {} at {}",
            self.line, message, location
        )
    }
}

/// Parses the scanner's tokens, which must end with `Eof`.
pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    expressions: Vec<Expr<'a>>,
    errors: Vec<ParseError>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Parser<'a> {
        Parser {
            tokens,
            expressions: Vec::new(),
            errors: Vec::new(),
            current: 0,
        }
    }

    /// Parses each `;`-terminated expression.
    ///
    /// On a syntax error, skips to the next `;` and keeps going,
    /// so every error in the source is reported at once.
    pub fn parse(&mut self) -> Result<&Vec<Expr<'a>>, &Vec<ParseError>> {
        while !self.is_at_end() {
            match self.expr_stmt() {
                Ok(expr) => self.expressions.push(expr),
                Err(error) => {
                    self.errors.push(error);
                    self.synchronize();
                }
            }
        }

        if self.errors.is_empty() {
            Ok(&self.expressions)
        } else {
            Err(&self.errors)
        }
    }

    /// `exprStmt → expression ";"`
    fn expr_stmt(&mut self) -> Result<Expr<'a>, ParseError> {
        let expr = self.expression()?;
        self.consume(&TokenType::Terminator, ParseErrorKind::MissingTerminator)?;
        Ok(expr)
    }

    /// Discards tokens up to and including the next `;`,
    /// so parsing resumes at the start of the next expression.
    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if matches!(self.advance().token_type(), TokenType::Terminator) {
                return;
            }
        }
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
        let mut expr = self.logic_or()?;

        while let Some(operator) = self.match_token(&[TokenType::Pipe]) {
            let right = self.logic_or()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// `logicOr → logicAnd ( "or" logicAnd )*`
    fn logic_or(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr = self.logic_and()?;

        while let Some(operator) = self.match_token(&[TokenType::Or]) {
            let right = self.logic_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// `logicAnd → logicNot ( "and" logicNot )*`
    fn logic_and(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr = self.logic_not()?;

        while let Some(operator) = self.match_token(&[TokenType::And]) {
            let right = self.logic_not()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// `logicNot → "not" logicNot | comparison`
    fn logic_not(&mut self) -> Result<Expr<'a>, ParseError> {
        if let Some(operator) = self.match_token(&[TokenType::Not]) {
            let right = self.logic_not()?;
            let expr = Expr::Unary {
                operator,
                right: Box::new(right),
            };
            Ok(expr)
        } else {
            self.comparison()
        }
    }

    /// `comparison → term ( ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) term )*`
    fn comparison(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr = self.term()?;

        while let Some(operator) = self.match_token(&[
            TokenType::Equals,
            TokenType::NotEquals,
            TokenType::Less,
            TokenType::LessEquals,
            TokenType::More,
            TokenType::MoreEquals,
        ]) {
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// `term → factor ( ( "+" | "-" | "++" ) factor )*`
    fn term(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr = self.factor()?;

        while let Some(operator) =
            self.match_token(&[TokenType::Add, TokenType::Subtract, TokenType::StringConcat])
        {
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// `factor → unary ( ( "*" | "/" | "%" ) unary )*`
    fn factor(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr = self.unary()?;

        while let Some(operator) =
            self.match_token(&[TokenType::Multiply, TokenType::Divide, TokenType::Modulo])
        {
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// `unary → "-" unary | exponent`
    fn unary(&mut self) -> Result<Expr<'a>, ParseError> {
        if let Some(operator) = self.match_token(&[TokenType::Subtract]) {
            let right = self.unary()?;
            let expr = Expr::Unary {
                operator,
                right: Box::new(right),
            };
            Ok(expr)
        } else {
            self.exponent()
        }
    }

    /// `exponent → primary ( "^" unary )?`
    fn exponent(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr = self.primary()?;

        if let Some(operator) = self.match_token(&[TokenType::Exponent]) {
            // `unary` rather than `exponent`, so the right operand can be negative, e.g. `2 ^ -1`.
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// `primary → INTEGER | FLOAT | STRING | "true" | "false" | "null"
    /// | IDENTIFIER | "(" expression ")"`
    fn primary(&mut self) -> Result<Expr<'a>, ParseError> {
        // TODO: Parse IDENTIFIER.
        let token = self.peek().clone();

        let value = match token.token_type() {
            TokenType::Integer(n) => Some(LiteralValue::Int(*n)),
            TokenType::Float(n) => Some(LiteralValue::Float(*n)),
            TokenType::String(s) => Some(LiteralValue::Str(s.clone())),
            TokenType::True => Some(LiteralValue::Bool(true)),
            TokenType::False => Some(LiteralValue::Bool(false)),
            TokenType::Null => Some(LiteralValue::Null),
            _ => None,
        };

        if let Some(value) = value {
            self.advance();
            return Ok(Expr::Literal {
                value,
                line: token.line(),
            });
        }

        if self.check(&TokenType::LParen) {
            self.advance();
            let expr = self.expression()?;
            self.consume(&TokenType::RParen, ParseErrorKind::UnclosedParen)?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        Err(self.error(ParseErrorKind::UnexpectedToken))
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

    /// Returns `true` if the current token is of type `token_type`.
    ///
    /// Compares only the variant, so a payload like the `0` in `Integer(0)` is ignored.
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        discriminant(self.peek().token_type()) == discriminant(token_type)
    }

    /// Consumes and returns the current token if it's one of `token_types`.
    fn match_token(&mut self, token_types: &[TokenType]) -> Option<Token<'a>> {
        for token_type in token_types {
            if self.check(token_type) {
                return Some(self.advance().clone());
            }
        }
        None
    }

    /// Consumes and returns the current token if it's of type `token_type`,
    /// or returns a syntax error of the given kind.
    fn consume(
        &mut self,
        token_type: &TokenType,
        kind: ParseErrorKind,
    ) -> Result<Token<'a>, ParseError> {
        if self.check(token_type) {
            Ok(self.advance().clone())
        } else {
            Err(self.error(kind))
        }
    }

    /// Returns `true` once `current` has reached the EOF token.
    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type(), TokenType::Eof)
    }

    /// Builds a syntax error pointing at the current token.
    fn error(&self, kind: ParseErrorKind) -> ParseError {
        let token = self.peek();
        let found = match token.token_type() {
            TokenType::Eof => None,
            _ => Some(token.lexeme().to_string()),
        };

        ParseError {
            line: token.line(),
            kind,
            found,
        }
    }
}
