//! Syntax tree built by the parser.

use crate::token::Token;

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
