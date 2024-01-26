use crate::token::{self, Token};
use std::path::{Path, PathBuf};
use std::{fs, io};

pub trait Source {
    fn next_char(&mut self) -> Option<char>;
    // fn get_chars<'a>(&'a self) -> impl Iterator<Item = char> + 'a;
}

pub struct FileSource<'a> {
    chars: std::str::Chars<'a>,
}

impl<'a> Source for FileSource<'a> {
    fn next_char(&mut self) -> Option<char> {
        self.chars.next()
    }
}

pub struct ForserFile {
    contents: String,
}

impl ForserFile {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let contents = fs::read_to_string(path)?;
        Ok(Self { contents })
    }

    pub fn source(&self) -> impl Source + '_ {
        FileSource { chars: self.contents.chars() }
    }
}

/* ============================= */

// fn lex_source<'a, S: Source + 'a>(source: &'a S) -> Lexer<'a, impl Iterator<Item = char> + 'a> {
//     let mut source = source.get_chars();
//     Lexer::new(&mut source)
// }

pub struct Lexer<'a, S> {
    source: &'a mut S,
    current: Option<char>,
    next: Option<char>,
    position: usize, // Position of next character
}

impl<'a, S> Lexer<'a, S>
where
    S: Source + 'a,
{
    pub fn new(source: &'a mut S) -> Self {
        Self {
            current: None,
            next: source.next_char(),
            source,
            position: 0,
        }
    }

    pub fn consume(&mut self) -> Option<char> {
        self.current = self.next.clone();
        self.next = self.source.next_char();
        self.position += 1;
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

            if c == ' ' || c == '\n' {
                continue;
            }

            let token = match c {
                '{' => Token::BraceLeft,
                '}' => Token::BraceRight,
                ';' => Token::Semicolon,
                'a'..='z' | 'A'..='Z' => {
                    let ident: String = self.consume_identifier();
                    token::to_keyword(&ident).unwrap_or_else(|| Token::Identifier(ident))
                }
                x => panic!("Invalid character: {}", x),
            };

            return token;
        }
    }

    fn consume_identifier(&mut self) -> String {
        let mut ident = String::new();
        ident.push(self.current.unwrap() as _);
        while let Some(c @ ('a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-')) = self.next {
            ident.push(c as _);
            self.consume();
        }

        ident
    }
}

pub trait LexedTokenStream {
    fn next_token(&mut self) -> Token;
}

impl<'a, S> LexedTokenStream for Lexer<'a, S>
where
    S: Source + 'a,
{
    fn next_token(&mut self) -> Token {
        self.next_token()
    }
}
