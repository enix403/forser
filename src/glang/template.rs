#[derive(Debug, Clone, Default)]
pub struct ExpandOptions {
    // The delimeter between items emitted from this (multi) variable
    pub delimeter: Option<char>,
    // Should the delimeter be emitted after the last item ?
    pub trailing: bool,
}

#[derive(Clone, Debug)]
pub enum Instruction<'t> {
    Newline,

    Indent(u16),

    Literal(&'t str),

    Expand { var: &'t str, opts: ExpandOptions },
}

#[derive(Clone, Debug, Default)]
pub struct TemplateSpan<'t> {
    pub instructions: Vec<Instruction<'t>>,
}

fn compile_span<'t>(content: &'t str) -> TemplateSpan<'t> {
    let mut instructions = vec![];

    #[derive(Clone, Copy, Debug)]
    enum State {
        Indenting,
        Literal,
        Variable,
        Closed,
    }

    let mut is_tail = false;

    for line in content.lines() {
        if is_tail {
            instructions.push(Instruction::Newline);
        } else {
            is_tail = true;
        }

        let mut indent = 0;
        let mut start = 0;
        let mut state = State::Indenting;

        let chars = line.char_indices().collect::<Vec<_>>();
        let num_chars = chars.len();

        let mut i: usize = 0;

        while i < num_chars {
            let (index, c) = chars[i];

            match (state, c) {
                (State::Indenting, c) if c.is_whitespace() => {
                    indent += 1;
                }
                (State::Indenting, c) => {
                    if indent > 0 {
                        instructions.push(Instruction::Indent(indent));
                    }
                    state = if c == '%' {
                        State::Variable
                    } else {
                        State::Literal
                    };
                    // NOTE: % is included in start in case of State::Variable
                    start = index;
                }

                (State::Literal, c) => {
                    if c == '%' {
                        if index > start {
                            instructions.push(Instruction::Literal(&line[start..index]));
                        }
                        state = State::Variable;
                        // NOTE: % is included in start
                        start = index;
                    }
                }

                (State::Variable, c) => {
                    let percentage_size = '%'.len_utf16();
                    if c == '%' {
                        // ignore the starting %
                        start += percentage_size;

                        instructions.push(Instruction::Expand {
                            var: &line[start..index],
                            opts: ExpandOptions::default(),
                        });

                        state = State::Literal;
                        start = index + percentage_size;
                    } else if c == '/' {
                        // options
                        let (_, delimeter) = chars[i + 1];
                        let trailing = chars[i + 2].1 == '+';

                        // ignore the starting %
                        let var_start = start + percentage_size;

                        // Update the start before as it might later become State::Closed
                        state = State::Literal;

                        {
                            let new_index = if trailing { i + 4 } else { i + 3 };

                            assert_eq!(chars[new_index - 1].1, '%');
                            if new_index >= num_chars {
                                state = State::Closed;
                            } else {
                                start = chars[new_index].0;
                            }
                            i = new_index;
                        }

                        instructions.push(Instruction::Expand {
                            var: &line[var_start..index],
                            opts: ExpandOptions {
                                delimeter: Some(delimeter),
                                trailing,
                            },
                        });
                    };
                }

                _ => (),
            }

            i += 1;
        }

        if let State::Closed = state {
            // ... do nothing ...
        } else if let State::Literal = state {
            let lit = &line[start..];
            if !lit.is_empty() {
                instructions.push(Instruction::Literal(lit));
            }
        } else if let State::Indenting = state {
            if indent > 0 {
                instructions.push(Instruction::Indent(indent));
            }
        } else {
            // syntax error
            panic!("Syntax error in TemplateSpan {:?}", state);
        }
    }

    TemplateSpan { instructions }
}

/* ==================================== */
/* ==================================== */
/* ==================================== */

#[derive(Debug, Default)]
struct TemplateSections<'a> {
    prelude: &'a str,
    types: &'a str,
    type_visitor: &'a str,
    field_visitor: &'a str,
    message_struct: &'a str,
}

fn compile_template_sections<'a>(source: &'a str) -> TemplateSections<'a> {
    let mut sections = TemplateSections::default();

    let lines = source.split_inclusive('\n');

    let mut cur_section: Option<&mut &'a str> = None;
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
                        "prelude" => &mut sections.prelude,
                        "types" => &mut sections.types,
                        "type_visitor" => &mut sections.type_visitor,
                        "field_visitor" => &mut sections.field_visitor,
                        "message_struct" => &mut sections.message_struct,
                        _ => panic!("Unknown Section"),
                    });

                    start = current_new;
                } else {
                    panic!("Already in section");
                }
            } else {
                let section = cur_section.take().unwrap();

                *section = &source[start..current];
            }
        }

        current = current_new;
    }

    sections
}

/* ==================================== */
/* ==================================== */
/* ==================================== */

#[derive(Debug, Clone, Default)]
pub struct Template<'t> {
    pub prelude: &'t str,

    pub field_string: TemplateSpan<'t>,
    pub field_int: TemplateSpan<'t>,
    pub field_float: TemplateSpan<'t>,
    pub field_bool: TemplateSpan<'t>,
    pub field_array: TemplateSpan<'t>,
    pub field_null: TemplateSpan<'t>,
    pub field_struct: TemplateSpan<'t>,
    /* ... */
    pub ast_primitive: TemplateSpan<'t>,
    pub ast_message: TemplateSpan<'t>,
    pub ast_array: TemplateSpan<'t>,
    pub ast_main: TemplateSpan<'t>,
    /* ... */
    pub field_body: TemplateSpan<'t>,
    /* ... */
    pub message_struct: TemplateSpan<'t>,
}

pub fn compile_template<'a>(source: &'a str) -> Template<'a> {
    let sections = compile_template_sections(source);
    let mut template = Template::default();

    stream_parse_visitors(sections.types, |name, span| match name {
        "string" => template.field_string = span,
        "int" => template.field_int = span,
        "float" => template.field_float = span,
        "bool_" => template.field_bool = span,
        "array" => template.field_array = span,
        "null" => template.field_null = span,
        "struct" => template.field_struct = span,
        _ => {}
    });

    stream_parse_visitors(sections.type_visitor, |name, span| match name {
        "primitive" => template.ast_primitive = span,
        "message" => template.ast_message = span,
        "array" => template.ast_array = span,
        "main" => template.ast_main = span,
        _ => {}
    });

    template.field_body = compile_span(sections.field_visitor);
    template.message_struct = compile_span(sections.message_struct);

    template
}

/* ======================= Utils ======================= */

fn stream_parse_visitors<'t, F>(mut source: &'t str, mut receiver: F)
where
    F: FnMut(&'t str, TemplateSpan<'t>),
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

        receiver(name, compile_span(body.trim()));

        // Remove the trailing (or now, leading) closing bracket after the parsed body
        source = rem.strip_prefix('}').unwrap()
    }
}
