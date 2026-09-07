use std::fmt::{Display, Formatter, Error};

enum TokenType {
    EQUALS,
    LPAREN,
    RPAREN,
    MORE,
    LESS,
    MULTIPLY,
    DIVIDE,
    ADD,
    SUBTRACT,
    NULL
}

struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: String,
    line: i32,
}

impl Token {
    fn new(token_type: TokenType, lexeme: &str, literal: &str, line: i32) -> Token {
        return Token {
            token_type,
            lexeme: String::from(lexeme),
            literal: String::from(literal),
            line
        };
    }

    // getters
    fn get_type(&self) -> &TokenType {
        return &self.token_type;
    }

    fn get_lexeme(&self) -> &String{
        return &self.lexeme;
    }

    fn get_literal(&self) -> &String{
        return &self.literal;
    }

    fn get_line(&self) -> i32 {
        return self.line;
    }
}

// string conversion of TokenType Enum
impl TokenType {
    fn as_str(&self) -> &'static str {
        return match self {
            TokenType::EQUALS => "EQUALS",
            TokenType::LPAREN => "LPAREN",
            TokenType::RPAREN => "RPAREN",
            TokenType::MORE => "MORE",
            TokenType::LESS => "LESS",
            TokenType::MULTIPLY => "MULTIPLY",
            TokenType::DIVIDE => "DIVIDE",
            TokenType::ADD => "ADD",
            TokenType::SUBTRACT => "SUBTRACT",
            TokenType::NULL => "NULL",
        };
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Token(type={}, lexeme={}, literal={}, line={})",
            self.token_type.as_str(), self.lexeme, self.literal, self.line )
    }
}
