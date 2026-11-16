//! Runs source through each stage. Shared by file mode and the REPL.

use crate::ast::{Expr, Program, Stmt};
use crate::ast_printer;
use crate::evaluator::Interpreter;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::stage::Stage;
use crate::token::Token;
use crate::value::Value;
use std::fmt::Display;

/// Why the pipeline stopped. Decides the exit code in file mode.
pub enum Failure {
    /// Rejected before running (lexical or syntax error).
    Static,
    /// Started running, then hit a runtime error.
    Runtime,
}

/// Runs source through each stage, stopping after `stage` to print its output.
/// With no `stage`, runs everything.
///
/// Errors are already printed by the time `Err` returns,
/// so callers only decide whether to exit or keep going.
pub fn run(
    interpreter: &mut Interpreter,
    source: &str,
    stage: Option<Stage>,
) -> Result<(), Failure> {
    let tokens = scan(source)?;
    if let Some(Stage::Tokenize) = stage {
        for token in &tokens {
            println!("{}", token);
        }
        return Ok(());
    }

    let program = parse(tokens)?;
    if let Some(Stage::Parse) = stage {
        for stmt in &program.statements {
            println!("{}", ast_printer::print_stmt(stmt));
        }
        return Ok(());
    }

    // `--eval` prints every value; a program only prints through `print`.
    evaluate(interpreter, &program, matches!(stage, Some(Stage::Eval)))
}

fn scan(source: &str) -> Result<Vec<Token>, Failure> {
    Scanner::new(source).scan_tokens().map_err(report)
}

fn parse(tokens: Vec<Token>) -> Result<Program, Failure> {
    Parser::new(tokens).parse().map_err(report)
}

/// Prints every static error, then marks the run as rejected before running.
fn report<E: Display>(errors: Vec<E>) -> Failure {
    for error in errors {
        eprintln!("{}", error);
    }
    Failure::Static
}

/// Executes each statement, stopping at the first runtime error.
/// With `print_values`, also prints each expression statement's value,
/// except a call's `null`.
fn evaluate(
    interpreter: &mut Interpreter,
    program: &Program,
    print_values: bool,
) -> Result<(), Failure> {
    for stmt in &program.statements {
        match interpreter.execute(stmt) {
            Ok(Some(value)) if print_values && echoes(stmt, &value) => println!("{}", value),
            Ok(_) => {}
            Err(error) => {
                eprintln!("{}", error);
                return Err(Failure::Runtime);
            }
        }
    }
    Ok(())
}

/// A call made for its effect, like `print(x);`, returns `null`.
/// Echoing it would print a stray `null` after the call's own output.
fn echoes(stmt: &Stmt, value: &Value) -> bool {
    !matches!(
        (stmt, value),
        (Stmt::Expression(Expr::Call { .. }), Value::Null)
    )
}
