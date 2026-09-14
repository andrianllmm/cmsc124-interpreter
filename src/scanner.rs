use crate::token::{Token, TokenType};
use std::fmt::{self, Display, Formatter};

// A lexical error encountered while scanning.
pub struct ScanError {
    pub line: u32,
    pub kind: ScanErrorKind,
}

pub enum ScanErrorKind {
    UnexpectedChar(char),
    MissingOneOf(&'static [char]),
}

impl Display for ScanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            ScanErrorKind::UnexpectedChar(c) => format!("Unexpected character '{}'", c),
            ScanErrorKind::MissingOneOf(chars) => {
                let opts: Vec<String> = chars.iter().map(|c| format!("'{}'", c)).collect();
                format!("Missing {}", opts.join(" or "))
            }
        };
        write!(f, "Error at line {}: {}", self.line, message)
    }
}

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    errors: Vec<ScanError>,
    start: usize,
    current: usize,
    line: u32,
}

impl Scanner {
    pub fn new(source: Vec<char>) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            errors: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    // Scans one lexeme per iteration until the source is exhausted.
    // Returns the tokens, or every lexical error collected along the way.
    pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, &Vec<ScanError>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "", "", self.line));

        if self.errors.is_empty() {
            Ok(&self.tokens)
        } else {
            Err(&self.errors)
        }
    }

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
                    self.error(ScanErrorKind::MissingOneOf(&['=']));
                }
            }
            '|' => {
                if self.match_expected('>') {
                    self.add_token(TokenType::Pipe);
                } else if self.match_expected('=') {
                    self.add_token(TokenType::PipeAssign);
                } else {
                    self.error(ScanErrorKind::MissingOneOf(&['>', '=']));
                }
            }
            '#' => {
                while !self.match_expected('\n') && !self.is_at_end() {
                    self.advance();
                }
            }

            ' ' | '\t' | '\r' => {}
            '\n' => self.line += 1,
            _ => self.error(ScanErrorKind::UnexpectedChar(c)),
        }
    }

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

    fn error(&mut self, kind: ScanErrorKind) {
        self.errors.push(ScanError {
            line: self.line,
            kind,
        });
    }
}
