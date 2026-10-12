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
    match operator.token_type() {
        TokenType::Add => {
            if !number_check(&left, &right) {
                return Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::OperandNotNumber,
                });
            }

            // TODO: Implement overflow checks
            match (left, right) {
                (Value::Int(n), Value::Int(m)) => Ok(Value::Int(n + m)),
                (Value::Int(n), Value::Float(m)) => Ok(Value::Float(n as f64 + m)),
                (Value::Float(n), Value::Int(m)) => Ok(Value::Float(n + m as f64)),
                (Value::Float(n), Value::Float(m)) => Ok(Value::Float(n + m)),
                _ => Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::OperandNotNumber,
                }),
            }
        }
        TokenType::Subtract => {
            if !number_check(&left, &right) {
                return Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::OperandNotNumber,
                });
            }

            // TODO: Implement overflow checks
            match (left, right) {
                (Value::Int(n), Value::Int(m)) => Ok(Value::Int(n - m)),
                (Value::Int(n), Value::Float(m)) => Ok(Value::Float(n as f64 - m)),
                (Value::Float(n), Value::Int(m)) => Ok(Value::Float(n - m as f64)),
                (Value::Float(n), Value::Float(m)) => Ok(Value::Float(n - m)),
                _ => Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::OperandNotNumber,
                }),
            }
        }
        TokenType::Multiply => {
            if !number_check(&left, &right) {
                return Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::OperandNotNumber,
                });
            }

            // TODO: Implement overflow checks
            match (left, right) {
                (Value::Int(n), Value::Int(m)) => Ok(Value::Int(n * m)),
                (Value::Int(n), Value::Float(m)) => Ok(Value::Float(n as f64 * m)),
                (Value::Float(n), Value::Int(m)) => Ok(Value::Float(n * m as f64)),
                (Value::Float(n), Value::Float(m)) => Ok(Value::Float(n * m)),
                _ => Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::OperandNotNumber,
                }),
            }
        }
        TokenType::Divide => {
            if !number_check(&left, &right) {
                return Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::OperandNotNumber,
                });
            }

            // TODO: Implement overflow checks
            match (left, right) {
                (Value::Int(n), Value::Int(m)) => Ok(Value::Int(n / m)),
                (Value::Int(n), Value::Float(m)) => Ok(Value::Float(n as f64 / m)),
                (Value::Float(n), Value::Int(m)) => Ok(Value::Float(n / m as f64)),
                (Value::Float(n), Value::Float(m)) => Ok(Value::Float(n / m)),
                _ => Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::OperandNotNumber,
                }),
            }
        }
        TokenType::Modulo => {
            if !number_check(&left, &right) {
                return Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::OperandNotNumber,
                });
            }

            // TODO: Implement overflow checks
            match (left, right) {
                (Value::Int(n), Value::Int(m)) => Ok(Value::Int(n % m)),
                (Value::Int(n), Value::Float(m)) => Ok(Value::Float(n as f64 % m)),
                (Value::Float(n), Value::Int(m)) => Ok(Value::Float(n % m as f64)),
                (Value::Float(n), Value::Float(m)) => Ok(Value::Float(n % m)),
                _ => Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::OperandNotNumber,
                }),
            }
        }
        TokenType::Exponent => {
            if !number_check(&left, &right) {
                return Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::OperandNotNumber,
                });
            }

            // TODO: Implement overflow checks
            match (left, right) {
                (Value::Int(n), Value::Int(m)) => Ok(Value::Int(n.pow(m.try_into().unwrap()))),
                (Value::Int(n), Value::Float(m)) => Ok(Value::Float((n as f64).powf(m))),
                (Value::Float(n), Value::Int(m)) => Ok(Value::Float(n.powf(m as f64))),
                (Value::Float(n), Value::Float(m)) => Ok(Value::Float(n.powf(m))),
                _ => Err(RuntimeError {
                    operator: operator.clone(),
                    kind: RuntimeErrorKind::OperandNotNumber,
                }),
            }
        }
        _ => unreachable!("token type unrecognized"),
    }
}

fn number_check(left: &Value, right: &Value) -> bool {
    let left_flag = matches!(left, Value::Int(_) | Value::Float(_));
    let right_flag = matches!(right, Value::Int(_) | Value::Float(_));

    left_flag && right_flag
}
