use crate::scanner::Scanner;
use std::io::{self, Write};

pub fn run() {
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

        let mut scanner = Scanner::new(line.chars().collect());

        match scanner.scan_tokens() {
            Ok(tokens) => {
                for token in tokens {
                    println!("{}", token);
                }
            }
            Err(errors) => {
                for error in errors {
                    eprintln!("{}", error);
                }
            }
        }
    }
}
