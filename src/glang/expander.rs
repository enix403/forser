pub trait Expander {
    /// Returns true if still more items need to be
    /// expanded, false otherwise
    fn expand_next(&mut self, base_indent: u16) -> bool;
}
