#[derive(Debug)]
pub enum Token {
    Struct,
    BraceLeft,
    BraceRight,
    Semicolon,
    Identifier(String),
    Eof,
}

pub fn to_keyword(ident: &str) -> Option<Token> {
    match ident {
        "struct" => Some(Token::Struct),
        _ => None,
    }
}
