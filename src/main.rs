#![allow(unused)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod lexer;

use std::fs;
use std::io::{BufReader, Read};

use crate::lexer::{Source, Lexer, Token};

fn file_source(path: &str) -> impl Read {
    let file = fs::File::open(path).unwrap();

    BufReader::new(file)
}

fn main() {
    // let source = "struct HelloWorld { int a, int b, string c, }".to_string();
    // let source = source.chars();
    // let mut lex = Lexer::new(source);

    let source = Source::file("./files/one.fr").expect("Failed to open files");
    println!("{}", source.get_contents());

    // loop {
    //     let tok = lex.next_token();
    //     println!("{:?}", tok);
    //     if let Token::Eof = tok {
    //         break;
    //     }
    // }
}
