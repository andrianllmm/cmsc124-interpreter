//! Runtime values produced by the evaluator.

use crate::ast::LiteralValue;
use crate::function::Function;
use crate::native::Native;
use crate::table::Table;
use std::fmt;
use std::rc::Rc;

/// A Grizzly value. Grizzly is dynamically typed, so the evaluator matches on
/// the variant to find out what it's holding.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
    /// A built-in function, e.g. `print`.
    Native(Native),
    /// A lambda, shared so copies of the value are the same function.
    Function(Rc<Function>),
    /// Lists are immutable, so copies share one `Vec` instead of cloning it.
    List(Rc<Vec<Value>>),
    /// Immutable like lists, so copies share one table.
    Table(Rc<Table>),
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
            Value::Native(native) => write!(f, "<native fn {}>", native.name()),
            Value::Function(_) => write!(f, "<fn>"),
            // Elements print by these same rules, so `["a", 1.0]` prints `[a, 1.0]`.
            Value::List(elements) => {
                write!(f, "[")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, "]")
            }
            Value::Table(table) => write!(f, "{}", table),
        }
    }
}
