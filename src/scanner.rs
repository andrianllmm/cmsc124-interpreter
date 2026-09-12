use crate::token::{Token, TokenType};

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: i32,
    had_error: bool,
}

impl Scanner {
    // Builds a scanner positioned at the start of `source`.
    pub fn new(source: Vec<char>) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            had_error: false,
        }
    }

    // Get the tokens that have been scanned.
    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    // Scans one lexeme per iteration until the source is exhausted.
    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            // Set start of the current token.
            self.start = self.current;
            // Scan the current token.
            self.scan_token();
        }

        // Append EOF.
        self.tokens
            .push(Token::new(TokenType::Eof, "", "", self.line));

        &self.tokens
    }

    // Consumes one lexeme starting at `current` and emits its token.
    fn scan_token(&mut self) {
        let c:char = self.advance();
        match c {
            '(' => self.add_token(TokenType::LParen),
            ')' => self.add_token(TokenType::RParen),
            '+' => self.add_token(TokenType::Add),
            '-' => self.add_token(TokenType::Subtract),
            '*' => self.add_token(TokenType::Multiply),
            '/' => self.add_token(TokenType::Divide),
            '=' => {
                if self.peek() == '=' {
                    _ = self.advance();
                    self.add_token(TokenType::Equals);
                } else {
                    self.add_token(TokenType::Assign);
                }
            }
            '>' => { 
                if self.peek() == '=' {
                    _ = self.advance();
                    self.add_token(TokenType::MoreEquals);
                } else {
                    self.add_token(TokenType::More);
                }
            }
            '<' => {
                if self.peek() == '=' {
                    _ = self.advance();
                    self.add_token(TokenType::LessEquals);
                } else {
                    self.add_token(TokenType::Less);
                }
            },

            // Skip whitespace.
            ' ' | '\t' | '\r' => {}
            // Increment line number.
            '\n' => self.line += 1,
            // Error on unexpected character.
            _ => self.error(self.line, "Unexpected character"),
        }
    }

    // Consumes the current character and advances `current` past it.
    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    // peeks at the next character
    fn peek(&mut self) -> char {
        // self.advance() causes self.current to point to next character
        return self.source[self.current];
    }

    // Records a finished token spanning `start..current`.
    fn add_token(&mut self, token_type: TokenType) {
        // Get the lexeme.
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        // Add the token.
        self.tokens
            .push(Token::new(token_type, &lexeme, "", self.line));
    }

    // True once `current` has passed the end of `source`.
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len() - 1
    }

    // True if the scan encountered an error.
    pub fn had_error(&self) -> bool {
        self.had_error
    }

    // Reports a lexical error on stderr and marks the scan as failed, without halting it.
    fn error(&mut self, line: i32, message: &str) {
        eprintln!("Error at line {}: {}", line, message);
        self.had_error = true;
    }
}
