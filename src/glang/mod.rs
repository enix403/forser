use crate::items::Program;
use std::io::{self, Write};

mod emit;
mod expander;
mod scope;
mod template;

use emit::{render_span, SpanWriter};
use expander::msg_struct::StructFieldsExpander;
use expander::msg_enum::EnumVariantsExpander;
use expander::typ::TypeExpander;
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
            .add_expander("fields", StructFieldsExpander::new(struct_.fields.iter()));

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

    for alias in program.type_aliases.iter() {
      let scope = Scope::new()
        .add_text("name", &alias.name)
        .add_expander("T", TypeExpander(&alias.typ));

        writer.write_char('\n')?;
        writer.write_char('\n')?;

        render_span::<W>(
            &template.type_alias,
            &mut writer,
            scope,
            0,
            &template,
        )?;
    }

    Ok(())
}
