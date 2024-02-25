use std::default::Default;
use std::io::Write;

use crate::items::Program;

#[derive(Debug, Clone)]
struct Line<'t> {
    indent: usize,
    empty: bool,
    content: &'t str,
}

#[derive(Debug, Clone)]
struct Section<'t> {
    // lines: Vec<Line<'t>>,
    body: &'t str,
}

// impl<'t> Section<'t> {
//     fn vertically_stripped(&self) -> &[Line<'t>] {
//         let start = self.lines.iter().take_while(|line| line.empty).count();
//         let count_end = self
//             .lines
//             .iter()
//             .rev()
//             .take_while(|line| line.empty)
//             .count();
//         let end = self.lines.len() - count_end;

//         &self.lines[start..end]
//     }
// }

impl<'t> Default for Section<'t> {
    fn default() -> Self {
        // Self { lines: vec![] }
        unimplemented!()
    }
}

#[derive(Default)]
struct Template<'t> {
    prelude: Section<'t>,
    types: Section<'t>,
    type_visitor: Section<'t>,
    field_visitor: Section<'t>,
    message_struct: Section<'t>,
}

impl<'t> Template<'t> {
    fn create_line(raw: &str) -> Line<'_> {
        if raw.trim().is_empty() {
            Line {
                empty: true,
                indent: 0,
                content: raw,
            }
        } else {
            // Index of first non-whitespace character
            let start = raw
                .char_indices()
                .find(|(index, ch)| !ch.is_whitespace())
                .map(|(index, ch)| index)
                .unwrap_or(0);

            Line {
                empty: false,
                indent: start, // TODO: Account for unicode whitespaces
                content: raw[start..].trim_end(),
            }
        }
    }

    fn compile(content: &'t str) -> Self {
        let mut template = Self::default();

        let mut cur_section: Option<&mut Section<'t>> = None;

        let mut offset_start: usize = 0;

        for mut line in content.lines() {
            if line.trim().starts_with("#") {
                line = line.trim().strip_prefix("#").unwrap();
                let (is_end, name) = if line.starts_with("end/") {
                    (true, line.strip_prefix("end/").unwrap())
                } else {
                    (false, line)
                };

                if !is_end {
                    if cur_section.is_none() {
                        cur_section = Some(match name {
                            "prelude" => &mut template.prelude,
                            "types" => &mut template.types,
                            "type_visitor" => &mut template.type_visitor,
                            "field_visitor" => &mut template.field_visitor,
                            "message_struct" => &mut template.message_struct,
                            _ => panic!("Unknown Section"),
                        });
                    } else {
                        panic!("Already in section");
                    }

                } else {
                    cur_section = None;
                }
                continue;
            } else if line.trim().starts_with("//") {
                continue;
            }

            // match cur_section.as_mut().map(|ptr| &mut **ptr) {
            //     None => {}
            //     Some(section) => {
            //         section.lines.push(Self::create_line(line));
            //     }
            // };
        }

        template
    }
}

pub fn generate_from_template<'t>(template: &'t str) {
    // let template = Template::compile(template);

    // let mut output = String::new();

    // for line in template.prelude.vertically_stripped() {
    //     output.push_str(&std::iter::repeat(' ').take(line.indent).collect::<String>());
    //     output.push_str(line.content);
    //     output.push('\n');
    // }

    // println!("{}", output);
}
