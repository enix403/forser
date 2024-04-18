use crate::items::Program;
use std::io::Write;

mod emit;
mod expander;
mod message_expanders;
mod scope;
mod template;

use template::compile_template;
use scope::Scope;
use message_expanders::{FieldExpander, TypeAstExpander};
use emit::render_span;

pub fn render_template<'a, W: Write>(source: &'a str, program: &Program, mut dest: W) {
    let template = compile_template(source);

    writeln!(dest, "{}", template.prelude).unwrap();

    for struct_ in program.structs.iter() {
        let scope = Scope::new()
            .add_text("name", &struct_.name)
            .add_expander("type_ast", TypeAstExpander::new(struct_.fields.iter()))
            .add_expander("fields", FieldExpander::new(struct_.fields.iter()));

        render_span(&template.message_struct, &mut dest, scope, 0, &template);
    }
}
