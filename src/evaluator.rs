//! Tree-walking evaluator that turns expressions into values.

use crate::ast::{Expr, Stmt};
use crate::callable::Callable;
use crate::environment::Environment;
use crate::function::Function;
use crate::native::Native;
use crate::table::{Table, TableError};
use crate::token::{Token, TokenType};
use crate::value::Value;
use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Write};
use std::rc::Rc;

/// An error raised while evaluating.
pub struct RuntimeError {
    /// Token where evaluation failed, e.g. an operator or a name,
    /// so the message can point at its line.
    pub token: Token,
    pub kind: RuntimeErrorKind,
}

impl RuntimeError {
    /// Builds an error pointing at `token`.
    pub fn new(token: &Token, kind: RuntimeErrorKind) -> Self {
        RuntimeError {
            token: token.clone(),
            kind,
        }
    }
}

/// What went wrong while evaluating.
pub enum RuntimeErrorKind {
    /// A numeric operator got a non-number, e.g. `-"a"`.
    OperandNotNumber,
    /// `++` got a non-string, e.g. `"a" ++ 1`.
    OperandNotString,
    /// The result doesn't fit in an `Int`, e.g. negating its minimum.
    IntegerOverflow,
    /// An `Int` divided by zero, with `/` or `%`.
    DivisionByZero,
    /// An `Int` raised to a negative `Int`, e.g. `2 ^ -1`.
    NegativeExponent,
    /// A name with no binding, e.g. `x` before `x = 1`.
    UndefinedVariable,
    /// A call on a value that isn't a function, e.g. `1()`.
    NotCallable,
    /// A `for` over a value that isn't a list, e.g. `for x in 1 { }`.
    NotIterable,
    /// Calls nested deeper than `MAX_CALL_DEPTH`, e.g. unbounded recursion.
    StackOverflow,
    /// A call passed the wrong number of arguments, e.g. `print()`.
    ArityMismatch { expected: usize, found: usize },
    /// A native got an argument of the wrong type, e.g. `length(1)`.
    /// `expected` names what it takes, e.g. `"a string"`.
    WrongArgumentType { expected: &'static str },
    /// A string that doesn't parse as `target`, e.g. `to_int("abc")`.
    ConversionFailed { text: String, target: &'static str },
    /// A table literal that couldn't be built, e.g. a list in a cell.
    InvalidTable(TableError),
    /// A signal to propagate the return value to the latest caller
    ReturnSignal { value: Value },
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match &self.kind {
            RuntimeErrorKind::UndefinedVariable => {
                format!("Undefined variable '{}'", self.token.lexeme())
            }
            RuntimeErrorKind::NotCallable => "Can only call functions".to_string(),
            RuntimeErrorKind::NotIterable => "Can only loop over a list".to_string(),
            RuntimeErrorKind::ArityMismatch { expected, found } => {
                format!("Expected {} arguments but got {}.", expected, found)
            }
            RuntimeErrorKind::StackOverflow => "Stack overflow".to_string(),
            RuntimeErrorKind::OperandNotNumber => "Operand must be a number".to_string(),
            RuntimeErrorKind::OperandNotString => "Operand must be a string".to_string(),
            RuntimeErrorKind::IntegerOverflow => "Integer overflow".to_string(),
            RuntimeErrorKind::DivisionByZero => "Division by zero".to_string(),
            RuntimeErrorKind::NegativeExponent => "Int exponent must not be negative".to_string(),
            RuntimeErrorKind::WrongArgumentType { expected } => {
                format!("Argument must be {}", expected)
            }
            RuntimeErrorKind::ConversionFailed { text, target } => {
                format!("Can't convert '{}' to {}", text, target)
            }
            RuntimeErrorKind::InvalidTable(error) => error.to_string(),
            RuntimeErrorKind::ReturnSignal { value: _ } => {
                unreachable!("return signal is caught by the call")
            }
        };
        write!(f, "Error at line {}: {}", self.token.line(), message)
    }
}

/// How many calls may be active at once before a call is a `Stack overflow`.
pub const MAX_CALL_DEPTH: usize = 5000;

/// Evaluates expressions against a global environment that outlives each call,
/// so state persists across a run's statements and across REPL lines.
pub struct Interpreter {
    /// Global bindings; a runtime error never resets it.
    environment: Rc<RefCell<Environment>>,
    /// Where `print`, echoed values, and stage output go.
    out: Box<dyn Write>,
    /// Where errors go.
    err: Box<dyn Write>,
    /// Calls currently active.
    call_depth: usize,
}

impl Interpreter {
    /// Starts with every native bound in the global environment,
    /// writing to stdout and stderr.
    pub fn new() -> Self {
        Self::with_output(Box::new(io::stdout()), Box::new(io::stderr()))
    }

    /// Like `new`, but writing to `out` and `err`, so a caller can capture output.
    pub fn with_output(out: Box<dyn Write>, err: Box<dyn Write>) -> Self {
        let mut globals = Environment::new();
        for native in Native::ALL {
            globals.set(native.name(), Value::Native(native));
        }
        Interpreter {
            environment: Rc::new(RefCell::new(globals)),
            out,
            err,
            call_depth: 0,
        }
    }

    /// Writes a line to the output.
    /// A failed write (e.g. a closed pipe) isn't the program's fault, so it's ignored.
    pub fn print(&mut self, line: impl Display) {
        let _ = writeln!(self.out, "{}", line);
    }

    /// Writes a line to the error output. Failed writes are ignored, like `print`.
    pub fn eprint(&mut self, line: impl Display) {
        let _ = writeln!(self.err, "{}", line);
    }

    /// Executes a statement, returning an expression statement's value.
    pub fn execute(&mut self, stmt: &Stmt) -> Result<Option<Value>, RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => self.evaluate(expr).map(Some),
            Stmt::Block(statements) => {
                let block = Environment::with_enclosing(Rc::clone(&self.environment));
                self.execute_block(statements, block).map(|_| None)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // Only the chosen branch runs. Each branch is a block, so it
                // gets its own child environment.
                if is_truthy(&self.evaluate(condition)?) {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
                Ok(None)
            }
            Stmt::While { condition, body } => {
                // The condition is checked for every iteration. If the condition is
                // true it would execute. Otherwise, skip the contents of the block
                // and terminate the loop.
                while is_truthy(&self.evaluate(condition)?) {
                    self.execute(body)?;
                }
                Ok(None)
            }
            Stmt::For {
                name,
                iterable,
                body,
            } => {
                let Value::List(elements) = self.evaluate(iterable)? else {
                    return Err(RuntimeError::new(name, RuntimeErrorKind::NotIterable));
                };
                // A fresh environment per iteration holds `name`, so it never
                // updates an outer binding, is gone after the loop, and each
                // iteration's closures capture their own element.
                for element in elements.iter() {
                    let mut environment = Environment::with_enclosing(Rc::clone(&self.environment));
                    environment.define(name.lexeme(), element.clone());
                    self.execute_block(body, environment)?;
                }
                Ok(None)
            }
            Stmt::Return { keyword, value } => {
                let value = value
                    .as_ref()
                    .map(|e| self.evaluate(e))
                    .transpose()?
                    .unwrap_or(Value::Null);
                Err(RuntimeError::new(
                    keyword,
                    RuntimeErrorKind::ReturnSignal { value },
                ))
            }
        }
    }

    /// Runs `statements` in `environment`.
    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Environment,
    ) -> Result<(), RuntimeError> {
        self.with_environment(environment, |interpreter| {
            statements
                .iter()
                .try_for_each(|stmt| interpreter.execute(stmt).map(|_| ()))
        })
    }

    /// Runs `body` with `environment` as the current scope, then restores the
    /// previous one, even on a runtime error, so the REPL never stays in a
    /// finished block's or call's scope.
    pub fn with_environment<T>(
        &mut self,
        environment: Environment,
        body: impl FnOnce(&mut Self) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        let previous = std::mem::replace(&mut self.environment, Rc::new(RefCell::new(environment)));
        let result = body(self);
        self.environment = previous;
        result
    }

    /// Calls `callee` with already-evaluated `arguments`.
    ///
    /// The only place that checks callability and arity, so every kind of
    /// callable gets the same checks and the same errors.
    fn call(
        &mut self,
        callee: Value,
        paren: &Token,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let callable: &dyn Callable = match &callee {
            Value::Native(native) => native,
            Value::Function(function) => function.as_ref(),
            _ => return Err(RuntimeError::new(paren, RuntimeErrorKind::NotCallable)),
        };
        if arguments.len() != callable.arity() {
            return Err(RuntimeError::new(
                paren,
                RuntimeErrorKind::ArityMismatch {
                    expected: callable.arity(),
                    found: arguments.len(),
                },
            ));
        }
        if self.call_depth >= MAX_CALL_DEPTH {
            return Err(RuntimeError::new(paren, RuntimeErrorKind::StackOverflow));
        }
        // Restored on success and error alike, so the REPL keeps working.
        self.call_depth += 1;
        let result = callable.call(self, paren, arguments);
        self.call_depth -= 1;

        match result {
            Err(RuntimeError {
                kind: RuntimeErrorKind::ReturnSignal { value },
                ..
            }) => Ok(value),
            other => other,
        }
    }

    /// Pipes `value` into `target`: `x |> f` calls `f(x)`, and `x |> f(a)`
    /// prepends `x` to the call's arguments, calling `f(x, a)`.
    ///
    /// A pipe chain as `target`, as in `x |= f |> g`, threads `value` through
    /// each step in order, so it means `x |> f |> g`.
    ///
    /// `target` is unevaluated, since `f(a)` after `|>` only names the function
    /// and its extra arguments.
    fn pipe(
        &mut self,
        value: Value,
        operator: &Token,
        target: &Expr,
    ) -> Result<Value, RuntimeError> {
        match target {
            // A chain like `f |> g` in `x |= f |> g`.
            // Only `|=` builds this, since `|>` is left-associative.
            Expr::Binary {
                left,
                operator: inner,
                right,
            } if *inner.token_type() == TokenType::Pipe => {
                // Run steps left to right: `g(f(x))`.
                let value = self.pipe(value, operator, left)?;
                self.pipe(value, inner, right)
            }
            // A call on the right, e.g. `x |> f(a)`, which calls `f(x, a)`.
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                // The piped value goes before the written arguments.
                let callee = self.evaluate(callee)?;
                let arguments = std::iter::once(Ok(value))
                    .chain(arguments.iter().map(|argument| self.evaluate(argument)))
                    .collect::<Result<Vec<_>, _>>()?;
                // Errors point at the `)`, like a direct call.
                self.call(callee, paren, arguments)
            }
            // Anything else, e.g. `x |> f`, is called with the piped value alone.
            _ => {
                let callee = self.evaluate(target)?;
                // No `)`, so errors point at the `|>`.
                self.call(callee, operator, vec![value])
            }
        }
    }

    /// Evaluates an expression. Operands are evaluated before their operator
    /// (post-order), so each node works with values, never with subtrees.
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal { value, .. } => Ok(value.clone().into()),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Variable { name } => self
                .environment
                .borrow()
                .get(name.lexeme())
                .ok_or_else(|| RuntimeError::new(name, RuntimeErrorKind::UndefinedVariable)),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(callee)?;
                let arguments = arguments
                    .iter()
                    .map(|argument| self.evaluate(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.call(callee, paren, arguments)
            }
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;
                unary(operator, right)
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                match (operator.token_type(), is_truthy(&left)) {
                    // The right side is a call target, not an operand.
                    (TokenType::Pipe, _) => return self.pipe(left, operator, right),
                    // `and` and `or` skip the right side once the left decides.
                    (TokenType::And, false) => return Ok(Value::Bool(false)),
                    (TokenType::Or, true) => return Ok(Value::Bool(true)),
                    _ => {}
                }
                let right = self.evaluate(right)?;
                binary(left, operator, right)
            }
            Expr::Assign { name, value } => {
                let right = self.evaluate(value)?;
                let mut env = self.environment.borrow_mut();

                env.set(name.lexeme(), right.clone());

                Ok(right)
            }
            Expr::List { elements } => {
                let values = elements
                    .iter()
                    .map(|element| self.evaluate(element))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Value::List(Rc::new(values)))
            }
            Expr::Table {
                keyword,
                columns,
                rows,
            } => {
                // Rows run top to bottom, left to right, then flip into columns.
                let mut cells: Vec<Vec<Value>> = vec![Vec::new(); columns.len()];
                for row in rows {
                    for (column, cell) in cells.iter_mut().zip(row) {
                        column.push(self.evaluate(cell)?);
                    }
                }
                let names = columns.iter().map(|c| c.lexeme().to_string()).collect();
                let cells = cells.into_iter().map(Rc::new).collect();
                Table::new(names, cells)
                    .map(|table| Value::Table(Rc::new(table)))
                    .map_err(|e| RuntimeError::new(keyword, RuntimeErrorKind::InvalidTable(e)))
            }
            // Captures the environment the lambda is created in, not the one
            // it's later called from, which makes scope lexical.
            Expr::Lambda(lambda) => Ok(Value::Function(Rc::new(Function {
                lambda: Rc::clone(lambda),
                closure: Rc::clone(&self.environment),
            }))),
            Expr::CompoundAssign {
                name,
                operator,
                value,
            } => {
                let env = self.environment.clone();
                let var = env.borrow().get(name.lexeme());
                let Some(val) = var else {
                    return Err(RuntimeError::new(name, RuntimeErrorKind::UndefinedVariable));
                };

                let new_val = if *operator.token_type() == TokenType::PipeAssign {
                    self.pipe(val, operator, value)?
                } else {
                    let right = self.evaluate(value)?;
                    binary(val, operator, right)?
                };

                env.borrow_mut().set(name.lexeme(), new_val.clone());

                Ok(new_val)
            }
        }
    }
}

/// Applies a unary operator to an evaluated operand.
fn unary(operator: &Token, right: Value) -> Result<Value, RuntimeError> {
    match operator.token_type() {
        TokenType::Subtract => match right {
            // `i64::MIN` has no positive counterpart, so negating it overflows.
            Value::Int(n) => n
                .checked_neg()
                .map(Value::Int)
                .ok_or(RuntimeErrorKind::IntegerOverflow),
            // Negating a float only flips its sign bit, so it can't overflow.
            Value::Float(n) => Ok(Value::Float(-n)),
            _ => Err(RuntimeErrorKind::OperandNotNumber),
        }
        .map_err(|kind| RuntimeError::new(operator, kind)),
        TokenType::Not => Ok(Value::Bool(!is_truthy(&right))),
        _ => unreachable!("parser only builds unary `-` and `not`"),
    }
}

/// Applies a binary operator to two evaluated operands.
fn binary(left: Value, operator: &Token, right: Value) -> Result<Value, RuntimeError> {
    let error = |kind| RuntimeError::new(operator, kind);

    match operator.token_type() {
        // `++` only joins strings; anything else must be converted first.
        TokenType::StringConcat => match (left, right) {
            (Value::Str(l), Value::Str(r)) => Ok(Value::Str(l + &r)),
            _ => Err(error(RuntimeErrorKind::OperandNotString)),
        },
        // Reached only when the left side didn't decide, so the right one does.
        TokenType::And | TokenType::Or => Ok(Value::Bool(is_truthy(&right))),
        TokenType::Equals => Ok(Value::Bool(is_equal(&left, &right))),
        TokenType::NotEquals => Ok(Value::Bool(!is_equal(&left, &right))),
        _ => {
            let numbers =
                numbers(&left, &right).ok_or_else(|| error(RuntimeErrorKind::OperandNotNumber))?;
            match operator.token_type() {
                TokenType::Less
                | TokenType::LessEquals
                | TokenType::More
                | TokenType::MoreEquals => Ok(Value::Bool(compare(operator, numbers))),
                _ => arithmetic(operator, numbers).map_err(error),
            }
        }
    }
}

/// Applies an arithmetic operator, keeping `Int`s exact and letting `Float`s follow IEEE 754.
fn arithmetic(operator: &Token, numbers: Numbers) -> Result<Value, RuntimeErrorKind> {
    match numbers {
        Numbers::Ints(n, m) => int_arithmetic(operator, n, m).map(Value::Int),
        Numbers::Floats(n, m) => Ok(Value::Float(float_arithmetic(operator, n, m))),
    }
}

/// Applies an arithmetic operator to `Int`s, erroring instead of overflowing.
fn int_arithmetic(operator: &Token, n: i64, m: i64) -> Result<i64, RuntimeErrorKind> {
    let result = match operator.token_type() {
        TokenType::Add | TokenType::AddAssign => n.checked_add(m),
        TokenType::Subtract | TokenType::SubAssign => n.checked_sub(m),
        TokenType::Multiply | TokenType::MulAssign => n.checked_mul(m),
        // `Int` has no infinity, so dividing by zero is an error.
        TokenType::Divide | TokenType::Modulo | TokenType::DivAssign if m == 0 => {
            return Err(RuntimeErrorKind::DivisionByZero);
        }
        // Only `Int::MIN / -1` is left to overflow.
        TokenType::Divide | TokenType::DivAssign => n.checked_div(m),
        TokenType::Modulo => n.checked_rem(m),
        // A negative exponent would need a fraction, which `Int` can't hold.
        TokenType::Exponent if m < 0 => return Err(RuntimeErrorKind::NegativeExponent),
        TokenType::Exponent => int_pow(n, m),
        _ => unreachable!("only called for arithmetic operators"),
    };
    result.ok_or(RuntimeErrorKind::IntegerOverflow)
}

/// Applies an arithmetic operator to `Float`s.
/// Can't fail: dividing by zero gives `inf` or `NaN`, e.g. `1.0 / 0` is `inf`.
fn float_arithmetic(operator: &Token, n: f64, m: f64) -> f64 {
    match operator.token_type() {
        TokenType::Add | TokenType::AddAssign => n + m,
        TokenType::Subtract | TokenType::SubAssign => n - m,
        TokenType::Multiply | TokenType::MulAssign => n * m,
        TokenType::Divide | TokenType::DivAssign => n / m,
        TokenType::Modulo => n % m,
        TokenType::Exponent => n.powf(m),
        _ => unreachable!("only called for arithmetic operators"),
    }
}

/// Orders two numbers. Mixed operands compare as floats.
fn compare(operator: &Token, numbers: Numbers) -> bool {
    let ordering = match numbers {
        Numbers::Ints(n, m) => n.partial_cmp(&m),
        Numbers::Floats(n, m) => n.partial_cmp(&m),
    };
    let Some(ordering) = ordering else {
        // A NaN has no ordering, so every comparison with it is false.
        return false;
    };
    match operator.token_type() {
        TokenType::Less => ordering.is_lt(),
        TokenType::LessEquals => ordering.is_le(),
        TokenType::More => ordering.is_gt(),
        TokenType::MoreEquals => ordering.is_ge(),
        _ => unreachable!("only called for `<`, `<=`, `>`, and `>=`"),
    }
}

/// Both operands of an arithmetic operator, promoted to a common type.
enum Numbers {
    Ints(i64, i64),
    /// At least one operand was a `Float`, so the `Int` one (if any) is promoted.
    Floats(f64, f64),
}

/// Pairs up two numbers, or `None` if either isn't one.
fn numbers(left: &Value, right: &Value) -> Option<Numbers> {
    match (left, right) {
        (&Value::Int(n), &Value::Int(m)) => Some(Numbers::Ints(n, m)),
        (&Value::Int(n), &Value::Float(m)) => Some(Numbers::Floats(n as f64, m)),
        (&Value::Float(n), &Value::Int(m)) => Some(Numbers::Floats(n, m as f64)),
        (&Value::Float(n), &Value::Float(m)) => Some(Numbers::Floats(n, m)),
        _ => None,
    }
}

/// `n ^ m` for a non-negative `m`, or `None` on overflow.
fn int_pow(n: i64, m: i64) -> Option<i64> {
    match u32::try_from(m) {
        Ok(m) => n.checked_pow(m),
        // Exponents past `u32` only fit when the base is 0, 1, or -1.
        Err(_) => match n {
            0 | 1 => Some(n),
            -1 => Some(if m % 2 == 0 { 1 } else { -1 }),
            _ => None,
        },
    }
}

/// Whether a value counts as true: zero, `null`, `false`, `""`, and `[]` don't.
fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Int(n) => *n != 0,
        Value::Float(f) => *f != 0.0,
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Str(s) => !s.is_empty(),
        Value::List(elements) => !elements.is_empty(),
        Value::Table(table) => !table.is_empty(),
        Value::Native(_) | Value::Function(_) => true,
    }
}

/// Whether two values are equal. Numbers compare by value across `Int` and
/// `Float`, so `1 == 1.0`; any other mix of types is unequal. Lists compare
/// element by element by this same rule, so `[1] == [1.0]`.
fn is_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::List(l), Value::List(r)) => {
            l.len() == r.len() && l.iter().zip(r.iter()).all(|(l, r)| is_equal(l, r))
        }
        (Value::Table(l), Value::Table(r)) => {
            l.names() == r.names()
                && l.columns().iter().zip(r.columns()).all(|(l, r)| {
                    l.len() == r.len() && l.iter().zip(r.iter()).all(|(l, r)| is_equal(l, r))
                })
        }
        _ => match numbers(left, right) {
            Some(Numbers::Ints(n, m)) => n == m,
            Some(Numbers::Floats(n, m)) => n == m,
            None => left == right,
        },
    }
}
