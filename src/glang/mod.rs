use crate::items::Program;
use std::io::{self, Write};

mod emit;
mod expander;
mod message_expanders;
mod scope;
mod template;
mod span_compiler;

use emit::{render_span, SpanWriter};
use message_expanders::{FieldExpander, EnumVariantsExpander};
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

    // One to end the last 
    // writer.write_char('\n')?;

    for enum_ in program.enums.iter() {
      let scope = Scope::new()
        .add_text("name", &enum_.name)
        .add_expander("variants", EnumVariantsExpander::new(enum_.variants.iter()));

        writer.write_char('\n')?;
        writer.write_char('\n')?;

        render_span::<W>(
            &template.message_enum,
            &mut writer,
            scope,
            0,
            &template,
        )?;
  
    }

    for struct_ in program.structs.iter() {
        let scope = Scope::new()
            .add_text("name", &struct_.name)
            .add_expander("fields", FieldExpander::new(struct_.fields.iter()));

        writer.write_char('\n')?;
        writer.write_char('\n')?;

        render_span::<W>(
            &template.message_struct,
            &mut writer,
            scope,
            0,
            &template,
        )?;
    }

    Ok(())
}
