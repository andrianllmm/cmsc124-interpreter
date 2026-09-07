use std::fmt::{Display, Formatter, Error};

pub enum TokenType {
    Equals,
    LParen,
    RParen,
    More,
    Less,
    Multiply,
    Divide,
    Add,
    Subtract,
    Null,
}

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: String,
    line: i32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, literal: &str, line: i32) -> Token {
        return Token {
            token_type,
            lexeme: String::from(lexeme),
            literal: String::from(literal),
            line
        };
    }

    // getters
    pub fn get_type(&self) -> &TokenType {
        return &self.token_type;
    }

    pub fn get_lexeme(&self) -> &String{
        return &self.lexeme;
    }

    pub fn get_literal(&self) -> &String{
        return &self.literal;
    }

    pub fn get_line(&self) -> i32 {
        return self.line;
    }
}

// string conversion of TokenType Enum
impl TokenType {
    fn as_str(&self) -> &'static str {
        return match self {
            TokenType::Equals => "EQUALS",
            TokenType::LParen => "LPAREN",
            TokenType::RParen => "RPAREN",
            TokenType::More => "MORE",
            TokenType::Less => "LESS",
            TokenType::Multiply => "MULTIPLY",
            TokenType::Divide => "DIVIDE",
            TokenType::Add => "ADD",
            TokenType::Subtract => "SUBTRACT",
            TokenType::Null => "NULL",
        };
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Token(type={}, lexeme={}, literal={}, line={})",
            self.token_type.as_str(), self.lexeme, self.literal, self.line )
    }
}
