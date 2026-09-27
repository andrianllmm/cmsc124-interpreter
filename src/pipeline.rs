use crate::ast::Expr;
use crate::ast_printer;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::stage::Stage;
use crate::token::Token;

// Runs source through the pipeline, stopping after `stage` to print its output.
// Err means errors were reported; callers decide whether to exit or keep going.
pub fn run(source: &str, stage: Option<Stage>) -> Result<(), ()> {
    let tokens = scan(source)?;
    if let Some(Stage::Tokenize) = stage {
        for token in &tokens {
            println!("{}", token);
        }
        return Ok(());
    }

    let expr = parse(tokens)?;
    if let Some(Stage::Parse) = stage {
        println!("{}", ast_printer::print(&expr));
        return Ok(());
    }

    // TODO: evaluate and print the result once the interpreter exists
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

fn parse(tokens: Vec<Token>) -> Result<Expr, ()> {
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(expr) => Ok(expr),
        Err(error) => {
            eprintln!("{}", error);
            Err(())
        }
    }
}
