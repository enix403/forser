use crate::items::{PrimitiveType, Program, StructField, TyKind};
use std::io;
use std::marker::PhantomData;
use std::{collections::HashMap, io::Write};

/* ==================================== */
/* ==================================== */
/* ==================================== */
/* ==================================== */
/* ==================================== */

#[derive(Debug, Clone, Default)]
pub struct EvaluateOptions {
    // The delimeter between items emitted from this (multi) variable
    delimeter: Option<char>,
    // Should the delimeter be emitted after the last item ?
    trailing: bool,
}

#[derive(Clone, Debug)]
pub enum Instruction<'t> {
    Newline,

    Indent(u16),

    Literal(&'t str),

    Evaluate { var: &'t str, opts: EvaluateOptions },
}

#[derive(Clone, Debug, Default)]
pub struct TemplateSpan<'t> {
    instructions: Vec<Instruction<'t>>,
}

fn compile_span<'t>(content: &'t str) -> TemplateSpan<'t> {
    let mut span = TemplateSpan {
        instructions: vec![],
    };

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
            span.instructions.push(Instruction::Newline);
        } else {
            is_tail = true;
        }

        let mut indent = 0;
        let mut start = 0;
        let mut state = State::Indenting;

        let mut chars = line.char_indices().collect::<Vec<_>>();
        let num_chars = chars.len();

        let mut i: usize = 0;

        while i < num_chars {
            let (index, c) = chars[i];

            match (state, c) {
                (State::Indenting, c) if c.is_whitespace() => {
                    // current +=
                    indent += 1;
                }
                (State::Indenting, c) => {
                    if indent > 0 {
                        span.instructions.push(Instruction::Indent(indent));
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
                        if (index > start) {
                            span.instructions
                                .push(Instruction::Literal(&line[start..index]));
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

                        span.instructions
                            // .push(Instruction::Evaluate(&line[start..index]));
                            .push(Instruction::Evaluate {
                                var: &line[start..index],
                                opts: EvaluateOptions::default(),
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

                        span.instructions.push(Instruction::Evaluate {
                            var: &line[var_start..index],
                            opts: EvaluateOptions {
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
                span.instructions.push(Instruction::Literal(lit));
            }
        } else if let State::Indenting = state {
            if indent > 0 {
                span.instructions.push(Instruction::Indent(indent));
            }
        } else {
            // syntax error
            panic!("Syntax error in TemplateSpan {:?}", state);
        }
    }

    span
}

/* ==================================== */
/* ==================================== */

#[derive(Debug, Clone, Default)]
pub struct Section<'t> {
    body: &'t str,
}

#[derive(Debug, Clone, Default)]
struct Template<'t> {
    prelude: Section<'t>,
    /* ... */
    field_string: TemplateSpan<'t>,
    field_int: TemplateSpan<'t>,
    field_float: TemplateSpan<'t>,
    field_bool: TemplateSpan<'t>,
    field_array: TemplateSpan<'t>,
    field_null: TemplateSpan<'t>,
    field_struct_: TemplateSpan<'t>,
    /* ... */
    ast_primitive: TemplateSpan<'t>,
    ast_message: TemplateSpan<'t>,
    ast_array: TemplateSpan<'t>,
    ast_main: TemplateSpan<'t>,
    /* ... */
    field_body: TemplateSpan<'t>,
    /* ... */
    message_struct: TemplateSpan<'t>,
}

/* ==================================== */

pub struct SpanWriter<'a, W> {
    inner: &'a mut W,
}

impl<'a, W: Write> SpanWriter<'a, W> {
    pub fn new(dest: &'a mut W) -> Self {
        Self { inner: dest }
    }

    pub fn write_char(&mut self, c: char) -> io::Result<()> {
        write!(self.inner, "{}", c)
    }

    pub fn write_str(&mut self, s: &str) -> io::Result<()> {
        write!(self.inner, "{}", s)
    }

    pub fn do_indent(&mut self, size: u16) -> io::Result<()> {
        // TODO: optimize
        for _ in 0..size {
            self.write_char(' ')?;
        }

        Ok(())
    }

    pub fn get_mut_io_writer(&mut self) -> &'_ mut W {
        &mut self.inner
    }
}

/* ==================================== */

trait Evaluater<W> {
    fn evaluate(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &EvaluateOptions,
        template: &Template<'_>,
    );
}

struct TextEvaluater<'a> {
    text: &'a str,
}

impl<'a> TextEvaluater<'a> {
    fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a, W: Write> Evaluater<W> for TextEvaluater<'a> {
    fn evaluate(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &EvaluateOptions,
        template: &Template<'_>,
    ) {
        write!(dest, "{}", self.text).unwrap();
    }
}

/* ==================================== */

struct ScopeEntry<'a, W> {
    pub evaluater: Box<dyn Evaluater<W> + 'a>,
}

struct Scope<'a, W> {
    pub entries: HashMap<&'static str, ScopeEntry<'a, W>>,
}

impl<'a, W> Scope<'a, W> {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn add_text(mut self, name: &'static str, text: &'a str) -> Self
    where
        W: Write,
    {
        self.add_evaluater(name, TextEvaluater { text })
    }

    pub fn add_evaluater<E: Evaluater<W> + 'a>(mut self, name: &'static str, evaluater: E) -> Self {
        self.entries.insert(
            name,
            ScopeEntry {
                evaluater: Box::new(evaluater),
            },
        );
        self
    }

    pub fn get_entry(&mut self, name: &'_ str) -> &'_ mut dyn Evaluater<W> {
        self.entries
            .get_mut(name)
            .map(|entry| entry.evaluater.as_mut())
            .unwrap_or_else(|| {
                panic!("Unknown variable %{}%", name);
            })
    }
}

/* ==================================== */
/* ==================================== */

fn newline_delimeters<I, F, W>(
    dest: &mut W,
    items: impl Iterator<Item = I>,
    opts: &EvaluateOptions,
    indent: u16,
    mut func: F,
) where
    W: Write,
    F: FnMut(I, &mut W)
{
    let mut writer = SpanWriter::new(dest);
    for (index, item) in items.enumerate() {
        if index != 0 {
            if let Some(delim) = opts.delimeter {
                // TDOD: to string
                writer.write_char(delim);
            }
            writer.write_char('\n');
            writer.do_indent(indent);
        }
        func(item, writer.get_mut_io_writer());
    }

    if opts.trailing && opts.delimeter.is_some() {
        writer.write_char(opts.delimeter.unwrap());
    }
}

/* ==================================== */
/* ==================================== */

struct TypeAstNodeEvaluater<'a> {
    ty: &'a TyKind,
}

impl<'a> TypeAstNodeEvaluater<'a> {
    fn new(ty: &'a TyKind) -> Self {
        Self { ty }
    }
}

impl<'a, W: Write> Evaluater<W> for TypeAstNodeEvaluater<'a> {
    fn evaluate(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &EvaluateOptions,
        template: &Template<'_>,
    ) {
        match self.ty {
            TyKind::Primitive(prim) => {
                render_span(
                    &template.ast_primitive,
                    dest,
                    Scope::new(),
                    indent,
                    template,
                );
            }

            TyKind::UserDefined(ref name) => {
                render_span(
                    &template.ast_message,
                    dest,
                    Scope::new().add_text("name", &name),
                    indent,
                    template,
                );
            }

            TyKind::Array(ref inner) => {
                render_span(
                    &template.ast_array,
                    dest,
                    Scope::new().add_evaluater("of", TypeAstNodeEvaluater::new(&inner)),
                    indent,
                    template,
                );
            }

            TyKind::Nullable(ref inner) => TypeAstNodeEvaluater::new(inner).evaluate(
                dest,
                indent,
                &EvaluateOptions::default(),
                template,
            ),
        }
    }
}

/* ---------------------------------------- */

pub struct TypeAstEvaluater<'a, F> {
    fields: F,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, F> TypeAstEvaluater<'a, F> {
    pub fn new(fields: F) -> Self {
        Self {
            fields,
            _phantom: PhantomData,
        }
    }
}

impl<'a, F, W> Evaluater<W> for TypeAstEvaluater<'a, F>
where
    W: Write,
    F: Iterator<Item = &'a StructField> + Clone,
{
    fn evaluate(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &EvaluateOptions,
        template: &Template<'_>,
    ) {
        newline_delimeters(dest, self.fields.clone(), opts, indent, |field, dest| {
            render_span(
                &template.ast_main,
                dest,
                Scope::new()
                    .add_text("name", &field.name)
                    .add_evaluater("ast", TypeAstNodeEvaluater::new(&field.datatype)),
                indent,
                template,
            )
            .unwrap();
        });
    }
}

/* ==================================== */
/* ==================================== */

pub struct FieldTypeEvaluater<'s>(&'s TyKind);

impl<'a, W: Write> Evaluater<W> for FieldTypeEvaluater<'a> {
    fn evaluate(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &EvaluateOptions,
        template: &Template<'_>,
    ) {
        match self.0 {
            TyKind::Primitive(prim) => match prim {
                PrimitiveType::String => {
                    render_span(&template.field_string, dest, Scope::new(), indent, template);
                }
                PrimitiveType::Int => {
                    render_span(&template.field_int, dest, Scope::new(), indent, template);
                }
                PrimitiveType::Float => {
                    render_span(&template.field_float, dest, Scope::new(), indent, template);
                }
                PrimitiveType::Bool => {
                    render_span(&template.field_bool, dest, Scope::new(), indent, template);
                }
            },

            TyKind::UserDefined(name) => {
                render_span(
                    &template.field_struct_,
                    dest,
                    Scope::new().add_text("T", &name),
                    indent,
                    template,
                );
            }

            TyKind::Nullable(inner) => {
                render_span(
                    &template.field_null,
                    dest,
                    Scope::new().add_evaluater("T", FieldTypeEvaluater(inner.as_ref())),
                    indent,
                    template,
                );
            }

            TyKind::Array(inner) => {
                render_span(
                    &template.field_array,
                    dest,
                    Scope::new().add_evaluater("T", FieldTypeEvaluater(inner.as_ref())),
                    indent,
                    template,
                );
            }
        }
    }
}

/* ---------------------------------------- */

pub struct FieldEvaluater<'a, F> {
    fields: F,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, F> FieldEvaluater<'a, F> {
    pub fn new(fields: F) -> Self {
        Self {
            fields,
            _phantom: PhantomData,
        }
    }
}

impl<'a, F, W> Evaluater<W> for FieldEvaluater<'a, F>
where
    W: Write,
    F: Iterator<Item = &'a StructField> + Clone,
{
    fn evaluate(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &EvaluateOptions,
        template: &Template<'_>,
    ) {
        newline_delimeters(dest, self.fields.clone(), opts, indent, |field, dest| {
            render_span(
                &template.field_body,
                dest,
                Scope::new()
                    .add_text("name", &field.name)
                    .add_evaluater("ty", FieldTypeEvaluater(&field.datatype)),
                indent,
                template,
            )
            .unwrap();
        });
    }
}

/* ==================================== */

/* TODO: make sure its result is used */
fn render_span<W: Write>(
    span: &TemplateSpan,
    dest: &mut W,
    mut scope: Scope<W>,
    indent: u16,
    template: &Template,
) -> io::Result<()> {
    let mut writer = SpanWriter::new(dest);

    let mut current_line_indent = 0;

    for inst in span.instructions.iter() {
        match inst {
            Instruction::Newline => {
                writer.write_char('\n')?;
                current_line_indent = 0;
                writer.do_indent(indent)?;
            }
            Instruction::Indent(size) => {
                let size = *size;
                current_line_indent = size;
                writer.do_indent(size)?;
            }
            Instruction::Literal(val) => {
                writer.write_str(val)?;
            }
            Instruction::Evaluate { var, opts } => {
                let evaluater = scope.get_entry(var);
                evaluater.evaluate(
                    writer.get_mut_io_writer(),
                    indent + current_line_indent,
                    opts,
                    template,
                );
            } /* End Match Instruction */
        }
    }

    Ok(())
}

/* ==================================== */
/* ==================================== */
/* ==================================== */
/* ==================================== */
/* ==================================== */

pub mod driver {
    use super::*;

    #[derive(Debug, Default)]
    struct TemplateSections<'t> {
        pub prelude: Section<'t>,
        pub types: Section<'t>,
        pub type_visitor: Section<'t>,
        pub field_visitor: Section<'t>,
        pub message_struct: Section<'t>,
    }

    fn compile_template_sections<'a>(source: &'a str) -> TemplateSections<'a> {
        let mut sections = TemplateSections::default();

        let lines = source.split_inclusive('\n');

        let mut cur_section: Option<&mut Section<'a>> = None;
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

                    section.body = &source[start..current];
                }
            }

            current = current_new;
        }

        sections
    }

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

    pub fn render_template<'a, W: Write>(source: &'a str, program: &Program, mut dest: W) {
        /**/

        // struct Template<'t> {
        //     prelude: Section<'t>,
        //     /* ... */
        //     field_string: TemplateSpan<'t>,
        //     field_int: TemplateSpan<'t>,
        //     field_float: TemplateSpan<'t>,
        //     field_bool: TemplateSpan<'t>,
        //     field_array: TemplateSpan<'t>,
        //     field_null: TemplateSpan<'t>,
        //     field_struct_: TemplateSpan<'t>,
        //     /* ... */
        //     ast_primitive: TemplateSpan<'t>,
        //     ast_message: TemplateSpan<'t>,
        //     ast_array: TemplateSpan<'t>,
        //     ast_main: TemplateSpan<'t>,
        //     /* ... */
        //     field_body: TemplateSpan<'t>,
        //     /* ... */
        //     message_struct: Section<'t>,
        // }

        let sections = compile_template_sections(source);
        let mut template = Template::default();

        template.prelude = sections.prelude.clone();

        stream_parse_visitors(sections.types.body, |name, span| match name {
            "string" => template.field_string = span,
            "int" => template.field_int = span,
            "float" => template.field_float = span,
            "bool_" => template.field_bool = span,
            "array" => template.field_array = span,
            "null" => template.field_null = span,
            "struct" => template.field_struct_ = span,
            _ => {}
        });

        stream_parse_visitors(sections.type_visitor.body, |name, span| match name {
            "primitive" => template.ast_primitive = span,
            "message" => template.ast_message = span,
            "array" => template.ast_array = span,
            "main" => template.ast_main = span,
            _ => {}
        });

        template.field_body = compile_span(sections.field_visitor.body);
        template.message_struct = compile_span(sections.message_struct.body);

        /* ===================== */

        // write
        writeln!(&mut dest, "{}", template.prelude.body).unwrap();

        for struct_ in program.structs.iter() {
            let scope = Scope::new()
                /* ... */
                .add_text("name", &struct_.name)
                .add_evaluater("type_ast", TypeAstEvaluater::new(struct_.fields.iter()))
                .add_evaluater("fields", FieldEvaluater::new(struct_.fields.iter()));
            // .add_expander(
            //     "fields",
            //     FieldsExpander::new(struct_.fields.iter(), &field_spanset, &field_body_span),
            // );

            // message_body_span.print(0, &mut WriteContext::new(ConsoleContext), scope);
            render_span(&template.message_struct, &mut dest, scope, 0, &template);

            print!("\n");
        }
    }
}

/*
#[derive(Debug, Clone, Default)]
pub struct Section<'t> {
    body: &'t str,
}

#[derive(Debug, Default)]
pub struct Template<'t> {
    pub prelude: Section<'t>,
    pub types: Section<'t>,
    pub type_visitor: Section<'t>,
    pub field_visitor: Section<'t>,
    pub message_struct: Section<'t>,
}

fn compile_template<'t>(content: &'t str) -> Template<'t> {
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

                section.body = &content[start..current];
            }
        }

        current = current_new;
    }

    template
}
*/
