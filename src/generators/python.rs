use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::Path;

use crate::items::Program;
use crate::language::Language;

use crate::glang::render_template;

pub struct PythonGenerator {
    _phantom: (),
}

impl PythonGenerator {
    pub fn new() -> Self {
        Self { _phantom: () }
    }
}

impl Language for PythonGenerator {
    fn lang_id(&self) -> &'static str {
        "py"
    }

    fn extension(&self) -> &'static str {
        "py"
    }

    fn generate(&self, program: &Program, outfile: &Path) {
        let dest = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(outfile)
            .expect("Failed to open file");

        render_template(include_str!("python.gx"), program, dest).unwrap();
    }
}