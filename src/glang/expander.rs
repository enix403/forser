#[derive(Debug, Clone)]
pub struct MultiVariableOptions {
    // The delimeter between items emitted from this (multi) variable
    delimeter: char,
    // Should the delimeter be emitted after the last item ?
    trailing: bool,
}

pub trait Expander {
    fn expand(&self, base_indent: u16);
}
