use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Token),
    Binary(Box<Expr>, Token, Box<Expr>),
}
