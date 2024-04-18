use std::io::{self, Write};
use super::template::{ExpandOptions, Template};

pub trait Expander<W> {
    fn expand(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &ExpandOptions,
        template: &Template<'_>,
    ) -> io::Result<()>;
}

pub struct TextExpander<'a>(/* text: */ pub &'a str);

impl<'a, W: Write> Expander<W> for TextExpander<'a> {
    fn expand(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &ExpandOptions,
        template: &Template<'_>,
    ) -> io::Result<()> {
        write!(dest, "{}", self.0)
    }
}