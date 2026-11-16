//! Syntax tree built by the parser.

use crate::token::Token;

/// The root of the tree: every statement in source order.
#[derive(Debug, Clone, PartialEq)]
pub struct Program<'a> {
    pub statements: Vec<Stmt<'a>>,
}

/// A statement node. Runs for its effect rather than its value.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'a> {
    /// An expression followed by `;`.
    Expression(Expr<'a>),
    /// Statements between `{` and `}`.
    Block(Vec<Stmt<'a>>),
    /// `if condition { ... }`, with an optional `else` that is a block or another `if`.
    If {
        condition: Expr<'a>,
        then_branch: Box<Stmt<'a>>,
        else_branch: Option<Box<Stmt<'a>>>,
    },
    While {
        condition: Expr<'a>,
        body: Box<Stmt<'a>>,
    },
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
    /// A assignment to a variable
    Assign {
        name: Token<'a>,
        value: Box<Expr<'a>>,
    },
    /// Compound assignments
    CompoundAssign {
        name: Token<'a>,
        operator: Token<'a>,
        value: Box<Expr<'a>>,
    },
    /// An anonymous function: `(params) -> body`.
    /// Named only by assigning it, e.g. `add = (a, b) -> a + b;`.
    Lambda {
        /// Parameter names in order. Empty for `() -> ...`.
        params: Vec<Token<'a>>,
        body: LambdaBody<'a>,
    },
}

/// What follows a lambda's `->`.
#[derive(Debug, Clone, PartialEq)]
pub enum LambdaBody<'a> {
    /// `x -> expr`, which produces `expr`.
    Expression(Box<Expr<'a>>),
    /// `x -> { ... }`, the statements of a block.
    Block(Vec<Stmt<'a>>),
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
