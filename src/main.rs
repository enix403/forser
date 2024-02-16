#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use clap::Parser;
use std::path::PathBuf;

pub mod codegen;
pub mod items;
pub mod lexer;
pub mod parser;
pub mod token;

use lexer::ForserFile;
use lexer::Lexer;

use codegen::Language;

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
    let program = parser.parse();

    {
        let mut gen = Box::new(codegen::TypeScriptGenerator::new());
        let gen_outdir = args
            .out_dir
            .unwrap_or_else(|| "build".into())
            .join(gen.lang_id());

        std::fs::create_dir_all(&gen_outdir).expect("Failed to create output directory");

        gen.generate(&program, &gen_outdir);
    }
}
