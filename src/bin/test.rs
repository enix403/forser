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
    // let template = Template::compile(CODE);
    // let program = get_test_program();

    // template.render(&program);

    let span = TemplateSpan::compile("{\n    kind: %%TyKindTag%%.Message,\n    of: \"%name%\"\n}");
    println!("{:#?}", span)
}

static CODE: &'static str = r#"
#types

string = { string }
int    = { number }
float  = { float }
bool   = { boolean }
array  = { Array<%T%> }
null   = { %T% | null }

#end/types

// -----------------------------------------------------

#type_visitor

primitive = {{ kind: TyKindTag.Primitive }}

message = {{
    kind: TyKindTag.Message,
    of: "%name%"
}}

array = {{
    kind: TyKindTag.Array,
    of: %of%
}}

#end/type_visitor

// -----------------------------------------------------

#message_struct

const _%name%Fields: StructField[] = [
    %%type_ast%%
];

#end/message_struct
"#;

/*

AstExpandeR {
    expand() {
        for f in fields {
            expand_field(f)
        }
    }

    expand_field(f) {
        prim -> print_span(some_indent, prim_inst, empty_scope)
        msg  -> print_span(some_indent, msg_inst, scope with name of message)
        arr  -> print_span(some_indent, arr_inst, FieldExpander(arr.of))
    }
}

*/