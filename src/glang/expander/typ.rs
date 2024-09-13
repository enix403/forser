use std::io::{self, Write};

use crate::items::{PrimitiveType, TyKind};

use crate::glang::emit::render_span;
use crate::glang::expander::Expander;
use crate::glang::scope::Scope;
use crate::glang::template::{ExpandOptions, Template};

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
                for inner in inner_tys {
                    render_span(
                        &template.field_map,
                        dest,
                        Scope::new().add_expander("T", TypeExpander(inner)),
                        indent,
                        template,
                    )?;
                }
            }
        }

        Ok(())
    }
}