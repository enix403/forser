use std::io::{self, Write};
use std::marker::PhantomData;

use crate::items::{EnumVariant, EnumVariantValue};

use crate::glang::emit::{newline_delimeters, render_span};
use crate::glang::expander::Expander;
use crate::glang::scope::Scope;
use crate::glang::template::{ExpandOptions, Template};

pub struct EnumVariantsExpander<'a, F> {
    variants: F,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, F> EnumVariantsExpander<'a, F> {
    pub fn new(variants: F) -> Self {
        Self {
            variants,
            _phantom: PhantomData,
        }
    }
}

impl<'a, F, W> Expander<W> for EnumVariantsExpander<'a, F>
where
    W: Write,
    F: Iterator<Item = &'a EnumVariant> + Clone,
{
    fn expand(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &ExpandOptions,
        template: &Template<'_>,
    ) -> io::Result<()> {
        newline_delimeters(
            dest,
            self.variants.clone(),
            opts,
            indent,
            |variant, dest| {
                // let value = match format!("{}", variant.value);
                let value = match &variant.value {
                    EnumVariantValue::Int(val) => val.to_string(),
                    EnumVariantValue::String(val) => format!("\"{}\"", val),
                };

                render_span(
                    &template.enum_variant,
                    dest,
                    Scope::new()
                        .add_text("name", &variant.name)
                        .add_text("val", &value),
                    // .add_expander("ty", FieldTypeExpander(&field.datatype)),
                    indent,
                    template,
                )
            },
        )
    }
}
