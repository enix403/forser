use std::io::{self, Write};
use super::template::{ExpandOptions, Template};

pub mod text;
pub mod typ;
pub mod msg_struct;
pub mod msg_enum;

pub trait Expander<W> {
    fn expand(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &ExpandOptions,
        template: &Template<'_>,
    ) -> io::Result<()>;
}