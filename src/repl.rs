use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::stage::Stage;
use crate::token::Token;
use std::io::{self, Write};

pub fn run(stage: Option<Stage>) {
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("> ");
        // print! isn't line-buffered; without this the prompt won't show before read_line blocks
        io::stdout().flush().unwrap();

        // read_line appends, so leftover text from the previous line must be cleared
        line.clear();
        let bytes_read = match stdin.read_line(&mut line) {
            Ok(n) => n, // 0 = EOF (Ctrl-D); an empty line is still 1 (the newline)
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        };
        if bytes_read == 0 {
            break;
        }

        run_line(&line, stage);
    }
}

fn run_line(line: &str, stage: Option<Stage>) {
    let mut scanner = Scanner::new(line);
    let tokens = match scanner.scan_tokens() {
        Ok(tokens) => tokens.clone(),
        Err(errors) => {
            for error in errors {
                eprintln!("{}", error);
            }
            return;
        }
    };
    match stage {
        Some(Stage::Tokenize) => print_tokens(&tokens),
        Some(Stage::Parse) => print_parse(tokens),
        // TODO: evaluate and print the result once it exists
        None => {}
    }
}

fn print_tokens(tokens: &[Token]) {
    for token in tokens {
        println!("{}", token);
    }
}

fn print_parse(tokens: Vec<Token>) {
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(expr) => println!("{:?}", expr),
        Err(error) => eprintln!("{}", error),
    }
}
