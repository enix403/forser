use std::path::Path;
use std::convert::AsRef;
use std::io;
use std::fs;

pub struct Source {
    contents: String
}

impl Source {
    pub fn file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Ok(Self {
            contents: fs::read_to_string(path)?
        })
    
    }
    pub fn get_contents(&self) -> &'_ str {
        &self.contents.as_ref()
    }
}

pub struct Lexer<R> {
    reader: R,
    current: Option<char>,
    next: Option<char>,
}

#[derive(Debug)]
pub enum Token {
    Struct,
    BraceLeft,
    BraceRight,
    Semicolon,
    Identifier(String),
    Eof,
}

fn to_keyword(ident: &str) -> Option<Token> {
    match ident {
        "struct" => Some(Token::Struct),
        _ => None,
    }
}

impl<R> Lexer<R>
where
    R: Iterator<Item = char>,
{
    pub fn new(mut reader: R) -> Self {
        Self {
            current: None,
            next: reader.next(),
            reader,
        }
    }

    pub fn consume(&mut self) -> Option<char> {
        self.current = self.next.clone();
        self.next = self.reader.next();
        self.current.clone()
    }

    pub fn peek(&self) -> Option<char> {
        self.next.clone()
    }
    
    pub fn next_token(&mut self) -> Token {
        loop {
            let c = match self.consume() {
                Some(x) => x,
                None => return Token::Eof,
            };

            if c == ' ' {
                continue;
            }

            let token = match c {
                '{' => Token::BraceLeft,
                '}' => Token::BraceRight,
                ';' => Token::Semicolon,
                'a'..='z' | 'A'..='Z' => {
                    let ident: String = self.consume_identifier();
                    to_keyword(&ident).unwrap_or_else(|| Token::Identifier(ident))
                }
                _ => panic!("Invalid character"),
            };

            return token;
        }
    }

    fn consume_identifier(&mut self) -> String {
        let mut ident = String::new();
        ident.push(self.current.unwrap() as _);
        while let Some(c @ ('a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-')) = self.next
        {
            ident.push(c as _);
            self.consume();
        }

        ident
    }
}