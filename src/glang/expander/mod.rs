use super::template::{ExpandOptions, Template};
use std::io::{self, Write};

pub mod msg_enum;
pub mod msg_struct;
pub mod text;
pub mod typ;

pub trait Expander<W> {
    fn expand(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &ExpandOptions,
        template: &Template<'_>,
    ) -> io::Result<()>;
}
