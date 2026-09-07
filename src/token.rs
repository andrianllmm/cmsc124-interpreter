enum TokenType {
    EQUALS,
    L_PAREN,
    R_PAREN,
    MORE,
    LESS,
    MULTIPLY,
    DIVIDE,
    ADD,
    SUBTRACT,
}

struct Token {
    token_type: String,
    lexeme: String,
    literal: String,
    line: i32,
}

impl Token {

    fn new(token_type: &str, lexeme: &str, literal: &str, line: i32) {
        return Token {
            token_type,
            lexeme,
            literal,
            line
        };
    }

    // getters
    fn get_type(&self) -> String{
        return self.token_type;
    }

    fn get_lexeme(&self) -> String{
        return self.lexeme;
    }

    fn get_literal(&self) -> String{
        return self.literal;
    }

    fn get_line(&self) -> i32 {
        return self.line;
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!("Token(type={}, lexeme={}, literal={}, line={})",
            self.token_type, self.lexeme, self.literal, self.line )
    }
}