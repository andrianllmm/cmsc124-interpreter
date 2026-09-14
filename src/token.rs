use std::fmt::{Display, Error, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Assign,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Terminator,
    Multiply,
    Divide,
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
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: String,
    line: u32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, literal: &str, line: u32) -> Token {
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
            TokenType::LBrace => "LBRACE",
            TokenType::RBrace => "RBRACE",
            TokenType::Terminator => "TERMINATOR",
            TokenType::Multiply => "MULTIPLY",
            TokenType::Divide => "DIVIDE",
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
