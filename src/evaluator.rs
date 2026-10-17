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
}

impl Display for RuntimeError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            RuntimeErrorKind::OperandNotNumber => "Operand must be a number",
            RuntimeErrorKind::IntegerOverflow => "Integer overflow",
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

    let numbers = numbers(left, right).ok_or_else(|| error(RuntimeErrorKind::OperandNotNumber))?;
    // TODO: Implement overflow checks
    match (operator.token_type(), numbers) {
        (TokenType::Add, Numbers::Ints(n, m)) => Ok(Value::Int(n + m)),
        (TokenType::Add, Numbers::Floats(n, m)) => Ok(Value::Float(n + m)),
        (TokenType::Subtract, Numbers::Ints(n, m)) => Ok(Value::Int(n - m)),
        (TokenType::Subtract, Numbers::Floats(n, m)) => Ok(Value::Float(n - m)),
        (TokenType::Multiply, Numbers::Ints(n, m)) => Ok(Value::Int(n * m)),
        (TokenType::Multiply, Numbers::Floats(n, m)) => Ok(Value::Float(n * m)),
        (TokenType::Divide, Numbers::Ints(n, m)) => Ok(Value::Int(n / m)),
        (TokenType::Divide, Numbers::Floats(n, m)) => Ok(Value::Float(n / m)),
        (TokenType::Modulo, Numbers::Ints(n, m)) => Ok(Value::Int(n % m)),
        (TokenType::Modulo, Numbers::Floats(n, m)) => Ok(Value::Float(n % m)),
        (TokenType::Exponent, Numbers::Ints(n, m)) => Ok(Value::Int(n.pow(m.try_into().unwrap()))),
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
