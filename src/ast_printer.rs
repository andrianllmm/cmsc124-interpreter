//! Prints the syntax tree for `--parse`.

use crate::ast::{Expr, LiteralValue};

/// Renders the tree in parenthesized prefix form, e.g. `(+ 1 2)`.
pub fn print(expr: &Expr) -> String {
    match expr {
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            format!("({} {} {})", operator.lexeme(), print(left), print(right))
        }
        Expr::Unary { operator, right } => {
            format!("({} {})", operator.lexeme(), print(right))
        }
        Expr::Grouping { expression } => {
            format!("(group {})", print(expression))
        }
        Expr::Literal { value, .. } => match value {
            // Quoted so strings can't be mistaken for identifiers.
            LiteralValue::Str(s) => format!("\"{}\"", s.escape_debug()),
            // Debug keeps the `.0` on whole floats, so `1.0` doesn't print as `1`.
            LiteralValue::Float(n) => format!("{:?}", n),
            LiteralValue::Int(n) => n.to_string(),
            LiteralValue::Bool(b) => b.to_string(),
            LiteralValue::Null => "null".to_string(),
        },
    }
}
