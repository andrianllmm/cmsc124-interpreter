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

    // Scans one lexeme per iteration until the source is exhausted.
    // Returns the tokens, or `Err` if any lexical error was reported along the way.
    pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, ()> {
        while !self.is_at_end() {
            // Set start of the current token.
            self.start = self.current;
            // Scan the current token.
            self.scan_token();
        }

        // Append EOF.
        self.tokens
            .push(Token::new(TokenType::Eof, "", "", self.line));

        if self.had_error {
            Err(())
        } else {
            Ok(&self.tokens)
        }
    }

    // Consumes one lexeme starting at `current` and emits its token.
    fn scan_token(&mut self) {
        let c: char = self.advance();
        match c {
            '(' => self.add_token(TokenType::LParen),
            ')' => self.add_token(TokenType::RParen),
            '{' => self.add_token(TokenType::LBrace),
            '}' => self.add_token(TokenType::RBrace),
            ';' => self.add_token(TokenType::Terminator),
            '+' => {
                if self.match_expected('=') {
                    self.add_token(TokenType::AddAssign);
                } else if self.match_expected('+') {
                    self.add_token(TokenType::StringConcat);
                } else {
                    self.add_token(TokenType::Add);
                }
            }
            '-' => {
                if self.match_expected('=') {
                    self.add_token(TokenType::SubAssign);
                } else if self.match_expected('>') {
                    self.add_token(TokenType::LambdaArrow);
                } else {
                    self.add_token(TokenType::Subtract);
                }
            }
            '*' => {
                if self.match_expected('=') {
                    self.add_token(TokenType::MulAssign);
                } else {
                    self.add_token(TokenType::Multiply);
                }
            }
            '/' => {
                if self.match_expected('=') {
                    self.add_token(TokenType::DivAssign);
                } else {
                    self.add_token(TokenType::Divide);
                }
            }
            '^' => self.add_token(TokenType::Exponent),
            '=' => {
                if self.match_expected('=') {
                    self.add_token(TokenType::Equals);
                } else if self.match_expected('>') {
                    self.add_token(TokenType::MatchArrow);
                } else {
                    self.add_token(TokenType::Assign);
                }
            }
            '>' => {
                if self.match_expected('=') {
                    self.add_token(TokenType::MoreEquals);
                } else {
                    self.add_token(TokenType::More);
                }
            }
            '<' => {
                if self.match_expected('=') {
                    self.add_token(TokenType::LessEquals);
                } else {
                    self.add_token(TokenType::Less);
                }
            }
            '!' => {
                if self.match_expected('=') {
                    self.add_token(TokenType::NotEquals);
                } else {
                    self.error(self.line, "Missing \'=\'")
                }
            }
            '|' => {
                if self.match_expected('>') {
                    self.add_token(TokenType::Pipe);
                } else if self.match_expected('=') {
                    self.add_token(TokenType::PipeAssign);
                } else {
                    self.error(self.line, "Missing \'>\' or \'=\'");
                }
            }

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
    fn peek(&self) -> char {
        // return sentinel if at end
        if self.is_at_end() {
            return '\0';
        }
        // self.advance() causes self.current to point to next character
        // use self.current instead of self.current + 1
        self.source[self.current]
    }

    // matches the next character
    fn match_expected(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    // Records a finished token spanning `start..current`.
    fn add_token_literal(&mut self, token_type: TokenType, literal: &str) {
        // Get the lexeme.
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        // Add the token.
        self.tokens
            .push(Token::new(token_type, &lexeme, literal, self.line));
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, "");
    }

    // True once `current` has passed the end of `source`.
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    // Reports a lexical error on stderr and marks the scan as failed, without halting it.
    fn error(&mut self, line: i32, message: &str) {
        eprintln!("Error at line {}: {}", line, message);
        self.had_error = true;
    }
}
