//! Runs source through each stage. Shared by file mode and the REPL.

use crate::ast::Expr;
use crate::ast_printer;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::stage::Stage;
use crate::token::Token;

/// Runs source through each stage, stopping after `stage` to print its output.
/// With no `stage`, runs everything.
///
/// Errors are already printed by the time `Err` returns,
/// so callers only decide whether to exit or keep going.
pub fn run(source: &str, stage: Option<Stage>) -> Result<(), ()> {
    let tokens = scan(source)?;
    if let Some(Stage::Tokenize) = stage {
        for token in &tokens {
            println!("{}", token);
        }
        return Ok(());
    }

    let exprs = parse(tokens)?;
    if let Some(Stage::Parse) = stage {
        for expr in &exprs {
            println!("{}", ast_printer::print(expr));
        }
        return Ok(());
    }

    // TODO: Evaluate and print the result once the interpreter exists.
    Ok(())
}

fn scan(source: &str) -> Result<Vec<Token<'_>>, ()> {
    let mut scanner = Scanner::new(source);
    match scanner.scan_tokens() {
        Ok(tokens) => Ok(tokens.clone()),
        Err(errors) => {
            for error in errors {
                eprintln!("{}", error);
            }
            Err(())
        }
    }
}

fn parse(tokens: Vec<Token<'_>>) -> Result<Vec<Expr<'_>>, ()> {
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(exprs) => Ok(exprs.clone()),
        Err(errors) => {
            for error in errors {
                eprintln!("{}", error);
            }
            Err(())
        }
    }
}
