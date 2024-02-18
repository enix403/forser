#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
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

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub position: i64,
    pub column: i64,
    pub line: i64,
}

pub fn to_keyword(ident: &str) -> Option<TokenKind> {
    match ident {
        "struct" => Some(TokenKind::Struct),
        _ => None,
    }
}
