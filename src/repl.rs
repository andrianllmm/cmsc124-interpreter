//! Interactive prompt.

use crate::evaluator::Interpreter;
use crate::pipeline;
use crate::scanner::Scanner;
use crate::stage::Stage;
use crate::token::TokenType;
use std::io::{self, Write};

/// Shown when the REPL waits for a new entry.
const PROMPT: &str = "> ";
/// Shown while the entry so far has an unclosed bracket.
const CONTINUATION_PROMPT: &str = "... ";

/// Runs input entry by entry until EOF.
/// An entry spans lines while a bracket is unclosed.
///
/// Errors are printed and the session keeps going.
pub fn run(stage: Option<Stage>) {
    // One interpreter for the whole session, so state survives across entries.
    let mut interpreter = Interpreter::new();

    while let Some(entry) = read_entry() {
        let _ = pipeline::run(&mut interpreter, &entry, stage);
    }
}

/// Reads lines until they form a complete entry.
///
/// An empty line or EOF ends an incomplete entry early, so running it
/// reports the syntax error. Returns `None` on EOF at a fresh prompt.
fn read_entry() -> Option<String> {
    let mut entry = String::new();

    loop {
        let prompt = if entry.is_empty() {
            PROMPT
        } else {
            CONTINUATION_PROMPT
        };
        print!("{}", prompt);
        // Stdout is line-buffered and the prompt has no newline,
        // so flush it before `read_line` blocks.
        io::stdout().flush().unwrap();

        // `read_line` appends, so the entry grows line by line.
        let line_start = entry.len();
        let bytes_read = match io::stdin().read_line(&mut entry) {
            Ok(n) => n, // 0 = EOF (Ctrl-D); an empty line is still 1 (the newline)
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                return None;
            }
        };
        if bytes_read == 0 && entry.is_empty() {
            return None;
        }

        // A blank line (or EOF) gives up on an incomplete entry.
        let gave_up = entry[line_start..].trim().is_empty();
        if gave_up || is_complete(&entry) {
            return Some(entry);
        }
    }
}

/// Whether `source` has no unclosed `(`, `[`, or `{`.
///
/// Brackets inside strings and comments don't count, since the scanner skips them.
/// Source the scanner rejects counts as complete, so its error is reported right away.
/// That includes an open string, since strings can't span lines.
fn is_complete(source: &str) -> bool {
    let Ok(tokens) = Scanner::new(source).scan_tokens() else {
        return true;
    };
    let depth: i32 = tokens
        .iter()
        .map(|token| match token.token_type() {
            TokenType::LParen | TokenType::LBracket | TokenType::LBrace => 1,
            TokenType::RParen | TokenType::RBracket | TokenType::RBrace => -1,
            _ => 0,
        })
        .sum();
    // A stray closer leaves the depth below zero; the parser reports it.
    depth <= 0
}
