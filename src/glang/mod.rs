use std::collections::HashMap;
use std::default::Default;
use std::io::Write;

use crate::items::{Program, StructDefinition};

pub mod expanders;
pub mod scope;
pub mod span;

use expanders::*;
use scope::Scope;
use span::TemplateSpan;

#[derive(Debug, Clone, Default)]
pub struct Section<'t> {
    body: &'t str,
}

#[derive(Debug, Default)]
pub struct Template<'t> {
    prelude: Section<'t>,
    types: Section<'t>,
    type_visitor: Section<'t>,
    field_visitor: Section<'t>,
    message_struct: Section<'t>,
}

impl<'t> Template<'t> {
    pub fn compile(content: &'t str) -> Self {
        let mut template = Template::default();
        let lines = content.split_inclusive('\n');

        let mut cur_section: Option<&mut Section<'t>> = None;
        let mut start = 0;
        let mut current = 0;

        for line in lines {
            let len = line.len();
            let current_new = current + len;

            // Strip trailing newlines
            let line = line
                .strip_suffix('\n')
                .map(|line| line.strip_suffix('\r').unwrap_or(line))
                .unwrap_or(line);

            if line.trim().starts_with("#") {
                let line = line.trim().strip_prefix("#").unwrap();
                let (is_start, name) = if line.starts_with("end/") {
                    (false, line.strip_prefix("end/").unwrap())
                } else {
                    (true, line)
                };

                if is_start {
                    if cur_section.is_none() {
                        cur_section = Some(match name {
                            "prelude" => &mut template.prelude,
                            "types" => &mut template.types,
                            "type_visitor" => &mut template.type_visitor,
                            "field_visitor" => &mut template.field_visitor,
                            "message_struct" => &mut template.message_struct,
                            _ => panic!("Unknown Section"),
                        });

                        start = current_new;
                    } else {
                        panic!("Already in section");
                    }
                } else {
                    let section = cur_section.take().unwrap();

                    section.body = &content[start..current].trim();
                }
            }

            current = current_new;
        }

        template
    }

    pub fn print(&self, program: &Program) {
        // write prelude
        println!("{}", self.prelude.body);

        for struct_ in program.structs.iter() {

            let mut scope = Scope::new()
                .add_text("name", &struct_.name);
                // .add_expander("type_ast", TypeAstExpander::new());

            // TODO: compile only once instead of for each new struct
            let span = TemplateSpan::compile(self.message_struct.body);

            span.print(0, scope);

            print!("\n");
        }
    }
}
