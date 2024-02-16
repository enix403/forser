use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use std::write;

use crate::items::{DataType, PrimitiveType, Program};

pub trait Language {
    fn lang_id(&self) -> &'static str;
    fn generate(&mut self, program: &Program, outdir: &Path);
}

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
    fn generate(&mut self, program: &Program, outdir: &Path) {
        let dest = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(outdir.join("main.ts"))
            .expect("Failed to open file");

        TypeScriptGeneratorInner::new(dest).generate(program);
    }
}

struct TypeScriptGeneratorInner<W> {
    dest: W,
}

impl<W: Write> TypeScriptGeneratorInner<W> {
    pub fn new(dest: W) -> Self {
        Self { dest }
    }

    fn type_str<'a>(dt: &'a DataType) -> &'a str {
        match dt {
            DataType::Primitive(prim) => match prim {
                PrimitiveType::String => "string",
                PrimitiveType::Int => "number",
            },
            DataType::UserDefined(ref name) => name.as_str(),
        }
    }

    fn generate(&mut self, program: &Program) -> io::Result<()> {
        for struct_ in program.structs.iter() {
            writeln!(&mut self.dest, "type {} = {{", struct_.name.as_str())?;

            for (index, field) in struct_.fields.iter().enumerate() {
                if index != 0 {
                    write!(&mut self.dest, ",\n")?;
                }

                let ty = Self::type_str(&field.datatype);

                write!(&mut self.dest, "    {}: {}", field.name, ty)?;
            }

            writeln!(&mut self.dest, "\n}};\n")?;
        }

        Ok(())
    }
}
