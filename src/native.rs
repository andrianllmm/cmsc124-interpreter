//! Functions built into the interpreter rather than written in Grizzly.

use crate::callable::Callable;
use crate::evaluator::{Interpreter, RuntimeError};
use crate::value::Value;

/// A built-in function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Native {
    /// `print(value)`: writes the value and a newline to stdout.
    Print,
}

impl Native {
    /// Every built-in, so each can be bound in the global environment.
    pub const ALL: [Native; 1] = [Native::Print];

    pub fn name(&self) -> &'static str {
        match self {
            Native::Print => "print",
        }
    }
}

impl Callable for Native {
    fn arity(&self) -> usize {
        match self {
            Native::Print => 1,
        }
    }

    /// Runs host code. Natives don't touch the interpreter's state.
    fn call(
        &self,
        _interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        match self {
            Native::Print => {
                println!("{}", arguments[0]);
                Ok(Value::Null)
            }
        }
    }
}
