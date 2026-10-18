//! Tree-walking evaluator that turns expressions into values.

use crate::ast::Expr;
use crate::token::{Token, TokenType};
use crate::value::Value;
use std::fmt::{self, Display, Formatter};

/// An error raised while evaluating.
pub struct RuntimeError<'a> {
    /// Operator that failed, so the message can point at its line.
    pub operator: Token<'a>,
    pub kind: RuntimeErrorKind,
}

/// What went wrong while evaluating.
pub enum RuntimeErrorKind {
    /// A numeric operator got a non-number, e.g. `-"a"`.
    OperandNotNumber,
    /// The result doesn't fit in an `Int`, e.g. negating its minimum.
    IntegerOverflow,
    /// An `Int` divided by zero, with `/` or `%`.
    DivisionByZero,
    /// An `Int` raised to a negative `Int`, e.g. `2 ^ -1`.
    NegativeExponent,
    /// `|>` needs functions, which don't exist yet.
    PipeUnsupported,
}

impl Display for RuntimeError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            RuntimeErrorKind::OperandNotNumber => "Operand must be a number",
            RuntimeErrorKind::IntegerOverflow => "Integer overflow",
            RuntimeErrorKind::DivisionByZero => "Division by zero",
            RuntimeErrorKind::NegativeExponent => "Int exponent must not be negative",
            RuntimeErrorKind::PipeUnsupported => "Pipe needs a function, which isn't supported yet",
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
        TokenType::Not => todo!("`not` needs a truthiness rule"),
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
