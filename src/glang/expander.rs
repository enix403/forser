use std::collections::HashMap;
use std::io;

/* ============================ */

pub trait Writable {
    fn write_char(&mut self, c: char) -> io::Result<()>;

    fn write_str(&mut self, s: &str) -> io::Result<()> {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }
}

pub struct WriteContext<W> {
    writer: W,
}

impl<W: Writable> WriteContext<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_char(&mut self, c: char) -> io::Result<()> {
        self.writer.write_char(c)
    }

    pub fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_str(s)
    }

    pub fn do_indent(&mut self, size: u16) -> io::Result<()> {
        // TODO: optimize
        for _ in 0..size {
            self.write_char(' ')?;
        }

        Ok(())
    }
}

/* ============================ */

pub trait Expander<W> {
    /// Number of items this expander will emit
    fn count(&self) -> usize;

    /// Returns true if still more items need to be
    /// expanded, false otherwise
    fn expand_next(&mut self, base_indent: u16, context: &mut WriteContext<W>) -> io::Result<()>;
}

/* ============================ */

pub struct TextExpander<'t> {
    text: &'t str,
}

impl<'t, W: Writable> Expander<W> for TextExpander<'t> {
    fn count(&self) -> usize {
        1
    }

    fn expand_next(&mut self, base_indent: u16, context: &mut WriteContext<W>) -> io::Result<()> {
        context.write_str(self.text)
    }
}

/* ============================ */

pub struct ScopeEntry<'a, W> {
    pub expander: Box<dyn Expander<W> + 'a>,
}

pub struct Scope<'a, W> {
    pub entries: HashMap<&'static str, ScopeEntry<'a, W>>,
}

impl<'a, W: Writable> Scope<'a, W> {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn add_text(mut self, name: &'static str, text: &'a str) -> Self {
        self.add_expander(name, TextExpander { text })
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

    pub fn get_entry(&mut self, name: &'static str) -> &'a mut dyn Expander<W> {
        self.entries
            .get_mut(name)
            .map(|entry| entry.expander.as_mut())
            .unwrap_or_else(|| {
                panic!("Unknown variable %{}%", name);
            })
    }
}

/* ========================== */
