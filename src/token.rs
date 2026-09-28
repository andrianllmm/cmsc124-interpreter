//! Tokens produced by the scanner.

use std::fmt::{Display, Error, Formatter};

/// Kind of token. Literals carry their value.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Assign,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Terminator,
    Multiply,
    Divide,
    Modulo,
    Add,
    Subtract,
    Exponent,
    Equals,
    NotEquals,
    More,
    MoreEquals,
    Less,
    LessEquals,
    MulAssign,
    DivAssign,
    AddAssign,
    SubAssign,
    Pipe,
    PipeAssign,
    StringConcat,
    LambdaArrow,
    MatchArrow,
    String(String),
    Integer(i64),
    Float(f64),
    Identifier,
    Return,
    If,
    Else,
    Match,
    Case,
    For,
    In,
    While,
    And,
    Or,
    Not,
    True,
    False,
    Null,
    Table,
    Eof,
}

/// A lexeme borrowed from the source, with its type and line.
#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    token_type: TokenType,
    lexeme: &'a str,
    line: u32,
}

impl<'a> Token<'a> {
    pub fn new(token_type: TokenType, lexeme: &'a str, line: u32) -> Token<'a> {
        Token {
            token_type,
            lexeme,
            line,
        }
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn lexeme(&self) -> &str {
        self.lexeme
    }

    pub fn line(&self) -> u32 {
        self.line
    }
}

impl TokenType {
    /// Name shown in `--tokenize` output.
    fn as_str(&self) -> &'static str {
        match self {
            TokenType::Assign => "ASSIGN",
            TokenType::LParen => "LPAREN",
            TokenType::RParen => "RPAREN",
            TokenType::LBrace => "LBRACE",
            TokenType::RBrace => "RBRACE",
            TokenType::LBracket => "LBRACKET",
            TokenType::RBracket => "RBRACKET",
            TokenType::Comma => "COMMA",
            TokenType::Colon => "COLON",
            TokenType::Terminator => "TERMINATOR",
            TokenType::Multiply => "MULTIPLY",
            TokenType::Divide => "DIVIDE",
            TokenType::Modulo => "MODULO",
            TokenType::Add => "ADD",
            TokenType::Subtract => "SUBTRACT",
            TokenType::Exponent => "EXPONENT",
            TokenType::Equals => "EQUALS",
            TokenType::NotEquals => "NOT_EQUALS",
            TokenType::More => "MORE",
            TokenType::MoreEquals => "MORE_EQUALS",
            TokenType::Less => "LESS",
            TokenType::LessEquals => "LESS_EQUALS",
            TokenType::MulAssign => "MUL_ASSIGN",
            TokenType::DivAssign => "DIV_ASSIGN",
            TokenType::AddAssign => "ADD_ASSIGN",
            TokenType::SubAssign => "SUB_ASSIGN",
            TokenType::Pipe => "PIPE",
            TokenType::PipeAssign => "PIPE_ASSIGN",
            TokenType::StringConcat => "STRING_CONCAT",
            TokenType::LambdaArrow => "LAMBDA_ARROW",
            TokenType::MatchArrow => "MATCH_ARROW",
            TokenType::String(_) => "STRING",
            TokenType::Integer(_) => "INTEGER",
            TokenType::Float(_) => "FLOAT",
            TokenType::Identifier => "IDENTIFIER",
            TokenType::Return => "RETURN",
            TokenType::If => "IF",
            TokenType::Else => "ELSE",
            TokenType::Match => "MATCH",
            TokenType::Case => "CASE",
            TokenType::For => "FOR",
            TokenType::In => "IN",
            TokenType::While => "WHILE",
            TokenType::And => "AND",
            TokenType::Or => "OR",
            TokenType::Not => "NOT",
            TokenType::True => "TRUE",
            TokenType::False => "FALSE",
            TokenType::Null => "NULL",
            TokenType::Table => "TABLE",
            TokenType::Eof => "EOF",
        }
    }

    /// The literal's value as printed, or an empty string for non-literals.
    pub fn literal_string(&self) -> String {
        match self {
            TokenType::String(s) => s.escape_debug().to_string(),
            TokenType::Integer(n) => n.to_string(),
            // Debug keeps the `.0` on whole floats, so `1.0` doesn't print as `1`.
            TokenType::Float(n) => format!("{:?}", n),
            _ => String::new(),
        }
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "Token(type={}, lexeme={}, literal={}, line={})",
            self.token_type.as_str(),
            self.lexeme,
            self.token_type.literal_string(),
            self.line
        )
    }
}
