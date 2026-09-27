//! Interpreter for Grizzly, a small language for transforming tabular data.
//!
//! Runs a file, or starts a REPL when no file is given. `--tokenize` and
//! `--parse` stop after that stage and print its output.

use crate::stage::Stage;
use std::process::exit;

mod ast;
mod ast_printer;
mod keyword;
mod parser;
mod pipeline;
mod repl;
mod scanner;
mod stage;
mod token;

// Exit codes follow BSD `sysexits.h`.
const EX_USAGE: i32 = 64;
const EX_DATAERR: i32 = 65;
const EX_NOINPUT: i32 = 66;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let flag = args.get(1).map(String::as_str);

    match flag {
        Some("--tokenize") => match args.get(2) {
            Some(path) => run_file(path, Stage::Tokenize),
            None => repl::run(Some(Stage::Tokenize)),
        },
        Some("--parse") => match args.get(2) {
            Some(path) => run_file(path, Stage::Parse),
            None => repl::run(Some(Stage::Parse)),
        },
        Some(path) if !path.starts_with("--") => run_program(path),
        None => repl::run(None),
        _ => {
            eprintln!("Unknown usage");
            exit(EX_USAGE);
        }
    }
}

fn run_program(_path: &str) {
    // TODO: Replace with real execution once the interpreter exists (Lab 4).
    println!("cmsc124-interpreter");
    println!("Members:");
    println!("Andrian Lloyd M. Maagma");
    println!("Julian Hanns T. Medalla");
}

fn run_file(path: &str, stage: Stage) {
    let source = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            exit(EX_NOINPUT);
        }
    };

    match pipeline::run(&source, Some(stage)) {
        Ok(()) => exit(0),
        Err(()) => exit(EX_DATAERR),
    }
}
