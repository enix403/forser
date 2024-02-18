#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Keywords
    Struct,
    
    // Punctuation
    BraceLeft,
    BraceRight,
    SquareLeft,
    SquareRight,
    Comma,
    Colon,

    // Identifiers 
    Identifier(String),

    // Other
    Init,
    Eof,
}

pub fn to_keyword(ident: &str) -> Option<Token> {
    match ident {
        "struct" => Some(Token::Struct),
        _ => None,
    }
}
