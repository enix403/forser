use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use std::write;

use crate::items::{PrimitiveType, Program, StructDefinition, StructField, TyKind};
use serde::Serialize;

use tinytemplate::TinyTemplate;

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

#[derive(Serialize)]
struct RenderedField<'a> {
    name: &'a str,
    info: String,
    ty: String,
}

#[derive(Serialize, Clone)]
enum TsFieldType {
    Primitve,
    Array(Box<TsFieldType>),
    Message,
}

impl TsFieldType {
    fn write_ts_field_type(&self, dest: &mut String) {
        match self {
            TsFieldType::Primitve => {
                write!(dest, "{{ kind: TyKindTag.Primitive }}").unwrap();
            }
            TsFieldType::Message => {
                write!(dest, "{{ kind: TyKindTag.Message }}").unwrap();
            }
            TsFieldType::Array(ref inner) => {
                write!(dest, "{{ kind: TyKindTag.Array, of: ");
                inner.write_ts_field_type(&mut *dest);
                write!(dest, "}}");
            }
        };
    }
}

#[derive(Serialize)]
struct StructContext<'a> {
    name: &'a str,
    fields: Vec<RenderedField<'a>>,
}

impl From<&TyKind> for TsFieldType {
    fn from(value: &TyKind) -> Self {
        match value {
            TyKind::Primitive(..) => TsFieldType::Primitve,
            TyKind::UserDefined(..) => TsFieldType::Message,
            TyKind::Array(inner) => TsFieldType::Array(Box::new(inner.as_ref().into())),
            TyKind::Nullable(inner) => inner.as_ref().into(),
        }
    }
}

impl<W: Write> TypeScriptGeneratorInner<W> {
    pub fn new(dest: W) -> Self {
        Self { dest }
    }

    fn render_static_type(dest: &mut String, ty: &TyKind) -> std::fmt::Result {
        match ty {
            TyKind::Primitive(prim) => match prim {
                PrimitiveType::String => write!(dest, "string")?,
                PrimitiveType::Int => write!(dest, "number")?,
            },
            TyKind::UserDefined(ref name) => write!(dest, "{}", name)?,
            TyKind::Array(ref ty) => {
                write!(dest, "Array<")?;
                Self::render_static_type(dest, ty)?;
                write!(dest, ">")?
            }
            TyKind::Nullable(ref ty) => {
                write!(dest, "(")?;
                Self::render_static_type(dest, ty)?;
                write!(dest, ") | null")?
            }
        }

        Ok(())
    }

    fn write_struct(&mut self, struct_: &StructDefinition) -> io::Result<()> {
        const TEMPLATE: &'static str = r#"
/* =========================================== */
        
const _{name}Fields: StructField[] = [
{{ for field in fields }}
  \{ name: "{field.name}", ty: {field.info} },
{{ endfor }}
];
export class {name} extends StructMessage \{
  static create(body: FieldsOf<{name}>): {name} \{
    return Object.assign(new {name}(), body);
  }

{{ for field in fields }}  public {field.name}!: {field.ty};
{{ endfor }}}
_fieldsMap.set({name}, _{name}Fields);
"#;

        let mut tt = TinyTemplate::new();
        tt.add_template("main", TEMPLATE).unwrap();
        tt.set_default_formatter(&tinytemplate::format_unescaped);

        let context = StructContext {
            name: &struct_.name,
            fields: struct_
                .fields
                .iter()
                .map(|field| RenderedField {
                    name: &field.name,
                    info: {
                        let mut info = String::new();
                        let ts_field: TsFieldType = (&field.datatype).into();
                        ts_field.write_ts_field_type(&mut info);
                        info
                    },
                    ty: {
                        let mut ty = String::new();
                        Self::render_static_type(&mut ty, &field.datatype).unwrap();
                        ty
                    },
                })
                .collect(),
        };

        let rendered = tt.render("main", &context).unwrap();

        write!(&mut self.dest, "{}", rendered)?;

        Ok(())
    }

    fn generate(&mut self, program: &Program) -> io::Result<()> {
        write!(&mut self.dest, "{}", TS_HEADER.trim_start())?;

        for struct_ in program.structs.iter() {
            self.write_struct(struct_);
        }

        Ok(())
    }
}

const TS_HEADER: &'static str = r#"
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
    let result: Record<string, any> = {};

    let fields = _fieldsMap.get(Object.getPrototypeOf(value).constructor)!;
    for (let i = 0; i < fields.length; ++i) {
      const { name: fieldName, ty } = fields[i];
      const fieldVal = value[fieldName];
      result[fieldName] = valueToPlainObject(fieldVal, ty);
    }

    return result;
  }
  //
  else if (ty.kind === TyKindTag.Array) {
    return (value as any[]).map((val) => valueToPlainObject(val, ty.of));
  } else {
    throw new Error("Invalid value/ty");
  }
}

export type Constructor<T> = new(...arguments_: any) => T;

let _fieldsMap: Map<Constructor<StructMessage>, StructField[]> = new Map();

abstract class Message {}
abstract class StructMessage extends Message {}

namespace forser {
  export function packMessage<M extends Message>(message: M) {
    return JSON.stringify(
      valueToPlainObject(message, { kind: TyKindTag.Message })
    );
  }
}

type FieldsOf<T> = Pick<
  T,
  {
    [K in keyof T]: T[K] extends Function ? never : K;
  }[keyof T]
>;
"#;
