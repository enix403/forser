use std::collections::HashMap;
use std::default::Default;
use std::io::Write;

use crate::items::{Program, StructDefinition};

pub mod expander;
pub mod scope;
pub mod span;
pub mod struct_expanders;

use scope::Scope;
use span::TemplateSpan;
use struct_expanders::{TypeAstExpander, TypeAstSpans};

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
            let mut spanset = TypeAstSpans {
                primitive: TemplateSpan::empty(),
                message: TemplateSpan::empty(),
                array: TemplateSpan::empty(),
                main: TemplateSpan::empty(),
            };

            stream_parse_visitors(self.type_visitor.body, |name, body| {
                let span = TemplateSpan::compile(body);
                match name {
                    "primitive" => spanset.primitive = span,
                    "message" => spanset.message = span,
                    "array" => spanset.array = span,
                    "main" => spanset.main = span,
                    _ => {}
                }
            });

            let mut scope = Scope::new()
                /* ... */
                .add_text("name", &struct_.name)
                .add_expander(
                    "type_ast",
                    TypeAstExpander::new(spanset, struct_.fields.iter()),
                );

            // TODO: compile only once instead of for each new struct
            let span = TemplateSpan::compile(self.message_struct.body);

            span.print(0, scope);

            print!("\n");
        }
    }
}

fn stream_parse_visitors<'t, F>(mut source: &'t str, mut receiver: F)
where
    F: FnMut(&'t str, &'t str),
{
    loop {
        source = source.trim();

        if source.is_empty() {
            break;
        }

        let (name, rem) = source.split_at(source.find(' ').unwrap());

        let rem = rem.trim_start().strip_prefix('{').unwrap();

        // at this point rem is something like this
        // ..target string..}..extra string...

        let mut brackets_open = 1;
        let mut end_index = None;

        for (i, c) in rem.char_indices() {
            if c == '}' {
                brackets_open -= 1;
                if brackets_open == 0 {
                    end_index = Some(i);
                    break;
                }
            } else if c == '{' {
                brackets_open += 1;
            }
        }

        let (body, rem) = rem.split_at(end_index.unwrap());

        receiver(name, body.trim());

        // Remove the trailing (or now, leading) closing bracket after the parsed body
        source = rem.strip_prefix('}').unwrap()
    }
}
