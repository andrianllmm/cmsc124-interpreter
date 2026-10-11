//! Runtime values produced by the evaluator.

use crate::ast::LiteralValue;
use std::fmt;

/// A Grizzly value. Grizzly is dynamically typed, so the evaluator matches on
/// the variant to find out what it's holding.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
}

impl From<LiteralValue> for Value {
    fn from(literal: LiteralValue) -> Self {
        match literal {
            LiteralValue::Int(n) => Value::Int(n),
            LiteralValue::Float(n) => Value::Float(n),
            LiteralValue::Str(s) => Value::Str(s),
            LiteralValue::Bool(b) => Value::Bool(b),
            LiteralValue::Null => Value::Null,
        }
    }
}

/// How `--eval` and the REPL print a value.
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            // Debug keeps the `.0` on whole floats, so `Float` never prints like an `Int`.
            Value::Float(n) => write!(f, "{:?}", n),
            // Unquoted, so `"hi" ++ " there"` prints `hi there`.
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}
