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

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    expressions: Vec<Expr<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Parser<'a> {
        Parser { tokens, expressions: Vec::new(), current: 0 }
    }

    // Parses a single expression.
    pub fn parse(&mut self) -> Result<&Vec<Expr<'a>>, ParseError> {
        while !self.is_at_end() {
            let expr: Expr<'_> = self.expression()?;
            let is_terminator: bool = matches!(
                self.peek().token_type(), TokenType::Terminator
            );

            if is_terminator {
                self.expressions.push(expr);
                self.advance();
            } else {
                return Err(self.error(self.peek().clone(), "Missing terminator ';'"));
            }
        }

        return Ok(&self.expressions);

    }

    // expression -> term
    fn expression(&mut self) -> Result<Expr<'a>, ParseError> {
        self.logic_or()
    }

    // logicOr -> logAnd ( "or" logicAnd)
    fn logic_or(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr: Expr<'_> = self.logic_and()?;

        loop {
            let is_or: bool = matches!(
                self.peek().token_type(),
                TokenType::Or
            );

            if !is_or {
                break;
            }

            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.logic_and()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        return Ok(expr);
    }

    // logicAnd -> logicNot ("and" logicNot)
    fn logic_and(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr: Expr<'_> = self.logic_not()?;

        loop {
            let is_and: bool = matches!(
                self.peek().token_type(),
                TokenType::And
            );

            if !is_and {
                break;
            }

            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.logic_not()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        return Ok(expr);
    }

    // logicNot -> "not" logicNot | comparison
    fn logic_not(&mut self) -> Result<Expr<'a>, ParseError> {
        let is_logic_not: bool = matches!(
            self.peek().token_type(),
            TokenType::Not
        );

        if is_logic_not {
            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.logic_not()?;
            let expr: Expr<'_> = Expr::Unary(operator, Box::new(right));
            return Ok(expr);
        } else {
            return Ok(self.comparison()?);
        }
    }

    // comparison -> term ( ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) term )
    fn comparison(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr: Expr<'_> = self.term()?;

        loop {
            let is_operator: bool = matches!(
                self.peek().token_type(),
                TokenType::Equals |
                TokenType::NotEquals |
                TokenType::Less |
                TokenType::LessEquals |
                TokenType::More |
                TokenType::MoreEquals
            );

            if !is_operator {
                break;
            }

            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        return Ok(expr);
    }

    // term -> primary ( ( "+" | "-" | "++" ) primary )*
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

    // factor -> unary (( "*" | "/" | "%" ) unary )
    fn factor(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr: Expr<'_> = self.unary()?;

        loop {
            let is_operator: bool = matches!(
                self.peek().token_type(),
                TokenType::Multiply |
                TokenType::Divide |
                TokenType::Modulo
            );

            if !is_operator {
                break;
            }

            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        return Ok(expr);
    }

    // unary -> "-" unary | exponent
    fn unary(&mut self) -> Result<Expr<'a>, ParseError> {
        let is_unary_operator: bool = matches!(
            self.peek().token_type(),
            TokenType::Subtract
        );

        if is_unary_operator {
            let operator: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.unary()?;
            let expr: Expr<'_> = Expr::Unary(operator, Box::new(right));
            return Ok(expr);
        } else {
            return Ok(self.exponent()?);
        }
    }

    // exponent -> primary ( "^" exponent )?
    fn exponent(&mut self) -> Result<Expr<'a>, ParseError> {
        let mut expr: Expr<'_> = self.primary()?;

        let is_exponent: bool = matches!(
            self.peek().token_type(), TokenType::Exponent
        );

        if is_exponent {
            let exponent: Token<'_> = self.advance().clone();
            let right: Expr<'_> = self.exponent()?;
            expr = Expr::Binary(Box::new(expr), exponent, Box::new(right));
        }

        return Ok(expr);
    }

    // primary -> INTEGER | FLOAT | STRING | "true" | "false" | "null"
    fn primary(&mut self) -> Result<Expr<'a>, ParseError> {
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
    fn peek(&self) -> &Token<'a> {
        &self.tokens[self.current]
    }

    // consumes and returns the current token
    fn advance(&mut self) -> &Token<'a> {
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
    fn error(&self, token: Token<'a>, message: &str) -> ParseError {
        ParseError {
            line: token.line(),
            message: message.to_string(),
        }
    }
}
