//! Recursive descent parser that turns tokens into a syntax tree.
//!
//! Each rule method follows its rule from the grammar in the README.

use crate::ast::{Expr, Lambda, LambdaBody, LiteralValue, Program, Stmt};
use crate::token::{Token, TokenType};
use std::fmt::{self, Display, Formatter};
use std::mem::discriminant;
use std::rc::Rc;

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
    /// A `[` has no matching `]`.
    UnclosedBracket,
    /// A `{` is missing where a block must start, e.g. after an `if` condition.
    MissingBrace,
    /// A `{` has no matching `}`.
    UnclosedBrace,
    /// A token that can't start an expression, e.g. `*` or `;`.
    UnexpectedToken,
    /// An invalid assignment operation
    InvalidAssignTarget,
    /// A lambda parameter that isn't an identifier, e.g. `(1) -> 1`.
    MissingParameter,
    /// A `for` without a loop variable, e.g. `for 1 in xs { }`.
    MissingLoopVariable,
    /// A `for` loop variable not followed by `in`.
    MissingIn,
    /// A return located outside of a lambda body
    InvalidReturnLoc,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            ParseErrorKind::MissingTerminator => "Missing ';'",
            ParseErrorKind::UnclosedParen => "Missing ')'",
            ParseErrorKind::UnclosedBracket => "Missing ']'",
            ParseErrorKind::MissingBrace => "Missing '{'",
            ParseErrorKind::UnclosedBrace => "Missing '}'",
            ParseErrorKind::UnexpectedToken => "Missing expression",
            ParseErrorKind::InvalidAssignTarget => "Invalid assignment target",
            ParseErrorKind::MissingParameter => "Missing parameter name",
            ParseErrorKind::MissingLoopVariable => "Missing loop variable",
            ParseErrorKind::MissingIn => "Missing 'in'",
            ParseErrorKind::InvalidReturnLoc => "Can't return from top level",
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
pub struct Parser {
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    current: usize,
    lambda_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            errors: Vec::new(),
            current: 0,
            lambda_depth: 0,
        }
    }

    /// `program → statement* EOF`
    ///
    /// Parses each statement into one `Program` root.
    ///
    /// On a syntax error, skips to the next statement boundary and keeps going,
    /// so every error in the source is reported at once.
    /// Consumes the parser, so the tree is handed over without a copy.
    pub fn parse(mut self) -> Result<Program, Vec<ParseError>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.statement() {
                Ok(stmt) => statements.push(stmt),
                Err(error) => {
                    self.errors.push(error);
                    self.synchronize();
                }
            }
        }

        if self.errors.is_empty() {
            Ok(Program { statements })
        } else {
            Err(self.errors)
        }
    }

    /// `statement → exprStmt | ifStmt | whileStmt | forStmt | returnStmt | block`
    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenType::If]).is_some() {
            return self.if_stmt();
        }
        if self.match_token(&[TokenType::LBrace]).is_some() {
            return Ok(Stmt::Block(self.block()?));
        }
        if self.match_token(&[TokenType::While]).is_some() {
            return self.while_stmt();
        }
        if self.match_token(&[TokenType::For]).is_some() {
            return self.for_stmt();
        }
        if let Some(token) = self.match_token(&[TokenType::Return]) {
            return self.return_stmt(&token);
        }
        self.expr_stmt()
    }

    /// `ifStmt → "if" expression block ( "else" ( ifStmt | block ) )?`,
    /// with the `if` already consumed.
    ///
    /// Branches must be blocks, so there is no dangling `else`.
    fn if_stmt(&mut self) -> Result<Stmt, ParseError> {
        let condition = self.expression()?;
        let then_branch = self.required_block()?;

        let else_branch = if self.match_token(&[TokenType::Else]).is_some() {
            if self.match_token(&[TokenType::If]).is_some() {
                Some(Box::new(self.if_stmt()?))
            } else {
                Some(Box::new(self.required_block()?))
            }
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    /// `whileStmt → "while" expression block`,
    /// with the while already consumed.
    fn while_stmt(&mut self) -> Result<Stmt, ParseError> {
        let condition = self.expression()?;
        let body = self.required_block()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    /// `forStmt → "for" IDENTIFIER "in" expression block`,
    /// with the `for` already consumed.
    fn for_stmt(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(&TokenType::Identifier, ParseErrorKind::MissingLoopVariable)?;
        self.consume(&TokenType::In, ParseErrorKind::MissingIn)?;
        let iterable = self.expression()?;
        self.consume(&TokenType::LBrace, ParseErrorKind::MissingBrace)?;
        let body = self.block()?;

        Ok(Stmt::For {
            name,
            iterable,
            body,
        })
    }

    /// `returnStmt → "return" expression? ";"`,
    /// with the return already consumed and the statement terminated
    fn return_stmt(&mut self, token: &Token) -> Result<Stmt, ParseError> {
        if self.lambda_depth == 0 {
            return Err(self.error(ParseErrorKind::InvalidReturnLoc));
        }

        if self.check(&TokenType::Terminator) {
            self.consume(&TokenType::Terminator, ParseErrorKind::MissingTerminator)?;
            Ok(Stmt::Return {
                keyword: token.clone(),
                value: None,
            })
        } else {
            let expr = self.expression()?;
            self.consume(&TokenType::Terminator, ParseErrorKind::MissingTerminator)?;
            Ok(Stmt::Return {
                keyword: token.clone(),
                value: Some(expr),
            })
        }
    }

    /// A block where the grammar requires one, so a missing `{` is an error.
    fn required_block(&mut self) -> Result<Stmt, ParseError> {
        self.consume(&TokenType::LBrace, ParseErrorKind::MissingBrace)?;
        Ok(Stmt::Block(self.block()?))
    }

    /// `block → "{" statement* "}"`, with the `{` already consumed.
    ///
    /// Returns the bare statements, so each caller wraps them in what it needs:
    /// a `Stmt::Block`, or a lambda's `LambdaBody::Block`.
    ///
    /// Recovers from errors inside the block, so its `}` still closes it
    /// instead of being reported as a stray token.
    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            match self.statement() {
                Ok(stmt) => statements.push(stmt),
                Err(error) => {
                    self.errors.push(error);
                    // Already at the closing `}`: let the loop end the block.
                    if !self.check(&TokenType::RBrace) {
                        self.synchronize();
                    }
                }
            }
        }
        self.consume(&TokenType::RBrace, ParseErrorKind::UnclosedBrace)?;
        Ok(statements)
    }

    /// `exprStmt → expression ";"`
    fn expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(&TokenType::Terminator, ParseErrorKind::MissingTerminator)?;
        Ok(Stmt::Expression(expr))
    }

    /// Discards tokens until a statement boundary: after a `;`,
    /// or before a `}` or a statement keyword.
    ///
    /// Always consumes at least one token, so recovery can't loop
    /// on the token that caused the error.
    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if matches!(self.advance().token_type(), TokenType::Terminator) {
                return;
            }
            if matches!(
                self.peek().token_type(),
                TokenType::RBrace
                    | TokenType::If
                    | TokenType::While
                    | TokenType::For
                    | TokenType::Match
                    | TokenType::Return
            ) {
                return;
            }
        }
    }

    /// `expression → assignment`
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    /// `assignment → IDENTIFIER ( "=" | "+=" | "-=" | "*=" | "/=" | "|=" ) assignment
    /// | lambda | pipe`
    fn assignment(&mut self) -> Result<Expr, ParseError> {
        // Checked before `pipe`, which would otherwise read a lambda's
        // parameters as a variable or grouping and then trip on the `->`.
        if self.is_lambda_start() {
            return self.lambda();
        }

        // parse the left side first
        let mut expr = self.pipe()?;

        while self.mul_check(&[
            TokenType::Assign,
            TokenType::AddAssign,
            TokenType::SubAssign,
            TokenType::MulAssign,
            TokenType::DivAssign,
            TokenType::PipeAssign,
        ]) {
            let op_token = self.peek().clone();
            // Moves the target out instead of copying it. `expr` is reassigned below.
            let Expr::Variable { name: name_token } = expr else {
                return Err(self.error(ParseErrorKind::InvalidAssignTarget));
            };
            self.advance();
            let right = self.assignment()?;
            expr = match op_token.token_type() {
                TokenType::Assign => Expr::Assign {
                    name: name_token,
                    value: Box::new(right),
                },
                // valid token types are filtered in mul_check. only compound assignments are present here
                _ => Expr::CompoundAssign {
                    name: name_token,
                    operator: op_token,
                    value: Box::new(right),
                },
            }
        }

        Ok(expr)
    }

    /// Whether the upcoming tokens start a lambda: `IDENTIFIER "->"`,
    /// or a parenthesized list whose matching `)` is followed by `->`.
    ///
    /// Looks past the whole `( ... )`, so a malformed parameter list like
    /// `(1) -> 1` is still parsed as a lambda and reported as one.
    fn is_lambda_start(&self) -> bool {
        match self.peek().token_type() {
            TokenType::Identifier => self.is_arrow_at(self.current + 1),
            TokenType::LParen => {
                // Track nesting so a `)` inside the list, e.g. in `(f(a)) -> 1`,
                // isn't mistaken for the one that closes it.
                let mut depth = 0;
                for (i, token) in self.tokens.iter().enumerate().skip(self.current) {
                    match token.token_type() {
                        TokenType::LParen => depth += 1,
                        TokenType::RParen => {
                            depth -= 1;
                            if depth == 0 {
                                return self.is_arrow_at(i + 1);
                            }
                        }
                        // Unclosed `(`: not a lambda. Let grouping report the missing `)`.
                        TokenType::Eof => return false,
                        _ => {}
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Whether the token at `index` is `->`.
    fn is_arrow_at(&self, index: usize) -> bool {
        self.tokens
            .get(index)
            .is_some_and(|t| matches!(t.token_type(), TokenType::LambdaArrow))
    }

    /// `lambda → ( IDENTIFIER | "(" parameters? ")" ) "->" ( expression | block )`
    ///
    /// Only called after `is_lambda_start`, so the parameters and `->` are known to be there
    /// unless the parameter list itself is malformed.
    fn lambda(&mut self) -> Result<Expr, ParseError> {
        // `(a, b) -> ...` or the paren-less single-parameter form `x -> ...`.
        let params = if self.match_token(&[TokenType::LParen]).is_some() {
            self.parameters()?
        } else {
            vec![self.consume(&TokenType::Identifier, ParseErrorKind::MissingParameter)?]
        };
        self.consume(&TokenType::LambdaArrow, ParseErrorKind::UnexpectedToken)?;

        // A `{` right after `->` always starts a block body. Otherwise the body is
        // a full expression, so it extends as far right as possible:
        // `x -> x + 1` is `x -> (x + 1)`, not `(x -> x) + 1`.
        self.lambda_depth += 1;
        let body = if self.match_token(&[TokenType::LBrace]).is_some() {
            LambdaBody::Block(self.block()?)
        } else {
            LambdaBody::Expression(Box::new(self.expression()?))
        };
        self.lambda_depth -= 1;
        Ok(Expr::Lambda(Rc::new(Lambda { params, body })))
    }

    /// `parameters → IDENTIFIER ( "," IDENTIFIER )*`, then the closing `)`,
    /// with the `(` already consumed.
    fn parameters(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut params = Vec::new();
        // `()` takes no parameters. Otherwise every `,` must be followed by a
        // name, so a trailing comma like `(a, )` is an error.
        if !self.check(&TokenType::RParen) {
            loop {
                params
                    .push(self.consume(&TokenType::Identifier, ParseErrorKind::MissingParameter)?);
                if self.match_token(&[TokenType::Comma]).is_none() {
                    break;
                }
            }
        }
        self.consume(&TokenType::RParen, ParseErrorKind::UnclosedParen)?;
        Ok(params)
    }

    /// `pipe → logicOr ( "|>" logicOr )*`
    fn pipe(&mut self) -> Result<Expr, ParseError> {
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
    fn logic_or(&mut self) -> Result<Expr, ParseError> {
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
    fn logic_and(&mut self) -> Result<Expr, ParseError> {
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
    fn logic_not(&mut self) -> Result<Expr, ParseError> {
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

    /// `comparison → term ( ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) term )?`
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        // `if`, not `while`: comparisons don't chain, so `1 < 2 < 3` is a syntax error.
        if let Some(operator) = self.match_token(&[
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
    fn term(&mut self) -> Result<Expr, ParseError> {
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
    fn factor(&mut self) -> Result<Expr, ParseError> {
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
    fn unary(&mut self) -> Result<Expr, ParseError> {
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

    /// `exponent → call ( "^" unary )?`
    fn exponent(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.call()?;

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

    /// `call → primary ( "(" exprList? ")" )*`
    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        while self.match_token(&[TokenType::LParen]).is_some() {
            expr = self.finish_call(expr)?;
        }

        Ok(expr)
    }

    /// The arguments and closing `)` of a call, with the `(` already consumed.
    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let (arguments, paren) =
            self.expr_list(TokenType::RParen, ParseErrorKind::UnclosedParen)?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    /// `exprList → expression ( "," expression )*`, then the `close` token,
    /// which is returned so errors can point at it.
    ///
    /// Shared by call arguments and list elements. `close` right away means an
    /// empty list. Otherwise every `,` must be followed by an expression, so a
    /// trailing comma like `[1, ]` is an error.
    fn expr_list(
        &mut self,
        close: TokenType,
        unclosed: ParseErrorKind,
    ) -> Result<(Vec<Expr>, Token), ParseError> {
        let mut exprs = Vec::new();
        if !self.check(&close) {
            loop {
                exprs.push(self.expression()?);
                if self.match_token(&[TokenType::Comma]).is_none() {
                    break;
                }
            }
        }

        let close = self.consume(&close, unclosed)?;
        Ok((exprs, close))
    }

    /// `primary → INTEGER | FLOAT | STRING | "true" | "false" | "null"
    /// | IDENTIFIER | "(" expression ")" | "[" exprList? "]"`
    fn primary(&mut self) -> Result<Expr, ParseError> {
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

        if self.check(&TokenType::Identifier) {
            self.advance();
            return Ok(Expr::Variable { name: token });
        }

        if self.check(&TokenType::LParen) {
            self.advance();
            let expr = self.expression()?;
            self.consume(&TokenType::RParen, ParseErrorKind::UnclosedParen)?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        if self.check(&TokenType::LBracket) {
            self.advance();
            let (elements, _) =
                self.expr_list(TokenType::RBracket, ParseErrorKind::UnclosedBracket)?;
            return Ok(Expr::List { elements });
        }

        Err(self.error(ParseErrorKind::UnexpectedToken))
    }

    /// Returns the current token without consuming it.
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Consumes and returns the current token.
    ///
    /// Stays on EOF so `peek` never goes out of bounds.
    fn advance(&mut self) -> &Token {
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

    /// Returns `true` if the current token is of one of `token_types`.
    ///
    /// Compares only the variant, so a payload like the `0` in `Integer(0)` is ignored.
    fn mul_check(&self, token_types: &[TokenType]) -> bool {
        let mut is_present = false;
        for token_type in token_types {
            if discriminant(self.peek().token_type()) == discriminant(token_type) {
                is_present = true;
                break;
            }
        }
        is_present
    }

    /// Consumes and returns the current token if it's one of `token_types`.
    fn match_token(&mut self, token_types: &[TokenType]) -> Option<Token> {
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
    ) -> Result<Token, ParseError> {
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
