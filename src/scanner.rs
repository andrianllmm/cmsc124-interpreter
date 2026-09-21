use crate::keyword::keyword_type;
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
    UnterminatedString,
    InvalidEscapeSequence(char),
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
            self.scan_numeric(c);
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
                while !self.match_expected('\n') && !self.is_at_end() {
                    self.advance();
                }

                if !self.is_at_end() {
                    self.line += 1;
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

    // Scans an identifier or keyword.
    fn scan_identifier(&mut self) {
        while !self.is_at_end() {
            let c = self.peek();

            // Keep going as long as it's a letter, number, or _
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        // Get the lexeme.
        let text: String = self.source[self.start..self.current].iter().collect();

        // Get the token type.
        let token_type = keyword_type(&text).unwrap_or(TokenType::Identifier);

        self.add_token(token_type);
    }

    // Scans a string literal.
    fn scan_string(&mut self) {
        let mut s = String::new();

        while !self.is_at_end() && self.peek() != '\n' && self.peek() != '"' {
            let c = self.advance();

            // handle non-escape sequences
            if c != '\\' {
                s.push(c);
                continue;
            }

            // check for unterminated string
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

    fn scan_numeric(&mut self, start: char) {
        let mut s: String = start.to_string();
        let mut is_float: bool = false;

        while !self.is_at_end()
            && self.peek() != '\n'
            && !{ [' ', '\t', '\r'].contains(&self.peek()) }
        {
            let c: char = self.advance();

            // check if number is valid digit or dot
            if !c.is_ascii_digit() && (c != '.' || is_float) {
                self.error(ScanErrorKind::InvalidNumber);
                return;
            }

            // check for float type
            if c == '.' {
                is_float = true;
            }

            // add number to literal
            s.push(c);
        }

        if s.starts_with('.') || s.ends_with('.') {
            self.error(ScanErrorKind::InvalidNumber);
            return;
        }

        if is_float {
            match s.parse::<f64>() {
                Ok(n) => self.add_token(TokenType::Float(n)),
                Err(_) => self.error(ScanErrorKind::InvalidNumber),
            }
        } else {
            match s.parse::<i64>() {
                Ok(n) => self.add_token(TokenType::Integer(n)),
                Err(_) => self.error(ScanErrorKind::InvalidNumber),
            }
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
    fn add_token(&mut self, token_type: TokenType) {
        // Get the lexeme.
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        // Add the token.
        self.tokens.push(Token::new(token_type, &lexeme, self.line));
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
