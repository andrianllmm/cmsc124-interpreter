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

#[allow(dead_code)] // Used once the interpreter owns an environment.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn nested() -> (Rc<RefCell<Environment>>, Environment) {
        let outer = Rc::new(RefCell::new(Environment::new()));
        let inner = Environment::with_enclosing(Rc::clone(&outer));
        (outer, inner)
    }

    #[test]
    fn missing_name_is_none() {
        assert_eq!(Environment::new().get("x"), None);
    }

    #[test]
    fn set_then_get() {
        let mut env = Environment::new();
        env.set("x", Value::Int(1));
        assert_eq!(env.get("x"), Some(Value::Int(1)));
    }

    #[test]
    fn set_overwrites_in_same_scope() {
        let mut env = Environment::new();
        env.set("x", Value::Int(1));
        env.set("x", Value::Int(2));
        assert_eq!(env.get("x"), Some(Value::Int(2)));
    }

    #[test]
    fn get_walks_enclosing() {
        let (outer, inner) = nested();
        outer.borrow_mut().set("x", Value::Int(1));
        assert_eq!(inner.get("x"), Some(Value::Int(1)));
    }

    #[test]
    fn set_updates_enclosing_binding() {
        let (outer, mut inner) = nested();
        outer.borrow_mut().set("x", Value::Int(1));
        inner.set("x", Value::Int(2));
        assert_eq!(outer.borrow().get("x"), Some(Value::Int(2)));
        assert!(!inner.values.contains_key("x"));
    }

    #[test]
    fn set_new_name_stays_in_current_scope() {
        let (outer, mut inner) = nested();
        inner.set("x", Value::Int(1));
        assert_eq!(inner.get("x"), Some(Value::Int(1)));
        assert_eq!(outer.borrow().get("x"), None);
    }
}
