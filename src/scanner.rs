//! Turns source text into tokens.

use crate::keyword::keyword_type;
use crate::token::{Token, TokenType};
use std::fmt::{self, Display, Formatter};

/// A lexical error encountered while scanning.
pub struct ScanError {
    pub line: u32,
    pub kind: ScanErrorKind,
}

/// What went wrong while scanning.
pub enum ScanErrorKind {
    /// A character that can't start a token, or a letter or `_` right after a number.
    UnexpectedChar(char),
    /// An operator is incomplete, e.g. `!` without `=`.
    MissingOneOf(&'static [char]),
    /// The line or file ended before the closing `"`.
    UnterminatedString,
    /// Only `\n`, `\t`, `\"`, and `\\` are supported.
    InvalidEscapeSequence(char),
    /// e.g. `1.`, `1.2.3`, or an integer too large for `i64`.
    InvalidNumber,
}

impl Display for ScanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            ScanErrorKind::UnexpectedChar(c) => format!("Unexpected character '{}'", c),
            ScanErrorKind::MissingOneOf(chars) => {
                let opts: Vec<String> = chars.iter().map(|c| format!("'{}'", c)).collect();
                format!("Missing {}", opts.join(" or "))
            }
            ScanErrorKind::UnterminatedString => "Unterminated string".to_string(),
            ScanErrorKind::InvalidEscapeSequence(c) => format!("Invalid escape sequence '\\{}'", c),
            ScanErrorKind::InvalidNumber => "Invalid number".to_string(),
        };
        write!(f, "Error at line {}: {}", self.line, message)
    }
}

/// Scans source into tokens that borrow their lexemes from it.
pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
    errors: Vec<ScanError>,
    /// Byte offset where the current lexeme starts.
    start: usize,
    /// Byte offset of the next unread character.
    current: usize,
    line: u32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner<'a> {
        Scanner {
            source,
            tokens: Vec::new(),
            errors: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    /// Scans the whole source into tokens ending with `Eof`.
    ///
    /// Returns every lexical error found, not just the first.
    pub fn scan_tokens(&mut self) -> Result<&Vec<Token<'a>>, &Vec<ScanError>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(TokenType::Eof, "", self.line));

        if self.errors.is_empty() {
            Ok(&self.tokens)
        } else {
            Err(&self.errors)
        }
    }

    fn scan_token(&mut self) {
        let c: char = self.advance();

        if c.is_ascii_digit() {
            self.scan_numeric();
            return;
        }

        match c {
            '(' => self.add_token(TokenType::LParen),
            ')' => self.add_token(TokenType::RParen),
            '{' => self.add_token(TokenType::LBrace),
            '}' => self.add_token(TokenType::RBrace),
            '[' => self.add_token(TokenType::LBracket),
            ']' => self.add_token(TokenType::RBracket),
            ',' => self.add_token(TokenType::Comma),
            ':' => self.add_token(TokenType::Colon),
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
            '%' => self.add_token(TokenType::Modulo),
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
                // Line comment.
                while self.peek() != '\n' && !self.is_at_end() {
                    self.advance();
                }
            }
            '"' => {
                self.scan_string();
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                self.scan_identifier();
            }

            ' ' | '\t' | '\r' => {}
            '\n' => self.line += 1,
            _ => self.error(ScanErrorKind::UnexpectedChar(c)),
        }
    }

    /// Scans an identifier or keyword.
    fn scan_identifier(&mut self) {
        while !self.is_at_end() {
            let c = self.peek();

            // Keep going as long as it's a letter, number, or _
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        // Get the lexeme.
        let text = &self.source[self.start..self.current];

        // Get the token type.
        let token_type = keyword_type(text).unwrap_or(TokenType::Identifier);

        self.add_token(token_type);
    }

    /// Scans a string literal. Strings can't span lines.
    fn scan_string(&mut self) {
        let mut s = String::new();

        while !self.is_at_end() && self.peek() != '\n' && self.peek() != '"' {
            let c = self.advance();

            // handle non-escape sequences
            if c != '\\' {
                s.push(c);
                continue;
            }

            // A trailing `\` leaves the string unclosed.
            if self.is_at_end() || self.peek() == '\n' {
                self.error(ScanErrorKind::UnterminatedString);
                return;
            }

            // handle escape sequences
            match self.advance() {
                'n' => s.push('\n'),
                't' => s.push('\t'),
                '"' => s.push('"'),
                '\\' => s.push('\\'),
                c => self.error(ScanErrorKind::InvalidEscapeSequence(c)),
            }
        }

        // check for unterminated string
        if self.is_at_end() || self.peek() == '\n' {
            self.error(ScanErrorKind::UnterminatedString);
            return;
        }

        // consume quote terminator
        self.advance();

        self.add_token(TokenType::String(s));
    }

    /// Scans an integer or float. Floats need digits on both sides of the `.`.
    fn scan_numeric(&mut self) {
        let mut is_float: bool = false;

        while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '.') {
            let c: char = self.advance();

            // check for float type
            if c == '.' {
                is_float = true;
            }
        }

        if self.peek() == '_' || self.peek().is_ascii_alphabetic() {
            self.error(ScanErrorKind::UnexpectedChar(self.peek()));
            return;
        }

        let text = &self.source[self.start..self.current];

        if text.starts_with('.') || text.ends_with('.') {
            self.error(ScanErrorKind::InvalidNumber);
            return;
        }

        if is_float {
            match text.parse::<f64>() {
                Ok(n) => self.add_token(TokenType::Float(n)),
                Err(_) => self.error(ScanErrorKind::InvalidNumber),
            }
        } else {
            match text.parse::<i64>() {
                Ok(n) => self.add_token(TokenType::Integer(n)),
                Err(_) => self.error(ScanErrorKind::InvalidNumber),
            }
        }
    }

    /// Consumes and returns the current character.
    fn advance(&mut self) -> char {
        let c = self.peek();
        self.current += c.len_utf8();
        c
    }

    /// Returns the current character without consuming it, or `'\0'` at the end.
    fn peek(&self) -> char {
        self.source[self.current..].chars().next().unwrap_or('\0')
    }

    /// Consumes the current character only if it's `expected`.
    fn match_expected(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            self.current += expected.len_utf8();
            true
        }
    }

    /// Records a finished token spanning `start..current`.
    fn add_token(&mut self, token_type: TokenType) {
        // Slice the lexeme directly out of the source.
        let lexeme = &self.source[self.start..self.current];
        // Add the token.
        self.tokens.push(Token::new(token_type, lexeme, self.line));
    }

    /// True once `current` has passed the end of `source`.
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
