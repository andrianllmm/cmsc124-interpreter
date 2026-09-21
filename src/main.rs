use crate::scanner::Scanner;
use std::process::exit;

mod keyword;
mod repl;
mod scanner;
mod token;

const EX_USAGE: i32 = 64;
const EX_DATAERR: i32 = 65;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let flag = args.get(1).map(String::as_str);

    match flag {
        Some("--tokenize") => {
            let path = args.get(2).expect("usage: run --tokenize <file>");
            run_tokenize_file(path);
        }
        Some(path) if !path.starts_with("--") => run_program(path),
        None => repl::run(),
        _ => {
            eprintln!("Unknown usage");
            exit(EX_USAGE);
        }
    }
}

fn run_program(_path: &str) {
    // TODO: replace with real execution once the interpreter exists (Lab 4)
    println!("cmsc124-interpreter");
    println!("Members:");
    println!("Andrian Lloyd M. Maagma");
    println!("Julian Hanns T. Medalla");
}

fn run_tokenize_file(path: &str) {
    let source = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            exit(EX_DATAERR);
        }
    };

    let mut scanner = Scanner::new(source.chars().collect());

    match scanner.scan_tokens() {
        Ok(tokens) => {
            for token in tokens {
                println!("{}", token);
            }
            exit(0);
        }
        Err(errors) => {
            for error in errors {
                eprintln!("{}", error);
            }
            exit(EX_DATAERR);
        }
    }
}
