use super::scope::Scope;
use super::template::{ExpandOptions, Instruction, Template, TemplateSpan};
use std::io::{self, Write};

pub struct SpanWriter<'a, W> {
    inner: &'a mut W,
}

impl<'a, W: Write> SpanWriter<'a, W> {
    pub fn new(dest: &'a mut W) -> Self {
        Self { inner: dest }
    }

    pub fn write_char(&mut self, c: char) -> io::Result<()> {
        // TODO: optimize
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
}

impl<'a, W> std::ops::Deref for SpanWriter<'a, W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, W> std::ops::DerefMut for SpanWriter<'a, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub fn newline_delimeters<I, F, W>(
    dest: &mut W,
    items: impl Iterator<Item = I>,
    opts: &ExpandOptions,
    indent: u16,
    mut func: F,
) -> io::Result<()>
where
    W: Write,
    F: FnMut(I, &mut W) -> io::Result<()>,
{
    let mut writer = SpanWriter::new(dest);
    let mut is_tail = false;
    for item in items {
        if is_tail {
            if let Some(delim) = &opts.delimeter {
                writer.write_str(delim.as_str())?;
            }
            writer.write_char('\n')?;
            writer.do_indent(indent)?;
        } else {
            is_tail = true;
        }

        // Here &mut SpanWriter<W> deref_mut()'s into &mut W
        func(item, &mut writer)?;
    }

    if opts.trailing && opts.delimeter.is_some() {
        writer.write_str(opts.delimeter.as_ref().unwrap())?;
    }

    Ok(())
}

/* TODO: make sure its result is used */
pub fn render_span<W: Write>(
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
            Instruction::Expand { var, opts } => {
                let expander = scope.get_expander(var);
                expander.expand(&mut writer, indent + current_line_indent, opts, template)?;
            }
        }
    }

    Ok(())
}
