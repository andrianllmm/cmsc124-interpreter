//! Syntax tree built by the parser.

use crate::token::Token;

/// The root of the tree: every statement in source order.
#[derive(Debug, Clone, PartialEq)]
pub struct Program<'a> {
    pub statements: Vec<Expr<'a>>,
}

/// An expression node. Borrows from the source through its tokens.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'a> {
    /// A number, string, boolean, or `null`.
    Literal { value: LiteralValue, line: u32 },
    /// A binary operation: `left operator right`.
    Binary {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    /// A unary operation: `operator right`.
    Unary {
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    /// A parenthesized expression.
    Grouping { expression: Box<Expr<'a>> },
    /// A name, read from the environment.
    Variable { name: Token<'a> },
    /// A call: `callee(arguments)`.
    Call {
        callee: Box<Expr<'a>>,
        /// The closing `)`, so errors can point at the call's line.
        paren: Token<'a>,
        arguments: Vec<Expr<'a>>,
    },
}

/// The value of a literal expression.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
}
