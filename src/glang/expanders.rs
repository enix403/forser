pub trait Expander {
    fn expand(&self, base_indent: u16);
}

// struct FieldAstSpans {

// }

pub struct TypeAstExpander;

impl TypeAstExpander {
    pub fn new() -> Self {
        Self
    }
}

impl Expander for TypeAstExpander {
    fn expand(&self, base_indent: u16) {
        
    }
}