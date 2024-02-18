use crate::token::{self, Token};
use std::path::{Path, PathBuf};
use std::{fs, io};

pub trait Source {
    fn next_char(&mut self) -> Option<char>;
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
        FileSource {
            chars: self.contents.chars(),
        }
    }
}

/* ============================= */

pub struct Lexer<'a, S> {
    source: &'a mut S,
    current: Option<char>,
    next: Option<char>,
    position: i64, // Position of next character
    column: i64,
    line: i64,
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
            column: 0,
            line: 1,
        }
    }

    pub fn consume(&mut self) -> Option<char> {
        self.current = self.next.clone();
        self.next = self.source.next_char();

        if let Some(x) = self.current {
            self.position += 1;

            // TODO: Convert all \r\n sequences to \n

            if x == '\n' {
                self.column = 1;
                self.line += 1;
            } else {
                self.column += 1;
            }

            Some(x)
        } else {
            None
        }
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

            if c == '/' && self.next.is_some_and(|c| c == '/') {
                self.consume_line_comment();
                continue;
            }

            let token = match c {
                '{' => Token::BraceLeft,
                '}' => Token::BraceRight,
                '[' => Token::SquareLeft,
                ']' => Token::SquareRight,
                ',' => Token::Comma,
                ':' => Token::Colon,
                'a'..='z' | 'A'..='Z' => {
                    let ident: String = self.consume_identifier();
                    token::to_keyword(&ident).unwrap_or_else(|| Token::Identifier(ident))
                }
                x => panic!("Invalid character: {}", x),
            };

            return token;
        }
    }

    fn consume_line_comment(&mut self) {
        loop {
            match self.consume() {
                Some('\n') => break,
                _ => ()
            }
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

pub trait TokenStream {
    fn next_token(&mut self) -> Token;
}

impl<'a, S> TokenStream for Lexer<'a, S>
where
    S: Source + 'a,
{
    fn next_token(&mut self) -> Token {
        self.next_token()
    }
}
