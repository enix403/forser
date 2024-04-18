use std::io::{self, Write};
use std::marker::PhantomData;

use crate::items::{PrimitiveType, StructField, TyKind};

use super::emit::{newline_delimeters, render_span};
use super::expander::Expander;
use super::scope::Scope;
use super::template::{ExpandOptions, Template};

struct TypeAstNodeExpander<'a>(&'a TyKind);

impl<'a, W: Write> Expander<W> for TypeAstNodeExpander<'a> {
    fn expand(
        &mut self,
        dest: &mut W,
        indent: u16,
        _opts: &ExpandOptions,
        template: &Template<'_>,
    ) -> io::Result<()> {
        match self.0 {
            TyKind::Primitive(..) => {
                render_span(
                    &template.ast_primitive,
                    dest,
                    Scope::new(),
                    indent,
                    template,
                )?;
            }

            TyKind::UserDefined(ref name) => {
                render_span(
                    &template.ast_message,
                    dest,
                    Scope::new().add_text("name", &name),
                    indent,
                    template,
                )?;
            }

            TyKind::Array(ref inner) => {
                render_span(
                    &template.ast_array,
                    dest,
                    Scope::new().add_expander("of", TypeAstNodeExpander(&inner)),
                    indent,
                    template,
                )?;
            }

            TyKind::Nullable(ref inner) => {
                TypeAstNodeExpander(inner).expand(
                    dest,
                    indent,
                    &ExpandOptions::default(),
                    template,
                )?;
            }
        }

        Ok(())
    }
}

/* ---------------------------------------- */

pub struct TypeAstExpander<'a, F> {
    fields: F,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, F> TypeAstExpander<'a, F> {
    pub fn new(fields: F) -> Self {
        Self {
            fields,
            _phantom: PhantomData,
        }
    }
}

impl<'a, F, W> Expander<W> for TypeAstExpander<'a, F>
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
                &template.ast_main,
                dest,
                Scope::new()
                    .add_text("name", &field.name)
                    .add_expander("ast", TypeAstNodeExpander(&field.datatype)),
                indent,
                template,
            )
        })
    }
}

/* ==================================== */
/* ==================================== */

pub struct FieldTypeExpander<'s>(&'s TyKind);

impl<'a, W: Write> Expander<W> for FieldTypeExpander<'a> {
    fn expand(
        &mut self,
        dest: &mut W,
        indent: u16,
        _opts: &ExpandOptions,
        template: &Template<'_>,
    ) -> io::Result<()> {
        match self.0 {
            TyKind::Primitive(prim) => match prim {
                PrimitiveType::String => {
                    render_span(&template.field_string, dest, Scope::new(), indent, template)?;
                }
                PrimitiveType::Int => {
                    render_span(&template.field_int, dest, Scope::new(), indent, template)?;
                }
                PrimitiveType::Float => {
                    render_span(&template.field_float, dest, Scope::new(), indent, template)?;
                }
                PrimitiveType::Bool => {
                    render_span(&template.field_bool, dest, Scope::new(), indent, template)?;
                }
            },

            TyKind::UserDefined(name) => {
                render_span(
                    &template.field_struct,
                    dest,
                    Scope::new().add_text("T", &name),
                    indent,
                    template,
                )?;
            }

            TyKind::Nullable(inner) => {
                render_span(
                    &template.field_null,
                    dest,
                    Scope::new().add_expander("T", FieldTypeExpander(inner.as_ref())),
                    indent,
                    template,
                )?;
            }

            TyKind::Array(inner) => {
                render_span(
                    &template.field_array,
                    dest,
                    Scope::new().add_expander("T", FieldTypeExpander(inner.as_ref())),
                    indent,
                    template,
                )?;
            }
        }

        Ok(())
    }
}

/* ---------------------------------------- */

pub struct FieldExpander<'a, F> {
    fields: F,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, F> FieldExpander<'a, F> {
    pub fn new(fields: F) -> Self {
        Self {
            fields,
            _phantom: PhantomData,
        }
    }
}

impl<'a, F, W> Expander<W> for FieldExpander<'a, F>
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
                    .add_expander("ty", FieldTypeExpander(&field.datatype)),
                indent,
                template,
            )
        })
    }
}
