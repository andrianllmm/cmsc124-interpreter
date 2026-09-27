//! Syntax tree built by the parser.

use crate::token::Token;

/// An expression node. Borrows from the source through its tokens.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'a> {
    /// A number, string, boolean, or `null`.
    Literal(Token<'a>),
    /// A binary operation: `left operator right`.
    Binary(Box<Expr<'a>>, Token<'a>, Box<Expr<'a>>),
    /// A unary operation: `operator right`.
    Unary(Token<'a>, Box<Expr<'a>>),
    /// A parenthesized expression.
    Grouping(Box<Expr<'a>>),
}
