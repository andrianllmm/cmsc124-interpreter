use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'a> {
    Literal(Token<'a>),
    Binary(Box<Expr<'a>>, Token<'a>, Box<Expr<'a>>),
    Unary(Token<'a>, Box<Expr<'a>>),
    Grouping(Box<Expr<'a>>),
}
