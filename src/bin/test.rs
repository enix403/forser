use forser::glang::*;

fn main() {
    generate_from_template(CODE);
}

static CODE: &'static str = r#"
#prelude

type TyKind =
  | { kind: TyKindTag.Primitive }
  | { kind: TyKindTag.Message; of: MessageClassID }
  | { kind: TyKindTag.Array; of: TyKind };

type StructField = {
  name: string;
  ty: TyKind;
};

#end/prelude

// -----------------------------------------------------

#types

string = string;
int    = number;
float  = float;
bool   = boolean;
array  = Array<%T%>;
null   = %T% | null;

#end/types

// -----------------------------------------------------

#type_visitor

primitive = { kind: TyKindTag.Primitive };

message = {
    kind: TyKindTag.Message,
    of: "%name%"
};

array = {
    kind: TyKindTag.Array,
    of: %of%
};

#end/type_visitor

// -----------------------------------------------------

#field_visitor
public %name%!: %ty%;
#end/field_visitor

// -----------------------------------------------------

#message_struct

const _%name%Fields: StructField[] = [
    %type_ast%
];

export class %name% extends StructMessage {
    %fields%
}

_messageMap.set("%name%", %name%);
_fieldsMap.set(%name%, _%name%Fields);

/* ================================ */

#end/message_struct
"#;