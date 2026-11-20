//! Functions built into the interpreter rather than written in Grizzly.

use crate::callable::Callable;
use crate::evaluator::{Interpreter, RuntimeError, RuntimeErrorKind};
use crate::token::Token;
use crate::value::Value;

/// A built-in function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Native {
    /// `print(value)`: writes the value and a newline to the interpreter's output.
    Print,
    /// `to_string(value)`: the value as text, as `print` would write it.
    ToString,
    /// `length(s)`: the number of characters in a string.
    Length,
    /// `to_int(s)`: parses a string as an `Int`.
    ToInt,
    /// `to_float(s)`: parses a string as a `Float`.
    ToFloat,
}

impl Native {
    /// Every built-in, so each can be bound in the global environment.
    pub const ALL: [Native; 5] = [
        Native::Print,
        Native::ToString,
        Native::Length,
        Native::ToInt,
        Native::ToFloat,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Native::Print => "print",
            Native::ToString => "to_string",
            Native::Length => "length",
            Native::ToInt => "to_int",
            Native::ToFloat => "to_float",
        }
    }
}

impl Callable for Native {
    fn arity(&self) -> usize {
        match self {
            Native::Print | Native::ToString | Native::Length | Native::ToInt | Native::ToFloat => {
                1
            }
        }
    }

    /// Runs host code. Errors point at `paren`, the call site.
    fn call(
        &self,
        interpreter: &mut Interpreter,
        paren: &Token,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let error = |kind| RuntimeError::new(paren, kind);
        let argument = &arguments[0];
        match self {
            Native::Print => {
                interpreter.print(argument);
                Ok(Value::Null)
            }
            Native::ToString => Ok(Value::Str(argument.to_string())),
            Native::Length => {
                let s = string_argument(argument).map_err(error)?;
                // Characters, not bytes, so `length("é")` is 1.
                Ok(Value::Int(s.chars().count() as i64))
            }
            Native::ToInt => {
                let s = string_argument(argument).map_err(error)?;
                s.parse()
                    .map(Value::Int)
                    .map_err(|_| error(conversion_failed(s, "Int")))
            }
            Native::ToFloat => {
                let s = string_argument(argument).map_err(error)?;
                s.parse()
                    .map(Value::Float)
                    .map_err(|_| error(conversion_failed(s, "Float")))
            }
        }
    }
}

/// The text of a `Str` argument, or an error for any other type.
fn string_argument(value: &Value) -> Result<&str, RuntimeErrorKind> {
    match value {
        Value::Str(s) => Ok(s),
        _ => Err(RuntimeErrorKind::ArgumentNotString),
    }
}

fn conversion_failed(text: &str, target: &'static str) -> RuntimeErrorKind {
    RuntimeErrorKind::ConversionFailed {
        text: text.to_string(),
        target,
    }
}
