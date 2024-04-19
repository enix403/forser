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
    fn generate(&self, program: &Program, outdir: &Path) {
        let dest = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(outdir.join("main.py"))
            .expect("Failed to open file");

        render_template(GLANG_TEMPLATE, program, dest).unwrap();
    }
}

static GLANG_TEMPLATE: &'static str = r#"
#prelude
from __future__ import annotations
from typing import Literal, Optional, cast, Type, TypeVar, Union, Dict, Any, List
from dataclasses import dataclass
import json

class StructMessage:
    pass

MessageClassID = Union[str, Type[StructMessage]]

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
_message_map: Dict[str, Type[StructMessage]] = {}

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

def _plain_object_to_value(obj: Any, ty: TyKind):
    if obj is None:
        return None

    elif ty.kind == 'primitive':
        return obj

    elif ty.kind == 'message':
        ctor = cast(
            Type[StructMessage],
            _message_map[ty.ctor] if isinstance(ty.ctor, str) else ty.ctor
        )
        fields = _fields_map[ctor]

        create_payload = {}
        for f in fields:
            create_payload[f.name] = _plain_object_to_value(
                obj[f.name],
                f.ty
            )

        return ctor(**create_payload)

    elif ty.kind == 'array':
        arr = cast(list[Any], obj)
        inner = cast(TyKind, ty.of)
        return list(map(
            lambda val: _plain_object_to_value(val, inner),
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

T = TypeVar('T', bound='StructMessage')
def unpack_message(message_type: Type[T], serialized: str) -> T:
    obj = json.loads(serialized)
    result = _plain_object_to_value(
        obj,
        TyKind('message', ctor=message_type)
    )

    return cast(T, result)
#end/prelude

// -------------------------------------------------------------- 

#types

string { str }
int { int }
float { float }
bool { bool }
array { List[%T%] }
null { Optional[%T%] }
struct { %T% }

#end/types

// -------------------------------------------------------------- 

#type_visitor

primitive { TyKind('primitive') }

message { TyKind('message', ctor="%name%") }

array { TyKind(
    'array',
    of=%of%
) }

main { StructField(
    "%name%",
    ty=%ast%
) }

#end/type_visitor

// -------------------------------------------------------------- 

#field_visitor
%name%: %ty%
#end/field_visitor

// -------------------------------------------------------------- 

#message_struct

_%name%Fields: list[StructField] = [
    %type_ast/,%
]

@dataclass
class %name%(StructMessage):
    %fields%

_fields_map[%name%] = _%name%Fields
_message_map['%name%'] = %name%

#end/message_struct
"#;
