use crate::items::Program;
use std::io::{self, Write};

mod emit;
mod expander;
mod message_expanders;
mod scope;
mod template;

use emit::render_span;
use message_expanders::{FieldExpander, TypeAstExpander};
use scope::Scope;
use template::compile_template;

pub fn render_template<'a, W: Write>(
    source: &'a str,
    program: &Program,
    mut dest: W,
) -> io::Result<()> {
    let template = compile_template(source);

    writeln!(dest, "{}", template.prelude)?;

    for struct_ in program.structs.iter() {
        let scope = Scope::new()
            .add_text("name", &struct_.name)
            .add_expander("type_ast", TypeAstExpander::new(struct_.fields.iter()))
            .add_expander("fields", FieldExpander::new(struct_.fields.iter()));

        render_span(&template.message_struct, &mut dest, scope, 0, &template)?;
    }

    Ok(())
}
