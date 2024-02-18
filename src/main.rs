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
    let program = parser.parse().unwrap_or_else(|errors| {
        // Right now we show a panic message on the first error.
        // TODO: Perform proper error reporting

        // Safety: There must be atleast one error for parsing to fail
        let error = unsafe { errors.get_unchecked(0) };

        match error {
            parser::ParseError::UnexpectedToken { expected, found } => {
                if let Some(expected) = expected {
                    panic!("Expected token {:?}, found {:?}", expected, found);
                }
                else {
                    panic!("Unexpected token {:?}", found);
                };
            }
        }
    });

    {
        let mut gen: Box<dyn Language> = Box::new(codegen::TypeScriptGenerator::new());
        let gen_outdir = args
            .out_dir
            .unwrap_or_else(|| "build".into())
            .join(gen.lang_id());

        std::fs::create_dir_all(&gen_outdir).expect("Failed to create output directory");

        gen.generate(&program, &gen_outdir);
    }
}
