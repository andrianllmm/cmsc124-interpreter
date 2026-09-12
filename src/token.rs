use std::fmt::{Display, Error, Formatter};

pub enum TokenType {
    Assign,
    LParen,
    RParen,
    Multiply,
    Divide,
    Add,
    Subtract,
    Equals,
    More,
    MoreEquals,
    Less,
    LessEquals,
    Eof,
}

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: String,
    line: i32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, literal: &str, line: i32) -> Token {
        Token {
            token_type,
            lexeme: String::from(lexeme),
            literal: String::from(literal),
            line,
        }
    }
}

// string conversion of TokenType Enum
impl TokenType {
    fn as_str(&self) -> &'static str {
        match self {
            TokenType::Assign => "ASSIGN",
            TokenType::LParen => "LPAREN",
            TokenType::RParen => "RPAREN",
            TokenType::Multiply => "MULTIPLY",
            TokenType::Divide => "DIVIDE",
            TokenType::Add => "ADD",
            TokenType::Subtract => "SUBTRACT",
            TokenType::Equals => "EQUALS",
            TokenType::More => "MORE",
            TokenType::MoreEquals => "MORE_EQUALS",
            TokenType::Less => "LESS",
            TokenType::LessEquals => "LESS_EQUALS",
            TokenType::Eof => "EOF",
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "Token(type={}, lexeme={}, literal={}, line={})",
            self.token_type.as_str(),
            self.lexeme,
            self.literal,
            self.line
        )
    }
}
