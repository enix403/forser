use std::io::{self, Write};
use std::marker::PhantomData;

use crate::items::{PrimitiveType, TyKind};

use crate::glang::emit::{newline_delimeters, render_span};
use crate::glang::expander::Expander;
use crate::glang::scope::Scope;
use crate::glang::template::{ExpandOptions, Template};

struct TupleTypeExpander<'a, F> {
    types: F,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, F> TupleTypeExpander<'a, F> {
    pub fn new(types: F) -> Self {
        Self {
            types,
            _phantom: PhantomData,
        }
    }
}

impl<'a, F, W> Expander<W> for TupleTypeExpander<'a, F>
where
    W: Write,
    F: Iterator<Item = &'a TyKind> + Clone,
{
    fn expand(
        &mut self,
        dest: &mut W,
        indent: u16,
        opts: &ExpandOptions,
        template: &Template<'_>,
    ) -> io::Result<()> {
        newline_delimeters(dest, self.types.clone(), opts, indent, |tykind, dest| {
            render_span(
                &template.echo,
                dest,
                Scope::new().add_expander("value", TypeExpander(tykind)),
                indent,
                template,
            )
        })
    }
}

/* --------- */

pub struct TypeExpander<'s>(pub &'s TyKind);

impl<'a, W: Write> Expander<W> for TypeExpander<'a> {
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
                    Scope::new().add_expander("T", TypeExpander(inner.as_ref())),
                    indent,
                    template,
                )?;
            }

            TyKind::Array(inner) => {
                render_span(
                    &template.field_array,
                    dest,
                    Scope::new().add_expander("T", TypeExpander(inner.as_ref())),
                    indent,
                    template,
                )?;
            }

            TyKind::Map(inner) => {
                render_span(
                    &template.field_map,
                    dest,
                    Scope::new().add_expander("T", TypeExpander(inner.as_ref())),
                    indent,
                    template,
                )?;
            }

            TyKind::Tuple(inner_tys) => {
                render_span(
                    &template.field_tuple,
                    dest,
                    Scope::new().add_expander("Ts", TupleTypeExpander::new(inner_tys.iter())),
                    indent,
                    template,
                )?;
            }
        }

        Ok(())
    }
}
