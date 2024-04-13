use super::expanders::Expander;
use std::collections::HashMap;

pub enum ScopeValue<'v, 't> {
    Text(&'v str),
    Expand(Box<dyn Expander + 't>),
}

pub struct Scope<'v, 't> {
    // TODO: 'static ?
    pub map: HashMap<&'static str, ScopeValue<'v, 't>>,
}

impl<'v, 't> Scope<'v, 't> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add_text(mut self, name: &'static str, text: &'v str) -> Self {
        self.map.insert(name, ScopeValue::Text(text));
        self
    }

    pub fn add_expander<E: Expander + 't>(mut self, name: &'static str, expander: E) -> Self {
        self.map
            .insert(name, ScopeValue::Expand(Box::new(expander)));
        self
    }
}
