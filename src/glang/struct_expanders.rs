use std::marker::PhantomData;

use crate::items::{PrimitiveType, StructField, TyKind};

use super::expander::Expander;
use super::scope::Scope;
use super::span::{do_indent, TemplateSpan};

#[derive(Clone)]
pub struct TypeAstSpans<'s> {
    pub primitive: TemplateSpan<'s>,
    pub message: TemplateSpan<'s>,
    pub array: TemplateSpan<'s>,
    pub main: TemplateSpan<'s>,
}

struct SingleTypeAstExpander<'s> {
    spanset: &'s TypeAstSpans<'s>,
    ty: &'s TyKind,
}

impl<'s> Expander for SingleTypeAstExpander<'s> {
    fn count(&self) -> usize {
        1
    }

    fn expand_next(&mut self, base_indent: u16) {
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
            TyKind::Nullable(ref inner) => {
                SingleTypeAstExpander {
                    spanset: self.spanset,
                    ty: inner.as_ref(),
                }
                .expand_next(base_indent);
            }
        }
    }
}

pub struct TypeAstExpander<'s, F> {
    spanset: &'s TypeAstSpans<'s>,
    fields: F,
}

impl<'s, F> TypeAstExpander<'s, F> {
    pub fn new(spanset: &'s TypeAstSpans<'s>, fields: F) -> Self {
        Self { spanset, fields }
    }
}

impl<'s, F> Expander for TypeAstExpander<'s, F>
where
    F: Iterator<Item = &'s StructField> + Clone,
{
    fn count(&self) -> usize {
        self.fields.clone().count()
    }

    fn expand_next(&mut self, base_indent: u16) {
        if let Some(field) = self.fields.next() {
            let field_ast_expander = SingleTypeAstExpander {
                spanset: self.spanset,
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

/* =============================================================================================== */

pub struct FieldTypeSpans<'s> {
    pub string: TemplateSpan<'s>,
    pub int: TemplateSpan<'s>,
    pub float: TemplateSpan<'s>,
    pub bool_: TemplateSpan<'s>,
    pub array: TemplateSpan<'s>,
    pub null: TemplateSpan<'s>,
    pub struct_: TemplateSpan<'s>,
}

pub struct FieldTypeExpander<'s> {
    spanset: &'s FieldTypeSpans<'s>,
    ty: &'s TyKind,
}

impl<'s> Expander for FieldTypeExpander<'s> {
    fn count(&self) -> usize {
        1
    }

    fn expand_next(&mut self, base_indent: u16) {
        match self.ty {
            TyKind::Primitive(prim) => match prim {
                PrimitiveType::String => self.spanset.string.print(base_indent, Scope::new()),
                PrimitiveType::Int => self.spanset.int.print(base_indent, Scope::new()),
                PrimitiveType::Float => self.spanset.float.print(base_indent, Scope::new()),
                PrimitiveType::Bool => self.spanset.bool_.print(base_indent, Scope::new()),
            },

            TyKind::UserDefined(name) => self
                .spanset
                .struct_
                .print(base_indent, Scope::new().add_text("T", &name)),

            TyKind::Nullable(inner) => self.spanset.struct_.print(
                base_indent,
                Scope::new().add_expander(
                    "T",
                    FieldTypeExpander {
                        spanset: self.spanset,
                        ty: inner.as_ref(),
                    },
                ),
            ),

            TyKind::Array(inner) => self.spanset.array.print(
                base_indent,
                Scope::new().add_expander(
                    "T",
                    FieldTypeExpander {
                        spanset: self.spanset,
                        ty: inner.as_ref(),
                    },
                ),
            ),
        }
    }
}

pub struct FieldsExpander<'s, F> {
    fields: F,
    spanset: &'s FieldTypeSpans<'s>,
    field_body_span: &'s TemplateSpan<'s>,
}

impl<'s, F> FieldsExpander<'s, F> {
    pub fn new(
        fields: F,
        spanset: &'s FieldTypeSpans<'s>,
        field_body_span: &'s TemplateSpan<'s>,
    ) -> Self {
        Self {
            spanset,
            fields,
            field_body_span,
        }
    }
}

impl<'s, F> Expander for FieldsExpander<'s, F>
where
    F: Iterator<Item = &'s StructField> + Clone,
{
    fn count(&self) -> usize {
        self.fields.clone().count()
    }

    fn expand_next(&mut self, base_indent: u16) {
        if let Some(field) = self.fields.next() {
            let field_type_expander = FieldTypeExpander {
                spanset: self.spanset,
                ty: &field.datatype,
            };

            self.field_body_span.print(
                base_indent,
                Scope::new()
                    .add_text("name", &field.name)
                    .add_expander("ty", field_type_expander),
            );
        }
    }
}
