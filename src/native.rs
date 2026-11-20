//! Functions built into the interpreter rather than written in Grizzly.

use crate::callable::Callable;
use crate::evaluator::{Interpreter, RuntimeError, RuntimeErrorKind};
use crate::token::Token;
use crate::value::Value;
use std::rc::Rc;

/// A built-in function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Native {
    /// `print(value)`: writes the value and a newline to the interpreter's output.
    Print,
    /// `to_string(value)`: the value as text, as `print` would write it.
    ToString,
    /// `length(s)`: the number of characters in a string, or elements in a list.
    Length,
    /// `to_int(s)`: parses a string as an `Int`.
    ToInt,
    /// `to_float(s)`: parses a string as a `Float`.
    ToFloat,
    /// `append(list, value)`: a new list with `value` added at the end.
    Append,
}

impl Native {
    /// Every built-in, so each can be bound in the global environment.
    pub const ALL: [Native; 6] = [
        Native::Print,
        Native::ToString,
        Native::Length,
        Native::ToInt,
        Native::ToFloat,
        Native::Append,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Native::Print => "print",
            Native::ToString => "to_string",
            Native::Length => "length",
            Native::ToInt => "to_int",
            Native::ToFloat => "to_float",
            Native::Append => "append",
        }
    }
}

impl Callable for Native {
    fn arity(&self) -> usize {
        match self {
            Native::Append => 2,
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
            Native::Length => match argument {
                // Characters, not bytes, so `length("é")` is 1.
                Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
                Value::List(elements) => Ok(Value::Int(elements.len() as i64)),
                _ => Err(error(wrong_type("a string or list"))),
            },
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
            Native::Append => {
                let Value::List(elements) = argument else {
                    return Err(error(wrong_type("a list")));
                };
                // A copy, so the original list is left unchanged.
                let mut elements = elements.as_ref().clone();
                elements.push(arguments[1].clone());
                Ok(Value::List(Rc::new(elements)))
            }
        }
    }
}

/// The text of a `Str` argument, or an error for any other type.
fn string_argument(value: &Value) -> Result<&str, RuntimeErrorKind> {
    match value {
        Value::Str(s) => Ok(s),
        _ => Err(wrong_type("a string")),
    }
}

fn wrong_type(expected: &'static str) -> RuntimeErrorKind {
    RuntimeErrorKind::WrongArgumentType { expected }
}

fn conversion_failed(text: &str, target: &'static str) -> RuntimeErrorKind {
    RuntimeErrorKind::ConversionFailed {
        text: text.to_string(),
        target,
    }
}
