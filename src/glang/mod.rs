use crate::items::Program;
use std::io::{self, Write};

mod emit;
mod expander;
mod message_expanders;
mod scope;
mod template;

use emit::{render_span, SpanWriter};
use message_expanders::FieldExpander;
use scope::Scope;
use template::compile_template;

pub fn render_template<'a, W: Write>(
    source: &'a str,
    program: &Program,
    mut dest: W,
) -> io::Result<()> {
    let template = compile_template(source);

    let mut writer = SpanWriter::new(&mut dest);

    writer.write_str(template.prelude);
    writer.write_char('\n')?;

    for struct_ in program.structs.iter() {
        let scope = Scope::new()
            .add_text("name", &struct_.name)
            .add_expander("fields", FieldExpander::new(struct_.fields.iter()));

        render_span::<W>(
            &template.message_struct,
            &mut writer,
            scope,
            0,
            &template,
        )?;
    }

    writer.write_char('\n')?;

    Ok(())
}
