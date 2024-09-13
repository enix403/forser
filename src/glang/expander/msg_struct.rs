use std::io::{self, Write};
use std::marker::PhantomData;

use crate::items::StructField;

use crate::glang::emit::{newline_delimeters, render_span};
use crate::glang::expander::Expander;
use crate::glang::scope::Scope;
use crate::glang::template::{ExpandOptions, Template};

use super::typ::TypeExpander;

pub struct StructFieldsExpander<'a, F> {
    fields: F,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, F> StructFieldsExpander<'a, F> {
    pub fn new(fields: F) -> Self {
        Self {
            fields,
            _phantom: PhantomData,
        }
    }
}

impl<'a, F, W> Expander<W> for StructFieldsExpander<'a, F>
where
    W: Write,
    F: Iterator<Item = &'a StructField> + Clone,
{
    fn expand(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &ExpandOptions,
        template: &Template<'_>,
    ) -> io::Result<()> {
        newline_delimeters(dest, self.fields.clone(), opts, indent, |field, dest| {
            render_span(
                &template.field_body,
                dest,
                Scope::new()
                    .add_text("name", &field.name)
                    .add_expander("ty", TypeExpander(&field.datatype)),
                indent,
                template,
            )
        })
    }
}
