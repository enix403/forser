use super::Expander;
use std::io::{self, Write};

use crate::glang::template::{ExpandOptions, Template};

pub struct TextExpander<'a>(/* text: */ pub &'a str);

impl<'a, W: Write> Expander<W> for TextExpander<'a> {
    fn expand(
        &mut self,
        dest: &mut W,
        _indent: u16,
        _opts: &ExpandOptions,
        _template: &Template<'_>,
    ) -> io::Result<()> {
        write!(dest, "{}", self.0)
    }
}
