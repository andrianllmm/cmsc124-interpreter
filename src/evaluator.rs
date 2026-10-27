//! Tree-walking evaluator that turns expressions into values.

use crate::ast::Expr;
use crate::token::{Token, TokenType};
use crate::value::Value;
use std::fmt::{self, Display, Formatter};

/// An error raised while evaluating.
pub struct RuntimeError<'a> {
    /// Token where evaluation failed, e.g. an operator or a name,
    /// so the message can point at its line.
    pub operator: Token<'a>,
    pub kind: RuntimeErrorKind,
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
    /// `|>` needs functions, which don't exist yet.
    PipeUnsupported,
    // Error raised when truthiness of a value cannot be extracted
    UnknownTruthiness,
    // Error raised when unsupported logical operation is performed
    UnsupportedLogical,
}

impl Display for RuntimeError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            RuntimeErrorKind::UndefinedVariable => {
                format!("Undefined variable '{}'", self.operator.lexeme())
            }
            RuntimeErrorKind::OperandNotNumber => "Operand must be a number".to_string(),
            RuntimeErrorKind::OperandNotString => "Operand must be a string".to_string(),
            RuntimeErrorKind::IntegerOverflow => "Integer overflow".to_string(),
            RuntimeErrorKind::DivisionByZero => "Division by zero".to_string(),
            RuntimeErrorKind::NegativeExponent => "Int exponent must not be negative".to_string(),
            RuntimeErrorKind::PipeUnsupported => {
                "Pipe needs a function, which isn't supported yet".to_string()
            }
            RuntimeErrorKind::UnknownTruthiness => {
                "Truthiness of operand cannot be extacted".to_string()
            }
            RuntimeErrorKind::UnsupportedLogical => {
                "Error performing logical operation due to invalid operand".to_string()
            }
        };
        write!(f, "Error at line {}: {}", self.operator.line(), message)
    }
}

/// Evaluates an expression. Operands are evaluated before their operator
/// (post-order), so each node works with values, never with subtrees.
pub fn evaluate<'a>(expr: &Expr<'a>) -> Result<Value, RuntimeError<'a>> {
    match expr {
        Expr::Literal { value, .. } => Ok(value.clone().into()),
        Expr::Grouping { expression } => evaluate(expression),
        // No bindings exist yet, so every name is undefined.
        Expr::Variable { name } => Err(RuntimeError {
            operator: name.clone(),
            kind: RuntimeErrorKind::UndefinedVariable,
        }),
        Expr::Unary { operator, right } => {
            let right = evaluate(right)?;
            unary(operator, right)
        }
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left = evaluate(left)?;
            let right = evaluate(right)?;
            binary(left, operator, right)
        }
    }
}

fn unary<'a>(operator: &Token<'a>, right: Value) -> Result<Value, RuntimeError<'a>> {
    match operator.token_type() {
        TokenType::Subtract => match right {
            // `i64::MIN` has no positive counterpart, so negating it overflows.
            Value::Int(n) => n.checked_neg().map(Value::Int).ok_or_else(|| RuntimeError {
                operator: operator.clone(),
                kind: RuntimeErrorKind::IntegerOverflow,
            }),
            // Negating a float only flips its sign bit, so it can't overflow.
            Value::Float(n) => Ok(Value::Float(-n)),
            _ => Err(RuntimeError {
                operator: operator.clone(),
                kind: RuntimeErrorKind::OperandNotNumber,
            }),
        },
        TokenType::Not => {
            let flag = is_truthy(right);
            match flag {
                Ok(b) => Ok(Value::Bool(!b)),
                Err(_) => Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::UnknownTruthiness,
                }),
            }
        }
        _ => unreachable!("parser only builds unary `-` and `not`"),
    }
}

fn binary<'a>(left: Value, operator: &Token<'a>, right: Value) -> Result<Value, RuntimeError<'a>> {
    let error = |kind| RuntimeError {
        operator: operator.clone(),
        kind,
    };

    // Both operands are already evaluated, so `|>` can only fail here.
    if let TokenType::Pipe = operator.token_type() {
        return Err(error(RuntimeErrorKind::PipeUnsupported));
    }

    // `++` only joins strings; anything else must be converted first.
    if let TokenType::StringConcat = operator.token_type() {
        return match (left, right) {
            (Value::Str(l), Value::Str(r)) => Ok(Value::Str(l + &r)),
            _ => Err(error(RuntimeErrorKind::OperandNotString)),
        };
    }

    // logical evaluation have a separate implementation here.
    if (TokenType::And == *operator.token_type()) || (TokenType::Or == *operator.token_type()) {
        let eval_result = logical_compare(left, operator, right);
        {
            let b = eval_result?;
            return Ok(Value::Bool(b));
        }
    }

    if (TokenType::Equals == *operator.token_type())
        || (TokenType::NotEquals == *operator.token_type())
    {
        match (&left, operator.token_type(), &right) {
            (Value::Str(l), TokenType::Equals, Value::Str(r)) => return Ok(Value::Bool(l == r)),
            (Value::Str(l), TokenType::NotEquals, Value::Str(r)) => return Ok(Value::Bool(l != r)),
            _ => {}
        }
    }

    let numbers = numbers(left, right).ok_or_else(|| error(RuntimeErrorKind::OperandNotNumber))?;
    match (operator.token_type(), numbers) {
        (TokenType::Add, Numbers::Ints(n, m)) => int(n.checked_add(m), error),
        (TokenType::Add, Numbers::Floats(n, m)) => Ok(Value::Float(n + m)),
        (TokenType::Subtract, Numbers::Ints(n, m)) => int(n.checked_sub(m), error),
        (TokenType::Subtract, Numbers::Floats(n, m)) => Ok(Value::Float(n - m)),
        (TokenType::Multiply, Numbers::Ints(n, m)) => int(n.checked_mul(m), error),
        (TokenType::Multiply, Numbers::Floats(n, m)) => Ok(Value::Float(n * m)),
        // `Int` has no infinity, so dividing by zero is an error. `Float`
        // follows IEEE 754 instead, e.g. `1.0 / 0` is `inf`.
        (TokenType::Divide, Numbers::Ints(_, 0)) => Err(error(RuntimeErrorKind::DivisionByZero)),
        // Only `Int::MIN / -1` is left to overflow.
        (TokenType::Divide, Numbers::Ints(n, m)) => int(n.checked_div(m), error),
        (TokenType::Divide, Numbers::Floats(n, m)) => Ok(Value::Float(n / m)),
        (TokenType::Modulo, Numbers::Ints(_, 0)) => Err(error(RuntimeErrorKind::DivisionByZero)),
        (TokenType::Modulo, Numbers::Ints(n, m)) => int(n.checked_rem(m), error),
        (TokenType::Modulo, Numbers::Floats(n, m)) => Ok(Value::Float(n % m)),
        // A negative exponent would need a fraction, which `Int` can't hold.
        (TokenType::Exponent, Numbers::Ints(_, m)) if m < 0 => {
            Err(error(RuntimeErrorKind::NegativeExponent))
        }
        (TokenType::Exponent, Numbers::Ints(n, m)) => int(int_pow(n, m), error),
        (TokenType::Exponent, Numbers::Floats(n, m)) => Ok(Value::Float(n.powf(m))),
        // Mixed operands compare as floats; a NaN makes every comparison false.
        (TokenType::Less, Numbers::Ints(n, m)) => Ok(Value::Bool(n < m)),
        (TokenType::Less, Numbers::Floats(n, m)) => Ok(Value::Bool(n < m)),
        (TokenType::LessEquals, Numbers::Ints(n, m)) => Ok(Value::Bool(n <= m)),
        (TokenType::LessEquals, Numbers::Floats(n, m)) => Ok(Value::Bool(n <= m)),
        (TokenType::More, Numbers::Ints(n, m)) => Ok(Value::Bool(n > m)),
        (TokenType::More, Numbers::Floats(n, m)) => Ok(Value::Bool(n > m)),
        (TokenType::MoreEquals, Numbers::Ints(n, m)) => Ok(Value::Bool(n >= m)),
        (TokenType::MoreEquals, Numbers::Floats(n, m)) => Ok(Value::Bool(n >= m)),
        (TokenType::Equals, Numbers::Ints(n, m)) => Ok(Value::Bool(n == m)),
        (TokenType::Equals, Numbers::Floats(n, m)) => Ok(Value::Bool(n == m)),
        (TokenType::NotEquals, Numbers::Ints(n, m)) => Ok(Value::Bool(n != m)),
        (TokenType::NotEquals, Numbers::Floats(n, m)) => Ok(Value::Bool(n != m)),
        _ => unreachable!("token type unrecognized"),
    }
}

/// Both operands of an arithmetic operator, promoted to a common type.
enum Numbers {
    Ints(i64, i64),
    /// At least one operand was a `Float`, so the `Int` one (if any) is promoted.
    Floats(f64, f64),
}

/// Pairs up two numbers, or `None` if either isn't one.
fn numbers(left: Value, right: Value) -> Option<Numbers> {
    match (left, right) {
        (Value::Int(n), Value::Int(m)) => Some(Numbers::Ints(n, m)),
        (Value::Int(n), Value::Float(m)) => Some(Numbers::Floats(n as f64, m)),
        (Value::Float(n), Value::Int(m)) => Some(Numbers::Floats(n, m as f64)),
        (Value::Float(n), Value::Float(m)) => Some(Numbers::Floats(n, m)),
        _ => None,
    }
}

/// Wraps a checked `Int` result, treating `None` as overflow.
fn int<'a>(
    result: Option<i64>,
    error: impl Fn(RuntimeErrorKind) -> RuntimeError<'a>,
) -> Result<Value, RuntimeError<'a>> {
    result
        .map(Value::Int)
        .ok_or_else(|| error(RuntimeErrorKind::IntegerOverflow))
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

fn is_truthy<'a>(val: Value) -> Result<bool, RuntimeError<'a>> {
    match val {
        Value::Int(n) => {
            if n == 0 {
                Ok(false)
            } else {
                Ok(true)
            }
        }
        Value::Float(f) => {
            if f == 0.0 {
                Ok(false)
            } else {
                Ok(true)
            }
        }
        Value::Null => Ok(false),
        Value::Bool(b) => Ok(b),
        Value::Str(str) => {
            if str.is_empty() {
                Ok(false)
            } else {
                Ok(true)
            }
        }
    }
}

fn logical_compare<'a>(
    left: Value,
    operator: &Token<'a>,
    right: Value,
) -> Result<bool, RuntimeError<'a>> {
    match operator.token_type() {
        TokenType::And => {
            let left_eval = is_truthy(left)?;
            let right_eval = is_truthy(right)?;
            match (left_eval, right_eval) {
                (true, true) => Ok(true),
                (false, _) => Ok(false),
                (_, false) => Ok(false),
            }
        }
        TokenType::Or => {
            let left_eval = is_truthy(left)?;
            let right_eval = is_truthy(right)?;
            match (left_eval, right_eval) {
                (true, _) => Ok(true),
                (_, true) => Ok(true),
                (false, false) => Ok(false),
            }
        }
        _ => Err(RuntimeError {
            operator: operator.clone(),
            kind: RuntimeErrorKind::UnsupportedLogical,
        }),
    }
}
