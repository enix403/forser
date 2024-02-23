#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser as ClapParser;
use lazy_static::lazy_static;

pub mod generators;
pub mod items;
pub mod language;
pub mod lexer;
pub mod parser;
pub mod token;

use language::Language;
use lexer::ForserFile;
use lexer::Lexer;
use parser::{ParseError, Parser};


#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)]
struct Args {
    /// Path to the input file
    in_file: PathBuf,

    /// Directory where the build files will be stored
    #[arg(long, short)]
    out_dir: Option<PathBuf>,

    /// Languages to generate the build for
    #[clap(
        short, long,
        required = true,
        value_parser = clap::builder::NonEmptyStringValueParser::new(),
        require_equals = true,
        num_args = 1..,
        value_delimiter = ','
    )]
    langs: Vec<String>,
}

lazy_static! {
    static ref GENERATORS: HashMap<&'static str, Box<dyn Language>> = {
        use crate::generators::*;
        let mut m: HashMap<&'static str, Box<dyn Language>> = HashMap::new();
        m.insert("ts", Box::new(TypeScriptGenerator::new()));
        m.insert("py", Box::new(PythonGenerator::new()));
        m
    };
}

fn main() -> ExitCode {
    let args = Args::parse();

    let file = ForserFile::new(args.in_file).expect("Failed to open files");
    let mut source = file.source();
    let mut lex = Lexer::new(&mut source);

    let mut parser = Parser::new(lex);
    let program = parser
        .parse()
        .map_err(|errors| unsafe { errors.get_unchecked(0).clone() });

    match program {
        Ok(program) => {
            let generators = args
                .langs
                .iter()
                .map(|lang| {
                    // GENERATORS.get(lang.as_str()).map(|bx| bx.as_ref())
                    let generator = GENERATORS.get(lang.as_str());
                    generator.map(|bx| bx.as_ref()).ok_or(lang)
                })
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|unknown_lang| {
                    panic!("Unknown language \"{}\"", unknown_lang);
                });

            let build_dir = args.out_dir.unwrap_or_else(|| "build".into());

            for gen in generators {
                let gen_outdir = build_dir.join(gen.lang_id());
                std::fs::create_dir_all(&gen_outdir).expect("Failed to create output directory");

                gen.generate(&program, &gen_outdir);
            }

            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        }
    }
}
