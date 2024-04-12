use std::default::Default;
use std::io::Write;

use crate::items::{Program, StructDefinition};

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

    pub fn render(&self, program: &Program) {
        // write prelude
        println!("{}", self.prelude.body);

        for struct_ in program.structs.iter() {
            self.render_struct(struct_);
        }
    }

    fn render_struct(&self, struct_: &StructDefinition) {
        // compile type map
        // compile type visitor
        // compile field visitor
        // compile struct
    }
}

enum Instruction<'t> {
    Newline,

    Indent(u16),

    Literal(&'t str),

    EvaluateSingle(&'t str),

    EvaluateMultiple(&'t str),
}

pub struct TemplateSpan<'t> {
    instructions: Vec<Instruction<'t>>,
}

impl<'t> TemplateSpan<'t> {
    fn compile(content: &'t str) {
        let mut span = TemplateSpan {
            instructions: vec![],
        };

        for line in content.lines() {
            let mut start = 0;
            // let mut indenting = true;
            let mut indent = 0;

            #[derive(Clone, Copy)]
            enum State {
                Indenting,
                Literal,
                VariableDetected,
                VariableSingle,
                VariableMutiple,
            }

            let mut state = State::Indenting;

            let mut chars = line.char_indices().peekable();
            let num_chars = line.chars().count();

            while let Some((index, c)) = chars.next() {
                // let p = chars.peek().map(|x| x.1).unwrap_or(' ');
                let p = chars.peek();

                match (state, c, p) {
                    (State::Indenting, c, _) if c.is_whitespace() => {
                        // current +=
                        indent += 1;
                    }
                    (State::Indenting, c, _) => {
                        println!("Indent {}", indent);
                        state = if c == '%' {
                            State::VariableDetected
                        } else {
                            State::Literal
                        };
                        start = index;
                    }

                    (State::Literal, c, _) => {
                        if c == '%' {
                            println!("Literal {}", &line[start..index]);

                            state = State::VariableDetected;
                            start = index;
                        }
                    }

                    (State::VariableDetected, c, p) => {
                        if c == '%' {
                            state = State::VariableMutiple;
                            // Index of next char
                            start = p.map(|p| p.0).unwrap_or(index);
                        } else {
                            state = State::VariableSingle;
                            start = index;
                        }
                    }

                    (State::VariableSingle, c, _) => {
                        if c == '%' {
                            println!("VariableSingle {}", &line[start..index]);

                            state = State::Literal;
                            start = index + c.len_utf16();
                        }
                    }

                    (State::VariableMutiple, c, p) => {
                        if c == '%' && p.is_some() && p.unwrap().1 == '%' {
                            println!("VariableMutiple {}", &line[start..index]);
                            state = State::Literal;
                            start = p.map(|p| p.0).unwrap_or(index) + '%'.len_utf8();
                        }
                    }

                    _ => (),
                }
            }
        }
    }
}
