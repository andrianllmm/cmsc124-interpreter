//! Prints the syntax tree for `--parse`.

use crate::ast::Expr;
use crate::token::TokenType;

/// Renders the tree in parenthesized prefix form, e.g. `(+ 1 2)`.
pub fn print(expr: &Expr) -> String {
    match expr {
        Expr::Binary(left, operator, right) => {
            format!("({} {} {})", operator.lexeme(), print(left), print(right))
        }
        Expr::Unary(operator, right) => {
            format!("({} {})", operator.lexeme(), print(right))
        }
        Expr::Grouping(inner) => {
            format!("(group {})", print(inner))
        }
        Expr::Literal(token) => match token.token_type() {
            // These carry no literal value, so their lexeme is the value.
            TokenType::True | TokenType::False | TokenType::Null => token.lexeme().to_string(),

            // Quoted so strings can't be mistaken for identifiers.
            TokenType::String(s) => format!("\"{}\"", s.escape_debug()),

            TokenType::Integer(_) | TokenType::Float(_) => token.token_type().literal_string(),

            // The parser only builds literals from the token types above.
            _ => unreachable!("non-literal token in Literal: {}", token.lexeme()),
        },
    }
}
