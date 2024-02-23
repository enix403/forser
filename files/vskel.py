"""
struct Foo {
  x: int?,
  bar: [[Bar]]
}

struct Bar {
  y: int
}
"""

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
    kind: TyKind

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
                f.kind
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

# ================================ #

_FooFields: list[StructField] = [
    StructField('x', kind=TyKind('primitive')),
    StructField('bar', kind=TyKind(
        'array',
        of=TyKind(
            'array',
            of=TyKind(
                'message',
                ctor='Bar'
            )
        )
    )),
]

@dataclass
class Foo(StructMessage):
    x: Optional[int]
    bar: list[list['Bar']]

_fields_map[Foo] = _FooFields

# ================================ #

_BarFields: list[StructField] = [
    StructField('y', kind=TyKind('primitive')),
]

@dataclass
class Bar(StructMessage):
    y: int

_fields_map[Bar] = _BarFields 

foo = Foo(
    x=45,
    bar=[
        [Bar(y=1), Bar(y=2)],
        [Bar(y=3), Bar(y=4)],
    ]
)

print(pack_message(foo))