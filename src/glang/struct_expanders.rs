use std::io::{self, Write};
use std::marker::PhantomData;

use crate::items::{PrimitiveType, StructField, TyKind};

use super::expander::Expander;
use super::scope::Scope;
use super::span::TemplateSpan;

#[derive(Clone)]
pub struct TypeAstSpans<'s> {
    pub primitive: TemplateSpan<'s>,
    pub message: TemplateSpan<'s>,
    pub array: TemplateSpan<'s>,
    pub main: TemplateSpan<'s>,
}

struct SingleTypeAstExpander<'s, W> {
    spanset: &'s TypeAstSpans<'s>,
    ty: &'s TyKind,
    dest: &'s mut W,
}

impl<'s, W: Write> Expander for SingleTypeAstExpander<'s, W> {
    fn count(&self) -> usize {
        1
    }

    fn expand_next(&mut self, base_indent: u16) -> io::Result<()> {
        match self.ty {
            TyKind::Primitive(..) => {
                self.spanset
                    .primitive
                    .print(self.dest, base_indent, Scope::new())?
            }
            TyKind::UserDefined(ref name) => self.spanset.message.print(
                self.dest,
                base_indent,
                Scope::new().add_text("name", &name),
            )?,
            TyKind::Array(ref inner) => self.spanset.array.print(
                self.dest,
                base_indent,
                Scope::new().add_expander(
                    "of",
                    SingleTypeAstExpander {
                        spanset: self.spanset,
                        ty: inner.as_ref(),
                        dest: &mut *self.dest,
                    },
                ),
            )?,
            TyKind::Nullable(ref inner) => {
                SingleTypeAstExpander {
                    spanset: self.spanset,
                    ty: inner.as_ref(),
                    dest: &mut *self.dest,
                }
                .expand_next(base_indent)?;
            }
        };

        Ok(())
    }
}

pub struct TypeAstExpander<'s, F, W> {
    spanset: &'s TypeAstSpans<'s>,
    fields: F,
    dest: &'s mut W,
}

impl<'s, F, W> TypeAstExpander<'s, F, W> {
    pub fn new(spanset: &'s TypeAstSpans<'s>, fields: F, dest: &'s mut W) -> Self {
        Self {
            spanset,
            fields,
            dest,
        }
    }
}

impl<'s, F, W> Expander for TypeAstExpander<'s, F, W>
where
    F: Iterator<Item = &'s StructField> + Clone,
    W: Write,
{
    fn count(&self) -> usize {
        self.fields.clone().count()
    }

    fn expand_next(&mut self, base_indent: u16) -> io::Result<()> {
        if let Some(field) = self.fields.next() {
            let field_ast_expander = SingleTypeAstExpander {
                spanset: self.spanset,
                ty: &field.datatype,
                dest: self.dest,
            };

            self.spanset.main.print(
                self.dest,
                base_indent,
                Scope::new()
                    .add_text("name", &field.name)
                    .add_expander("ast", field_ast_expander),
            )?;
        }

        Ok(())
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

pub struct FieldTypeExpander<'s, W> {
    spanset: &'s FieldTypeSpans<'s>,
    ty: &'s TyKind,
    dest: &'s mut W,
}

impl<'s, W: Write> Expander for FieldTypeExpander<'s, W> {
    fn count(&self) -> usize {
        1
    }

    fn expand_next(&mut self, base_indent: u16) -> io::Result<()> {
        match self.ty {
            TyKind::Primitive(prim) => match prim {
                PrimitiveType::String => {
                    self.spanset
                        .string
                        .print(self.dest, base_indent, Scope::new())?
                }
                PrimitiveType::Int => {
                    self.spanset
                        .int
                        .print(self.dest, base_indent, Scope::new())?
                }
                PrimitiveType::Float => {
                    self.spanset
                        .float
                        .print(self.dest, base_indent, Scope::new())?
                }
                PrimitiveType::Bool => {
                    self.spanset
                        .bool_
                        .print(self.dest, base_indent, Scope::new())?
                }
            },

            TyKind::UserDefined(name) => self.spanset.struct_.print(
                self.dest,
                base_indent,
                Scope::new().add_text("T", &name),
            )?,

            TyKind::Nullable(inner) => self.spanset.struct_.print(
                self.dest,
                base_indent,
                Scope::new().add_expander(
                    "T",
                    FieldTypeExpander {
                        spanset: self.spanset,
                        ty: inner.as_ref(),
                        dest: unimplemented!() as &mut std::fs::File,
                    },
                ),
            )?,

            TyKind::Array(inner) => self.spanset.array.print(
                self.dest,
                base_indent,
                Scope::new().add_expander(
                    "T",
                    FieldTypeExpander {
                        spanset: self.spanset,
                        ty: inner.as_ref(),
                        dest: unimplemented!() as &mut std::fs::File,
                    },
                ),
            )?,
        }

        Ok(())
    }
}

pub struct FieldsExpander<'s, F, W> {
    fields: F,
    spanset: &'s FieldTypeSpans<'s>,
    field_body_span: &'s TemplateSpan<'s>,
    dest: &'s mut W,
}

impl<'s, F, W> FieldsExpander<'s, F, W> {
    pub fn new(
        fields: F,
        spanset: &'s FieldTypeSpans<'s>,
        field_body_span: &'s TemplateSpan<'s>,
        dest: &'s mut W
    ) -> Self {
        Self {
            spanset,
            fields,
            field_body_span,
            dest
        }
    }
}

impl<'s, F, W: Write> Expander for FieldsExpander<'s, F, W>
where
    F: Iterator<Item = &'s StructField> + Clone,
{
    fn count(&self) -> usize {
        self.fields.clone().count()
    }

    fn expand_next(&mut self, base_indent: u16) -> io::Result<()> {
        if let Some(field) = self.fields.next() {
            let field_type_expander = FieldTypeExpander {
                spanset: self.spanset,
                ty: &field.datatype,
                dest: self.dest,
            };

            self.field_body_span.print(
                self.dest,
                base_indent,
                Scope::new()
                    .add_text("name", &field.name)
                    .add_expander("ty", field_type_expander),
            )?;
        }

        Ok(())
    }
}
