use std::fmt::{Display, Error, Formatter};

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
    /*
        NOTE: separate implementation of int and float
        specific to the language
    */
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
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    line: u32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, line: u32) -> Token {
        Token {
            token_type,
            lexeme: String::from(lexeme),
            line,
        }
    }

    #[allow(dead_code)]
    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    #[allow(dead_code)]
    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }

    #[allow(dead_code)]
    pub fn line(&self) -> u32 {
        self.line
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
            TokenType::Eof => "EOF",
        }
    }

    fn literal_string(&self) -> String {
        match self {
            TokenType::String(s) => s.clone(),
            TokenType::Integer(n) => n.to_string(),
            TokenType::Float(n) => n.to_string(),
            _ => String::new(),
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
            self.token_type.literal_string(),
            self.line
        )
    }
}
