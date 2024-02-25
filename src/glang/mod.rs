use std::default::Default;
use std::io::Write;

use crate::items::Program;

// #[derive(Debug, Clone)]
// struct Line {
//     indent: usize,
//     empty: bool,
//     content: &'t str,
// }

#[derive(Debug, Clone, Default)]
pub struct Section<'t> {
    // lines: Vec<Line>,
    body: &'t str,
}

#[derive(Default)]
pub struct Template<'t> {
    prelude: Section<'t>,
    types: Section<'t>,
    type_visitor: Section<'t>,
    field_visitor: Section<'t>,
    message_struct: Section<'t>,
}

impl<'t> Template<'t> {
    pub fn compile(content: &str) {
        let mut template = Template::default();
        let lines = content.split_inclusive('\n');
        let mut cur_section: Option<&mut Section> = None;

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

            // println!("{}", line);

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

                    section.body = &content[start..current];
                }
            }

            current = current_new;
        }


        println!("{}", template.message_struct.body.trim());

    }
}

// pub fn generate_from_template(template: &str) {}
