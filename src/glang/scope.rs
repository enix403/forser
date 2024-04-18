use super::expander::{Expander, TextExpander};

use std::collections::HashMap;
use std::io::Write;

struct ScopeEntry<'a, W> {
    expander: Box<dyn Expander<W> + 'a>,
}

pub struct Scope<'a, W> {
    entries: HashMap<&'static str, ScopeEntry<'a, W>>,
}

impl<'a, W> Scope<'a, W> {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn add_text(mut self, name: &'static str, text: &'a str) -> Self
    where
        W: Write,
    {
        self.add_expander(name, TextExpander(text))
    }

    pub fn add_expander<E: Expander<W> + 'a>(mut self, name: &'static str, expander: E) -> Self {
        self.entries.insert(
            name,
            ScopeEntry {
                expander: Box::new(expander),
            },
        );
        self
    }

    pub fn get_expander(&mut self, name: &'_ str) -> &'_ mut dyn Expander<W> {
        self.entries
            .get_mut(name)
            .map(|entry| entry.expander.as_mut())
            .unwrap_or_else(|| {
                panic!("Unknown variable %{}%", name);
            })
    }
}
