//! Interactive prompt.

use crate::pipeline;
use crate::stage::Stage;
use std::io::{self, Write};

/// Runs input line by line until EOF. Errors are printed and the session keeps going.
pub fn run(stage: Option<Stage>) {
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("> ");
        // Stdout is line-buffered and the prompt has no newline,
        // so flush it before `read_line` blocks.
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

        let _ = pipeline::run(&line, stage);
    }
}
