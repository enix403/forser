#![allow(unused)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::fs;
use std::io::{BufReader, Read};

fn file_source(path: &str) -> impl Read {
    let file = fs::File::open(path).unwrap();

    BufReader::new(file)
}

struct Lexer<R> {
    reader: R,
    current: Option<char>,
    next: Option<char>,
}

#[derive(Debug)]
enum Token {
    Struct,
    BraceLeft,
    BraceRight,
    // ParenLeft,
    // ParenRight,
    // Colon,
    Comma,
    Identifier(String),
    // LiteralInteger(i32),
    // LiteralString(String),
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
                ',' => Token::Comma,
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

fn main() {
    let source = "struct HelloWorld { int a, int b, string c, }".to_string();
    let source = source.chars();
    let mut lex = Lexer::new(source);

    loop {
        let tok = lex.next_token();
        println!("{:?}", tok);
        if let Token::Eof = tok {
            break;
        }
    }

    // println!("{:?}", lex.next_token());
    // println!("{:?}", lex.next_token());
    // println!("{:?}", lex.next_token());
    // println!("{:?}", lex.next_token());
    // println!("{:?}", lex.next_token());
    // println!("{:?}", lex.next_token());
    // println!("{:?}", lex.next_token());
    // println!("{:?}", lex.next_token());
}
