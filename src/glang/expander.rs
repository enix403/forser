use std::io::{self, Write};

pub trait Expander {
    /// Number of items this expander will emit
    fn count(&self) -> usize;

    /// Returns true if still more items need to be
    /// expanded, false otherwise
    fn expand_next(&mut self, base_indent: u16) -> io::Result<()>;
}
