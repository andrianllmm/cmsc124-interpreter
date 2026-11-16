//! Functions written in Grizzly, as opposed to natives.

use crate::ast::{Lambda, LambdaBody};
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
    pub closure: Rc<RefCell<Environment>>,
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.lambda.params.len()
    }

    /// Runs the body in a fresh environment whose parent is the captured one,
    /// not the caller's, so names in the body mean what they meant where the
    /// lambda was written.
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        // Fresh per call, so locals never carry over between calls and
        // recursive calls each get their own parameters.
        let mut environment = Environment::with_enclosing(Rc::clone(&self.closure));
        for (param, argument) in self.lambda.params.iter().zip(arguments) {
            environment.define(param.lexeme(), argument);
        }

        match &self.lambda.body {
            LambdaBody::Expression(expr) => {
                interpreter.with_environment(environment, |interpreter| interpreter.evaluate(expr))
            }
            // Without `return`, a block body produces `null`.
            LambdaBody::Block(statements) => interpreter
                .execute_block(statements, environment)
                .map(|()| Value::Null),
        }
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
