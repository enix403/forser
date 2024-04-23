use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::Path;

use crate::items::Program;
use crate::language::Language;

use crate::glang::render_template;

pub struct TypeScriptGenerator {
    _phantom: (),
}

impl TypeScriptGenerator {
    pub fn new() -> Self {
        Self { _phantom: () }
    }
}

impl Language for TypeScriptGenerator {
    fn lang_id(&self) -> &'static str {
        "ts"
    }

    fn generate(&self, program: &Program, outdir: &Path) {
        let dest = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(outdir.join("main.ts"))
            .expect("Failed to open file");

        println!("{:#?}", program.enums);

        render_template(include_str!("typescript.gx"), program, dest).unwrap();
    }
}
