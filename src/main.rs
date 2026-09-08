use std::process::exit;
use crate::scanner::Scanner;

mod token;
mod scanner;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let flag = args.get(1).map(String::as_str);

    match flag {
        Some("--tokenize") => {
            let path = args.get(2).expect("usage: run --tokenize <file>");
            run_tokenize_file(path);
        }
        Some(path) if !path.starts_with("--") => run_program(path),
        _ => {
            eprintln!("Unknown usage");
            exit(64);
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
            exit(1);
        }
    };

    let mut scanner = Scanner::new(source.chars().collect());
    scanner.scan_tokens();

    if scanner.had_error() {
        exit(1);
    }

    let tokens = scanner.get_tokens();

    for token in tokens {
        println!("{}", token);
    }

    exit(0);
}
