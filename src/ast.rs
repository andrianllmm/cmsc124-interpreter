//! Syntax tree built by the parser.

use crate::token::Token;
use std::rc::Rc;

/// The root of the tree: every statement in source order.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

/// A statement node. Runs for its effect rather than its value.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// An expression followed by `;`.
    Expression(Expr),
    /// Statements between `{` and `}`.
    Block(Vec<Stmt>),
    /// `if condition { ... }`, with an optional `else` that is a block or another `if`.
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

/// An expression node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A number, string, boolean, or `null`.
    Literal { value: LiteralValue, line: u32 },
    /// A binary operation: `left operator right`.
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    /// A unary operation: `operator right`.
    Unary { operator: Token, right: Box<Expr> },
    /// A parenthesized expression.
    Grouping { expression: Box<Expr> },
    /// A name, read from the environment.
    Variable { name: Token },
    /// A call: `callee(arguments)`.
    Call {
        callee: Box<Expr>,
        /// The closing `)`, so errors can point at the call's line.
        paren: Token,
        arguments: Vec<Expr>,
    },
    /// A assignment to a variable
    Assign { name: Token, value: Box<Expr> },
    /// Compound assignments
    CompoundAssign {
        name: Token,
        operator: Token,
        value: Box<Expr>,
    },
    /// An anonymous function: `(params) -> body`.
    /// Named only by assigning it, e.g. `add = (a, b) -> a + b;`.
    ///
    /// Behind an `Rc` so a function value can share the node instead of copying it.
    Lambda(Rc<Lambda>),
}

/// A lambda's parameters and body.
#[derive(Debug, PartialEq)]
pub struct Lambda {
    /// Parameter names in order. Empty for `() -> ...`.
    pub params: Vec<Token>,
    pub body: LambdaBody,
}

/// What follows a lambda's `->`.
#[derive(Debug, PartialEq)]
pub enum LambdaBody {
    /// `x -> expr`, which produces `expr`.
    Expression(Box<Expr>),
    /// `x -> { ... }`, the statements of a block.
    Block(Vec<Stmt>),
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
