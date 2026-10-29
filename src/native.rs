//! Functions built into the interpreter rather than written in Grizzly.

use crate::value::Value;

/// A built-in function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Native {
    /// `print(value)`: writes the value and a newline to stdout.
    Print,
}

impl Native {
    /// Finds the built-in named `name`, if there is one.
    pub fn lookup(name: &str) -> Option<Native> {
        match name {
            "print" => Some(Native::Print),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Native::Print => "print",
        }
    }

    /// How many arguments a call must pass.
    pub fn arity(&self) -> usize {
        match self {
            Native::Print => 1,
        }
    }

    /// Runs the function. The caller has already checked `arity`.
    pub fn call(&self, arguments: Vec<Value>) -> Value {
        match self {
            Native::Print => {
                println!("{}", arguments[0]);
                Value::Null
            }
        }
    }
}
