//! The one interface every callable value implements, so a call
//! expression never needs to know which kind it's holding.

use crate::evaluator::{Interpreter, RuntimeError};
use crate::token::Token;
use crate::value::Value;

/// Something a call expression can invoke: a native like `print`, or a `Function`.
pub trait Callable {
    /// How many arguments a call must pass.
    fn arity(&self) -> usize;

    /// Runs the callable. The caller has already checked `arity`.
    /// `paren` is the call site, for errors raised inside the call.
    fn call(
        &self,
        interpreter: &mut Interpreter,
        paren: &Token,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError>;
}
