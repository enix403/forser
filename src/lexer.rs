use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::token::{self, Token};

pub trait Source {
    fn get_contents(&self) -> &'_ str;
}

pub struct FileSource {
    path: PathBuf,
    contents: String,
}

impl FileSource {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        Ok(Self {
            contents: fs::read_to_string(&path)?,
            path,
        })
    }
}

impl Source for FileSource {
    fn get_contents(&self) -> &'_ str {
        self.contents.as_str()
    }
}

pub struct Lexer<'a, R: 'a> {
    reader: R,
    current: Option<char>,
    next: Option<char>,
    position: usize, // Position of next character
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Lexer<'a, ()> {
    pub fn new<S: Source>(source: &'a S) -> Lexer<'a, std::str::Chars<'a>> {
        let chars = source.get_contents().chars();
        Lexer::new_from_reader(chars)
    }
}

impl<'a, R: 'a> Lexer<'a, R>
where
    R: Iterator<Item = char>,
{
    pub fn new_from_reader(mut reader: R) -> Self {
        Self {
            current: None,
            next: reader.next(),
            reader,
            position: 0,
            _phantom: PhantomData,
        }
    }

    pub fn consume(&mut self) -> Option<char> {
        self.current = self.next.clone();
        self.next = self.reader.next();
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
