//! Variable bindings, chained so inner scopes can see outer ones.

use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A scope's bindings plus a link to the scope it sits in.
#[derive(Debug, Default)]
pub struct Environment {
    values: HashMap<String, Value>,
    /// Shared, since blocks and closures can outlive the code that made them.
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    /// A global environment, with nothing enclosing it.
    pub fn new() -> Self {
        Self::default()
    }

    /// A scope nested inside `enclosing`.
    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    /// Looks `name` up here, then outward. `None` if no scope has it.
    pub fn get(&self, name: &str) -> Option<Value> {
        match self.values.get(name) {
            Some(value) => Some(value.clone()),
            None => self.enclosing.as_ref()?.borrow().get(name),
        }
    }

    /// Binds `name` in this scope, shadowing any outer binding.
    /// Used for parameters, which are always local to their call.
    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    /// Updates the nearest binding of `name`, or creates it here if none exists.
    /// Never creates it in an outer scope.
    pub fn set(&mut self, name: &str, value: Value) {
        if let Err(value) = self.update(name, value) {
            self.values.insert(name.to_string(), value);
        }
    }

    /// Updates the nearest existing binding, handing `value` back if there is none.
    fn update(&mut self, name: &str, value: Value) -> Result<(), Value> {
        if let Some(slot) = self.values.get_mut(name) {
            *slot = value;
            return Ok(());
        }
        match &self.enclosing {
            Some(enclosing) => enclosing.borrow_mut().update(name, value),
            None => Err(value),
        }
    }
}
