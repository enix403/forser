use super::expanders::Expander;
use std::collections::HashMap;

pub enum ScopeValue<'a> {
    Text(&'a str),
    Expand(Box<dyn Expander + 'static>),
}

pub struct Scope<'t> {
    // TODO: 'static ?
    pub map: HashMap<&'static str, ScopeValue<'t>>,
}

impl<'t> Scope<'t> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add_text(&mut self, name: &'static str, text: &'t str) {
        self.map.insert(name, ScopeValue::Text(text));
    }

    pub fn add_expander<E: Expander + 'static>(&mut self, name: &'static str, expander: E) {
        self.map
            .insert(name, ScopeValue::Expand(Box::new(expander)));
    }
}
