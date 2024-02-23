use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use std::write;

use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::items::{PrimitiveType, Program, StructDefinition, StructField, TyKind};
use crate::language::Language;

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
    fn generate(&self, program: &Program, outdir: &Path) {
        let dest = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(outdir.join("main.py"))
            .expect("Failed to open file");

        PythonGeneratorInner::new(dest).generate(program);
    }
}

struct PythonGeneratorInner<W> {
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
    Message(String),
}

impl TsFieldType {
    fn write_ts_field_type(&self, dest: &mut String) {
        match self {
            TsFieldType::Primitve => {
                write!(dest, "TyKind('primitive')").unwrap();
            }
            TsFieldType::Message(name) => {
                write!(dest, "TyKind('message', ctor=\"{}\")", name).unwrap();
            }
            TsFieldType::Array(ref inner) => {
                write!(dest, "TyKind('array', of=");
                inner.write_ts_field_type(&mut *dest);
                write!(dest, ")");
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
            TyKind::UserDefined(name) => TsFieldType::Message(name.clone()),
            TyKind::Array(inner) => TsFieldType::Array(Box::new(inner.as_ref().into())),
            TyKind::Nullable(inner) => inner.as_ref().into(),
        }
    }
}

impl<W: Write> PythonGeneratorInner<W> {
    pub fn new(dest: W) -> Self {
        Self { dest }
    }

    fn render_static_type(dest: &mut String, ty: &TyKind) -> std::fmt::Result {
        match ty {
            TyKind::Primitive(prim) => match prim {
                PrimitiveType::String => write!(dest, "str")?,
                PrimitiveType::Int => write!(dest, "int")?,
                PrimitiveType::Float => write!(dest, "float")?,
                PrimitiveType::Bool => write!(dest, "bool")?,
            },
            TyKind::UserDefined(ref name) => write!(dest, "{}", name)?,
            TyKind::Array(ref ty) => {
                write!(dest, "List[")?;
                Self::render_static_type(dest, ty)?;
                write!(dest, "]")?
            }
            TyKind::Nullable(ref ty) => {
                write!(dest, "Optional[")?;
                Self::render_static_type(dest, ty)?;
                write!(dest, "]")?
            }
        }

        Ok(())
    }

    fn write_struct(&mut self, struct_: &StructDefinition) -> io::Result<()> {
        const TEMPLATE: &'static str = r#"
# =========================================== #
        
_{name}Fields: list[StructField] = [
{{ for field in fields }}
  StructField("{field.name}", ty={field.info}),
{{ endfor }}
]
@dataclass
class {name}(StructMessage) :
{{ for field in fields }}    {field.name}: {field.ty}
{{ endfor }}
_fields_map[{name}] = _{name}Fields
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
from __future__ import annotations
from typing import Literal, Optional, cast, Type, TypeVar, Union, Dict, Any, List
from dataclasses import dataclass
import json

T = TypeVar('T', bound='StructMessage')
MessageClassID = Union[str, Type[T]]

class StructMessage:
    pass

@dataclass
class TyKind:
    kind: Literal['primitive', 'message', 'array']
    of: TyKind | None = None
    ctor: MessageClassID | None = None

@dataclass
class StructField:
    name: str
    ty: TyKind

_fields_map: Dict[Type[StructMessage], List[StructField]] = {}

def _value_to_plain_object(value: Any, ty: TyKind) -> Any:
    if value is None:
        return None
    
    elif ty.kind == 'primitive':
        return value

    elif ty.kind == 'message':
        result: Dict[str, Any] = {}

        fields = _fields_map[value.__class__]
        for f in fields:
            result[f.name] = _value_to_plain_object(
                getattr(value, f.name),
                f.ty
            )

        return result

    elif ty.kind == 'array':
        arr = cast(list[Any], value)
        inner = cast(TyKind, ty.of)
        return list(map(
            lambda val: _value_to_plain_object(val, inner),
            arr
        ))
    else:
        raise ValueError("Invalid value/ty")

def pack_message(message: StructMessage):
    return json.dumps(
        _value_to_plain_object(
            message,
            TyKind('message', ctor=message.__class__)
        )
    )
"#;
