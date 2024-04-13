use std::marker::PhantomData;

use crate::items::{StructField, TyKind};

use super::expander::Expander;
use super::scope::Scope;
use super::span::{do_indent, TemplateSpan};

#[derive(Clone)]
pub struct TypeAstSpans<'t> {
    pub primitive: TemplateSpan<'t>,
    pub message: TemplateSpan<'t>,
    pub array: TemplateSpan<'t>,
    pub main: TemplateSpan<'t>,
}

struct SingleTypeAstExpander<'s, 'k, 't> {
    spanset: &'s TypeAstSpans<'t>,
    ty: &'k TyKind,
}

impl<'s, 'k, 't> Expander for SingleTypeAstExpander<'s, 'k, 't> {
    fn expand(&self, base_indent: u16) {
        match self.ty {
            TyKind::Primitive(..) => self.spanset.primitive.print(base_indent, Scope::new()),
            TyKind::UserDefined(ref name) => self
                .spanset
                .message
                .print(base_indent, Scope::new().add_text("name", &name)),
            TyKind::Array(ref inner) => self.spanset.array.print(
                base_indent,
                Scope::new().add_expander(
                    "of",
                    SingleTypeAstExpander {
                        spanset: self.spanset,
                        ty: inner.as_ref(),
                    },
                ),
            ),
            TyKind::Nullable(ref inner) => SingleTypeAstExpander {
                spanset: self.spanset,
                ty: inner.as_ref(),
            }
            .expand(base_indent),
        }
    }
}

pub struct TypeAstExpander<'t, 'f, F> {
    spanset: TypeAstSpans<'t>,
    fields: F,
    _phantom: PhantomData<&'f ()>,
}

impl<'t, 'f, F> TypeAstExpander<'t, 'f, F> {
    pub fn new(spanset: TypeAstSpans<'t>, fields: F) -> Self {
        Self {
            spanset,
            fields,
            _phantom: PhantomData,
        }
    }
}

impl<'t, 'f, F> Expander for TypeAstExpander<'t, 'f, F>
where
    F: Iterator<Item = &'f StructField> + Clone,
{
    fn expand(&self, base_indent: u16) {
        let mut is_tail = false;

        for field in self.fields.clone() {
            if is_tail {
                print!(",\n");
                do_indent(base_indent);
            } else {
                is_tail = true;
            }

            let field_ast_expander = SingleTypeAstExpander {
                spanset: &self.spanset,
                ty: &field.datatype,
            };

            self.spanset.main.print(
                base_indent,
                Scope::new()
                    .add_text("name", &field.name)
                    .add_expander("ast", field_ast_expander),
            );
        }
    }
}
