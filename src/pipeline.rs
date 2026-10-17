//! Runs source through each stage. Shared by file mode and the REPL.

use crate::ast::Program;
use crate::ast_printer;
use crate::evaluator;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::stage::Stage;
use crate::token::Token;

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
pub fn run(source: &str, stage: Option<Stage>) -> Result<(), Failure> {
    let tokens = scan(source)?;
    if let Some(Stage::Tokenize) = stage {
        for token in &tokens {
            println!("{}", token);
        }
        return Ok(());
    }

    let program = parse(tokens)?;
    if let Some(Stage::Parse) = stage {
        for expr in &program.statements {
            println!("{}", ast_printer::print(expr));
        }
        return Ok(());
    }

    if let Some(Stage::Eval) = stage {
        return evaluate(&program);
    }

    // TODO: Execute statements once the interpreter exists (Lab 4).
    Ok(())
}

fn scan(source: &str) -> Result<Vec<Token<'_>>, Failure> {
    let mut scanner = Scanner::new(source);
    match scanner.scan_tokens() {
        Ok(tokens) => Ok(tokens.clone()),
        Err(errors) => {
            for error in errors {
                eprintln!("{}", error);
            }
            Err(Failure::Static)
        }
    }
}

fn parse(tokens: Vec<Token<'_>>) -> Result<Program<'_>, Failure> {
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(program) => Ok(program.clone()),
        Err(errors) => {
            for error in errors {
                eprintln!("{}", error);
            }
            Err(Failure::Static)
        }
    }
}

/// Prints each expression's value, stopping at the first runtime error.
fn evaluate(program: &Program<'_>) -> Result<(), Failure> {
    for expr in &program.statements {
        match evaluator::evaluate(expr) {
            Ok(value) => println!("{}", value),
            Err(error) => {
                eprintln!("{}", error);
                return Err(Failure::Runtime);
            }
        }
    }
    Ok(())
}
