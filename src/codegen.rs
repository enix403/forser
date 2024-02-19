use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use std::write;

use crate::items::{PrimitiveType, Program, TyKind};

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

    fn write_type(dest: &mut W, ty: &TyKind) -> io::Result<()> {
        match ty {
            TyKind::Primitive(prim) => match prim {
                PrimitiveType::String => write!(dest, "string")?,
                PrimitiveType::Int => write!(dest, "number")?,
            },
            TyKind::UserDefined(ref name) => write!(dest, "{}", name)?,
            TyKind::Array(ref ty) => {
                write!(dest, "Array<")?;
                Self::write_type(dest, ty)?;
                write!(dest, ">")?
            }
            TyKind::Nullable(ref ty) => {
                write!(dest, "(")?;
                Self::write_type(dest, ty)?;
                write!(dest, ") | null")?
            }
        }

        Ok(())
    }

    fn generate(&mut self, program: &Program) -> io::Result<()> {
        const INDENT: &'static str = "    ";

        for struct_ in program.structs.iter() {
            writeln!(&mut self.dest, "type {} = {{", struct_.name.as_str())?;

            for (index, field) in struct_.fields.iter().enumerate() {
                // The comma and new line
                if index != 0 {
                    write!(&mut self.dest, ",\n")?;
                }
                // indent
                write!(&mut self.dest, "{}", INDENT)?;
                // name: datatype
                write!(&mut self.dest, "{}: ", field.name)?;
                Self::write_type(&mut self.dest, &field.datatype)?;
            }

            writeln!(&mut self.dest, "\n}};\n")?;
        }

        Ok(())
    }
}
