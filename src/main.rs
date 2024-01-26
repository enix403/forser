#![allow(unused)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod lexer;
pub mod token;

use std::fs;
use std::io::{BufReader, Read};

use crate::lexer::{FileSource, Lexer};
use crate::token::Token;

fn file_source(path: &str) -> impl Read {
    let file = fs::File::open(path).unwrap();

    BufReader::new(file)
}

fn main() {
    let source = FileSource::new("./files/one.fr").expect("Failed to open files");
    let mut lex = Lexer::new(&source);

    loop {
        let tok = lex.next_token();
        println!("{:?}", tok);
        if let Token::Eof = tok {
            break;
        }
    }
}
