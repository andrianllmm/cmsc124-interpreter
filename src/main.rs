//! Interpreter for Grizzly, a small language for transforming tabular data.
//!
//! Runs a file, or starts a REPL when no file is given. `--tokenize`,
//! `--parse`, and `--eval` stop after that stage and print its output.

use crate::pipeline::Failure;
use crate::stage::Stage;
use std::process::exit;

mod ast;
mod ast_printer;
mod evaluator;
mod keyword;
mod native;
mod parser;
mod pipeline;
mod repl;
mod scanner;
mod stage;
mod token;
mod value;

// Exit codes follow BSD `sysexits.h`.
const EX_USAGE: i32 = 64;
const EX_DATAERR: i32 = 65;
const EX_NOINPUT: i32 = 66;
const EX_SOFTWARE: i32 = 70;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let flag = args.get(1).map(String::as_str);

    match flag {
        Some("--tokenize") => match args.get(2) {
            Some(path) => run_file(path, Some(Stage::Tokenize)),
            None => repl::run(Some(Stage::Tokenize)),
        },
        Some("--parse") => match args.get(2) {
            Some(path) => run_file(path, Some(Stage::Parse)),
            None => repl::run(Some(Stage::Parse)),
        },
        Some("--eval") => match args.get(2) {
            Some(path) => run_file(path, Some(Stage::Eval)),
            None => repl::run(Some(Stage::Eval)),
        },
        Some(path) if !path.starts_with("--") => run_file(path, None),
        // Until statements exist (Lab 4), the REPL prints each expression's value.
        None => repl::run(Some(Stage::Eval)),
        _ => {
            eprintln!("Unknown usage");
            exit(EX_USAGE);
        }
    }
}

/// Runs a file, stopping after `stage`, or executing it with no `stage`.
fn run_file(path: &str, stage: Option<Stage>) {
    let source = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            exit(EX_NOINPUT);
        }
    };

    match pipeline::run(&source, stage) {
        Ok(()) => exit(0),
        Err(Failure::Static) => exit(EX_DATAERR),
        Err(Failure::Runtime) => exit(EX_SOFTWARE),
    }
}
