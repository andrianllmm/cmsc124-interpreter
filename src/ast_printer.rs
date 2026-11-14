//! Prints the syntax tree for `--parse`.

use crate::ast::{Expr, LiteralValue, Stmt};

/// Renders a statement. An expression statement prints as its expression.
pub fn print_stmt(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Expression(expr) => print(expr),
        Stmt::Block(statements) => {
            let mut parts = vec!["block".to_string()];
            parts.extend(statements.iter().map(print_stmt));
            format!("({})", parts.join(" "))
        }
    }
}

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
        Expr::Variable { name } => name.lexeme().to_string(),
        Expr::Call {
            callee, arguments, ..
        } => {
            let mut parts = vec![print(callee)];
            parts.extend(arguments.iter().map(print));
            format!("(call {})", parts.join(" "))
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
        Expr::Assign { name, value } => {
            format!("(= {} {})", name.lexeme(), print(value))
        }
    }
}
