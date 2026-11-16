//! Functions written in Grizzly, as opposed to natives.

use crate::ast::Lambda;
use crate::callable::Callable;
use crate::environment::Environment;
use crate::evaluator::{Interpreter, RuntimeError};
use crate::value::Value;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

/// A lambda turned into a value: its syntax, plus the environment it was
/// created in, which its calls will see.
pub struct Function {
    /// Shared with the syntax tree, so creating a function copies nothing.
    pub lambda: Rc<Lambda>,
    /// Kept alive by the function, so it can outlive the call that made it.
    #[expect(dead_code, reason = "read once calls run the body (#251)")]
    pub closure: Rc<RefCell<Environment>>,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.lambda.params.len()
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        todo!("call lambdas")
    }
}

/// Functions are equal only to themselves, like references in most languages.
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

/// Written by hand: a derived `Debug` would print the closure, which can
/// contain the function itself and recurse forever.
impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn>")
    }
}
