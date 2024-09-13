#[derive(Debug, Clone, Default)]
pub struct ExpandOptions {
    /// The delimeter between items emitted from this (multi) variable
    pub delimeter: Option<String>,
    /// Should the delimeter be emitted after the last item ?
    pub trailing: bool,
    /// Should all the items be emitted on a single line ?
    pub inline: bool,
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

fn parse_replacer<'t>(source: &'t str) -> (&'t str, ExpandOptions) {
    // %var/,/+/i%
    let parts = source.split('/').collect::<Vec<_>>();

    let var = parts[0];

    let mut delimeter = None;
    let mut trailing = false;
    let mut inline = false;

    if parts.len() > 1 {
        delimeter = Some(parts[1].to_string());
        trailing = parts.len() > 2 && parts[2] == "+";
        inline = parts.len() > 3 && parts[3] == "i";
    }

    let opts = ExpandOptions {
        delimeter,
        trailing,
        inline
    };

    (var, opts)
}

pub fn compile_span<'t>(content: &'t str) -> TemplateSpan<'t> {
    let mut instructions: Vec<Instruction> = vec![];

    let re = regex::Regex::new(r"%([^%]+)%").unwrap();

    let mut is_tail = false;

    for line in content.lines() {
        if is_tail {
            instructions.push(Instruction::Newline);
        } else {
            is_tail = true;
        }

        let mut last_end = 0;

        let leading_whitespace_count =
            line.chars().take_while(|c| c.is_whitespace()).count() as u16;

        if leading_whitespace_count > 0 {
            instructions.push(Instruction::Indent(leading_whitespace_count));

            // start the first section after the indent
            last_end = leading_whitespace_count as usize;
        }

        for caps in re.captures_iter(line) {
            let w_start = caps.get(0).unwrap().start();
            let w_end = caps.get(0).unwrap().end();

            let start = caps.get(1).unwrap().start();
            let end = caps.get(1).unwrap().end();

            if w_start > last_end {
                instructions.push(Instruction::Literal(&line[last_end..w_start]));
            }

            let rep_source = &line[start..end];
            let (var, opts) = parse_replacer(rep_source);

            instructions.push(Instruction::Expand { var, opts });

            last_end = w_end;
        }

        if last_end < line.len() {
            instructions.push(Instruction::Literal(&line[last_end..]));
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
    // type_visitor: &'a str,
    // structs
    field_visitor: &'a str,
    message_struct: &'a str,
    // enums
    enum_variant_visitor: &'a str,
    message_enum: &'a str,
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
                        // "type_visitor" => &mut sections.type_visitor,
                        "field_visitor" => &mut sections.field_visitor,
                        "message_struct" => &mut sections.message_struct,
                        "enum_variant_visitor" => &mut sections.enum_variant_visitor,
                        "message_enum" => &mut sections.message_enum,
                        _ => panic!("Unknown Section \"{}\"", name),
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
    pub field_map: TemplateSpan<'t>,
    pub field_null: TemplateSpan<'t>,
    pub field_struct: TemplateSpan<'t>,

    /* ... */
    // pub ast_primitive: TemplateSpan<'t>,
    // pub ast_message: TemplateSpan<'t>,
    // pub ast_array: TemplateSpan<'t>,
    // pub ast_main: TemplateSpan<'t>,

    /* Structs */
    pub field_body: TemplateSpan<'t>,
    pub message_struct: TemplateSpan<'t>,

    /* Enums */
    pub enum_variant: TemplateSpan<'t>,
    pub message_enum: TemplateSpan<'t>,
}

pub fn compile_template<'a>(source: &'a str) -> Template<'a> {
    let sections = compile_template_sections(source);
    let mut template = Template::default();

    template.prelude = sections.prelude.trim();

    stream_parse_visitors(sections.types, |name, span| match name {
        "string" => template.field_string = span,
        "int" => template.field_int = span,
        "float" => template.field_float = span,
        "bool" => template.field_bool = span,
        "array" => template.field_array = span,
        "map" => template.field_map = span,
        "null" => template.field_null = span,
        "struct" => template.field_struct = span,
        _ => {}
    });

    // stream_parse_visitors(sections.type_visitor, |name, span| match name {
    //     "primitive" => template.ast_primitive = span,
    //     "message" => template.ast_message = span,
    //     "array" => template.ast_array = span,
    //     "main" => template.ast_main = span,
    //     _ => {}
    // });

    template.field_body = compile_span(sections.field_visitor.trim());
    template.message_struct = compile_span(sections.message_struct.trim());

    template.enum_variant = compile_span(sections.enum_variant_visitor.trim());
    template.message_enum = compile_span(sections.message_enum.trim());

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
