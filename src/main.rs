#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use clap::Parser;
use std::path::PathBuf;

pub mod lexer;
pub mod items;
pub mod token;
pub mod parser;

use lexer::ForserFile;
use lexer::Lexer;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)]
struct Args {
    /// Path to the input file
    in_file: PathBuf,

    /// Directory where the build files will be stored
    #[arg(long, short)]
    out_dir: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let file = ForserFile::new(args.in_file).expect("Failed to open files");
    let mut source = file.source();
    let mut lex = Lexer::new(&mut source);

    let mut parser = parser::Parser::new(lex);
    parser.parse();

    println!("{:#?}", parser.structs);
}
