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
    If,
    Else,
    True,
    False,
    Null,
    And,
    Or,
    Not,
    Table,      // begins a table literal
    Load,       // loads a table from a file
    Save,       // saves a table to a file
    Show,       // displays a value
    Where,      // filters table rows using a condition
    Select,     // selects table columns by name
    At,         // performs positional table selection
    Rows,       // specifies rows for positional selection
    Columns,    // specifies columns for positional selection
    Add,        // adds a column
    Take,       // limits the number of rows returned
    Sort,       // sorts a table
    Descending, // sorts values from greatest to least
    By,         // specifies the column used for sorting or grouping
    Group,      // begins a grouping operation
    Calculate,  // defines an aggregation
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
            TokenType::If => "IF",
            TokenType::Else => "ELSE",
            TokenType::True => "TRUE",
            TokenType::False => "FALSE:",
            TokenType::Null => "NULL",
            TokenType::And => "AND",
            TokenType::Or => "OR",
            TokenType::Not => "NOT",
            TokenType::Table => "TABLE",
            TokenType::Load => "LOAD",
            TokenType::Save => "SAVE",
            TokenType::Show => "SHOW",
            TokenType::Where => "WHERE",
            TokenType::Select => "SELECT",
            TokenType::At => "AT",
            TokenType::Rows => "ROWS",
            TokenType::Columns => "COLUMNS",
            TokenType::Add => "ADD",
            TokenType::Take => "TAKE",
            TokenType::Sort => "SORT",
            TokenType::Descending => "DESCENDING",
            TokenType::By => "BY",
            TokenType::Group => "GROUP",
            TokenType::Calculate => "CALCULATE",
            TokenType::String => "STRING",
            TokenType::Comment => "COMMENT",
            TokenType::Eof => "EOF",
        };
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Token(type={}, lexeme={}, literal={}, line={})",
            self.token_type.as_str(), self.lexeme, self.literal, self.line )
    }
}
