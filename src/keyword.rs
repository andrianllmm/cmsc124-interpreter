use crate::token::TokenType;

pub fn keyword_type(text: &str) -> Option<TokenType> {
    match text {
        "return" => Some(TokenType::Return),
        "if" => Some(TokenType::If),
        "else" => Some(TokenType::Else),
        "match" => Some(TokenType::Match),
        "case" => Some(TokenType::Case),
        "for" => Some(TokenType::For),
        "in" => Some(TokenType::In),
        "while" => Some(TokenType::While),
        "and" => Some(TokenType::And),
        "or" => Some(TokenType::Or),
        "not" => Some(TokenType::Not),
        "true" => Some(TokenType::True),
        "false" => Some(TokenType::False),
        "null" => Some(TokenType::Null),
        _ => None,
    }
}
