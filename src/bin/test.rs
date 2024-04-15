#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use forser::glang::*;
use forser::items::Program;
use forser::lexer::ForserFile;
use forser::lexer::Lexer;
use forser::parser::{ParseError, Parser};

fn get_test_program() -> Program {
    let file = ForserFile::new("files/one.fr").unwrap();
    let mut source = file.source();
    let lex = Lexer::new(&mut source);

    let parser = Parser::new(lex);

    parser.parse().unwrap()
}

fn main() {
    let template = Template::compile(CODE);
    let program = get_test_program();

    template.print(&program);
}

static CODE: &'static str = r#"
#types

string { string }
int { number }
float { float }
bool { boolean }
array { Array<%T%> }
null { %T% | null }
struct { %T% }

#end/types

// -----------------------------------------------------

#type_visitor

primitive {{ kind: TyKindTag.Primitive }}

message {{
    kind: TyKindTag.Message,
    of: "%name%"
}}

array {{
    kind: TyKindTag.Array,
    of: %of%
}}

main {{ name: "%name%", ty: %ast% }}

#end/type_visitor

// -----------------------------------------------------

#field_visitor
public %name%!: %ty%;
#end/field_visitor

// -----------------------------------------------------

#message_struct

const _%name%Fields: StructField[] = [
    %type_ast/$%
];

export class %name% extends StructMessage {
    %fields%
}

_messageMap.set("%name%", %name%);
_fieldsMap.set(%name%, _%name%Fields);

#end/message_struct
"#;
