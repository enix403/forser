#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Struct,
    Enum,

    // Punctuation
    ParenLeft,
    ParenRight,
    BraceLeft,
    BraceRight,
    SquareLeft,
    SquareRight,
    AngleLeft,
    AngleRight,
    Comma,
    Colon,
    QuestionMark,
    Equal,

    // Identifiers
    Identifier(String),

    // Literals
    StringLiteral(String),
    IntLiteral(i32),

    // Unknowm
    Unknowm(char),

    // Other
    Init,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub pos_start: i64,
    pub pos_end: i64,
    pub line: i64,
    pub column: i64,
}

impl Token {
    pub fn init() -> Self {
        Self {
            kind: TokenKind::Init,
            pos_start: 0,
            pos_end: 0,
            column: 0,
            line: 1,
        }
    }
}

pub fn to_keyword(ident: &str) -> Option<TokenKind> {
    match ident {
        "struct" => Some(TokenKind::Struct),
        "enum" => Some(TokenKind::Enum),
        _ => None,
    }
}
