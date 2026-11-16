//! The one interface every callable value implements, so a call
//! expression never needs to know which kind it's holding.

use crate::evaluator::{Interpreter, RuntimeError};
use crate::value::Value;

/// Something a call expression can invoke, e.g. a native like `print`.
pub trait Callable {
    /// How many arguments a call must pass.
    fn arity(&self) -> usize;

    /// Runs the callable. The caller has already checked `arity`.
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError>;
}
