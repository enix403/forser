#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser as ClapParser;
use lazy_static::lazy_static;

use forser::items::Program;
use forser::language::Language;
use forser::lexer::ForserFile;
use forser::lexer::Lexer;
use forser::parser::{ParseError, Parser};

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)]
struct Args {
    /// Input file(s)
    #[clap(
        num_args = 1..,
        required = true
    )]
    in_files: Vec<PathBuf>,

    /// Comma separated list of target languages
    #[clap(
        short, long,
        required = true,
        value_parser = clap::builder::NonEmptyStringValueParser::new(),
        require_equals = true,
        num_args = 1..,
        value_delimiter = ','
    )]
    langs: Vec<String>,

    /// Directory where the generated files will be stored
    #[arg(short = 'd', long, default_value = ".")]
    out_dir: String,

    /// Put generated file(s) of each language in its own subdirectory under `out_dir`
    #[arg(short = 'a', long, default_value = "false")]
    lang_dir: bool,

    /// Name of generated file(s).
    ///
    /// `[name]` is replaced by the filename of corresponding input file
    ///
    /// `[ext]` is replaced by the standard extension of the generated language
    #[arg(short = 'f', long, default_value = "[name].[ext]")]
    out_filename: String,
}

lazy_static! {
    static ref GENERATORS: HashMap<&'static str, Box<dyn Language>> = {
        use forser::generators::*;
        let mut m: HashMap<&'static str, Box<dyn Language>> = HashMap::new();
        m.insert("ts", Box::new(TypeScriptGenerator::new()));
        m.insert("py", Box::new(PythonGenerator::new()));
        m
    };
}

// TODO: optimise
fn write_programs(args: &Args, tasks: Vec<(&PathBuf, Program)>) {
    let generators = args
        .langs
        .iter()
        .map(|lang| {
            let generator = GENERATORS.get(lang.as_str());
            generator.map(|bx| bx.as_ref()).ok_or(lang)
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|unknown_lang| {
            panic!("Unknown language \"{}\"", unknown_lang);
        });

    let out_dir = PathBuf::from(&args.out_dir);

    for gen in generators {
        // Append language id to final output path if lang_dir is true
        let mut out = if args.lang_dir {
            out_dir.join(gen.lang_id())
        } else {
            out_dir.clone()
        };

        std::fs::create_dir_all(&out).expect("Failed to create output directory");

        for (in_file, program) in tasks.iter() {
            let in_file_name = in_file.file_stem().and_then(|p| p.to_str()).unwrap();

            let filename = args
                .out_filename
                .replace("[name]", in_file_name)
                .replace("[ext]", gen.extension());

            let out = out.join(filename);

            gen.generate(&program, &out);
        }
    }
}

fn main() -> ExitCode {
    let args = Args::parse();

    let tasks = args
        .in_files
        .iter()
        .map(|in_file| {
            let file = ForserFile::new(&in_file).expect("Failed to open file");
            let mut source = file.source();
            let mut lex = Lexer::new(&mut source);

            let mut parser = Parser::new(lex);
            let task = parser
                .parse()
                .map(|program| (in_file, program))
                .map_err(|errors| unsafe { errors.get_unchecked(0).clone() });

            task
        })
        .collect::<Result<Vec<_>, _>>();

    match tasks {
        Ok(tasks) => {
            write_programs(&args, tasks);
            ExitCode::SUCCESS
        },
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        }
    }
}
