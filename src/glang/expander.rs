use std::io;

pub trait AssemblyContext: Clone {
    fn write(&mut self, s: &str) -> io::Result<()>;
}

pub trait Expander<C> {
    /// Number of items this expander will emit
    fn count(&self) -> usize;

    /// Returns true if still more items need to be
    /// expanded, false otherwise
    fn expand_next(&mut self, context: &mut C, base_indent: u16) -> io::Result<()>;
}
