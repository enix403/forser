use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use std::write;

use crate::items::{PrimitiveType, Program, StructDefinition, StructField, TyKind};

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

    fn render_type(dest: &mut String, ty: &TyKind) -> std::fmt::Result {
        match ty {
            TyKind::Primitive(prim) => match prim {
                PrimitiveType::String => write!(dest, "string")?,
                PrimitiveType::Int => write!(dest, "number")?,
            },
            TyKind::UserDefined(ref name) => write!(dest, "{}", name)?,
            TyKind::Array(ref ty) => {
                write!(dest, "Array<")?;
                Self::render_type(dest, ty)?;
                write!(dest, ">")?
            }
            TyKind::Nullable(ref ty) => {
                write!(dest, "(")?;
                Self::render_type(dest, ty)?;
                write!(dest, ") | null")?
            }
        }

        Ok(())
    }

    fn render_field(dest: &mut String, field: &StructField) -> std::fmt::Result {
        write!(dest, "public {}!: ", field.name)?;
        Self::render_type(dest, &field.datatype)?;
        write!(dest, ";\n")?;

        Ok(())
    }

    fn write_struct(&mut self, struct_: &StructDefinition) -> io::Result<()> {
        let mut field_lines = String::new();

        for f in struct_.fields.iter() {
            Self::render_field(&mut field_lines, f);
        }

        // println!("{}", field_lines);

        Ok(())
    }

    fn generate(&mut self, program: &Program) -> io::Result<()> {
        write!(&mut self.dest, "{}", TS_HEADER)?;

        for struct_ in program.structs.iter() {
            self.write_struct(struct_);
        }

        Ok(())
    }

    // fn generate(&mut self, program: &Program) -> io::Result<()> {
    //     const INDENT: &'static str = "    ";

    //     for struct_ in program.structs.iter() {
    //         writeln!(&mut self.dest, "type {} = {{", struct_.name.as_str())?;

    //         for (index, field) in struct_.fields.iter().enumerate() {
    //             // The comma and new line
    //             if index != 0 {
    //                 write!(&mut self.dest, ",\n")?;
    //             }

    //             // indent
    //             write!(&mut self.dest, "{}", INDENT)?;
    //             // name: datatype
    //             write!(&mut self.dest, "{}: ", field.name)?;
    //             Self::write_type(&mut self.dest, &field.datatype)?;
    //         }

    //         writeln!(&mut self.dest, "\n}};\n")?;
    //     }

    //     Ok(())
    // }
}

const TS_HEADER: &'static str = r"#
const enum TyKindTag {
  Primitive,
  Message,
  Array,
}

type TyKind =
  | { kind: TyKindTag.Primitive }
  | { kind: TyKindTag.Message }
  | { kind: TyKindTag.Array; of: TyKind };

type StructField = {
  name: string;
  ty: TyKind;
};

function valueToPlainObject(value: any, ty: TyKind) {
  if (value === null) {
    return null;
  }
  //
  else if (ty.kind === TyKindTag.Primitive) {
    return value;
  }
  //
  else if (ty.kind === TyKindTag.Message) {
    return (value as Message).toPlainObject();
  }
  //
  else if (ty.kind === TyKindTag.Array) {
    return (value as any[]).map((val) => valueToPlainObject(val, ty.of));
  } else {
    throw new Error('Invalid value/ty');
  }
}

abstract class Message {
  abstract toPlainObject(): object;
}

abstract class StructMessage extends Message {
  protected abstract getFields(): StructField[];
  toPlainObject(): object {
    let result: Record<string, any> = {};

    let fields = this.getFields();
    for (let i = 0; i < fields.length; ++i) {
      const { name, ty } = fields[i];
      const value = this[name];

      result[name] = valueToPlainObject(value, ty);
    }

    return result;
  }
}

namespace forser {
  export function packMessage<M extends Message>(message: M) {
    return JSON.stringify((message as Message).toPlainObject());
  }
}

type FieldsOf<T> = Pick<
  T,
  {
    [K in keyof T]: T[K] extends Function ? never : K;
  }[keyof T]
>;
#";
