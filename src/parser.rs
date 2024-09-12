use crate::items::{
    EnumDefinition, EnumVariant, EnumVariantValue, PrimitiveType, Program, StructDefinition,
    StructField, TyKind,
};
use crate::lexer::TokenStream;
use crate::token::{Token, TokenKind};
use std::collections::{HashMap, HashSet};

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Expected token {expected:?}, found {found:?}")]
    UnexpectedToken {
        expected: Option<TokenKind>,
        found: Token,
    },

    #[error("\"{0}\"")]
    Custom(String),

    #[error("Recursive Type \"{0}\" has infinite size")]
    RecursiveType(String),

    #[error("Unknown Type \"{0}\"")]
    UnknownType(String),

    #[error("Type \"{0}\" is not a valid user-defined type")]
    InvalidUdt(String),

    #[error("Type \"{0}\" is already defined")]
    RedefinedType(String),
}

pub struct Parser<L> {
    lexer: L,
    current: Token,
    next: Token,
    errors: Vec<ParseError>,

    // Items
    structs: HashMap<String, StructDefinition>,
    enums: HashMap<String, EnumDefinition>,

    // Validation
    /// Set of user defined types yet to be found
    pending_types: HashSet<String>,
}

mod guards {
    use super::*;

    pub fn ty_recursive(parent: &str, ty: &TyKind) -> bool {
        match ty {
            TyKind::UserDefined(udt) => parent == udt,
            TyKind::Primitive(..) | TyKind::Array(..) | TyKind::Nullable(..) | TyKind::Map(..) => {
                false
            }
        }
    }

    pub fn is_reserved(name: &str) -> bool {
        match name {
            "struct" | "string" | "int" | "float" | "bool" => false,
            _ => true,
        }
    }
}

impl<L> Parser<L>
where
    L: TokenStream,
{
    pub fn new(mut lexer: L) -> Self {
        Self {
            current: Token::init(),
            next: lexer.next_token(),
            lexer,
            errors: vec![],
            structs: HashMap::new(),
            enums: HashMap::new(),
            pending_types: HashSet::new(),
        }
    }

    fn consume(&mut self) -> &Token {
        self.current = std::mem::replace(&mut self.next, self.lexer.next_token());
        &self.current
    }

    fn syntax_error(&mut self, expected: Option<TokenKind>) {
        self.errors.push(ParseError::UnexpectedToken {
            expected,
            found: self.current.clone(),
        });
    }

    fn custom_error(&mut self, message: &str) {
        self.errors.push(ParseError::Custom(message.into()));
    }

    fn consume_expected(&mut self, expected: TokenKind) {
        let token = self.consume();
        if expected != token.kind {
            self.syntax_error(Some(expected));
        }
    }

    fn parse_ident(&mut self) -> String {
        let token = self.consume();

        match token.kind {
            TokenKind::Identifier(ref ident) => ident.clone(),
            _ => {
                // TODO: This is ugly
                self.syntax_error(Some(TokenKind::Identifier("".to_string())));
                String::new()
            }
        }
    }

    fn is_valid_udt(&self, name: &str) -> bool {
        self.structs.contains_key(name) || self.enums.contains_key(name)
    }

    fn parse_type(&mut self) -> TyKind {
        let ty = if self.next.kind == TokenKind::AngleLeft {
            self.consume();

            let ty = self.parse_type();
            self.consume_expected(TokenKind::AngleRight);

            TyKind::Map(Box::new(ty))
        } else if self.next.kind == TokenKind::SquareLeft {
            self.consume();

            let ty = self.parse_type();
            self.consume_expected(TokenKind::SquareRight);

            TyKind::Array(Box::new(ty))
        } else {
            let name = self.parse_ident();

            match name.as_str() {
                "string" => TyKind::Primitive(PrimitiveType::String),
                "int" => TyKind::Primitive(PrimitiveType::Int),
                "float" => TyKind::Primitive(PrimitiveType::Float),
                "bool" => TyKind::Primitive(PrimitiveType::Bool),
                _ => {
                    // If the referenced type is not yet defined, then add it to
                    // pending_types to be checked later
                    if !self.is_valid_udt(&name) {
                        self.pending_types.insert(name.clone());
                    }

                    TyKind::UserDefined(name)
                }
            }
        };

        if self.next.kind == TokenKind::QuestionMark {
            self.consume();
            TyKind::Nullable(Box::new(ty))
        } else {
            ty
        }
    }

    fn parse_struct(&mut self) {
        let struct_name = self.parse_ident();

        if !guards::is_reserved(&struct_name) {
            self.errors
                .push(ParseError::InvalidUdt(struct_name.clone()));
        } else if self.is_valid_udt(&struct_name) {
            self.errors
                .push(ParseError::RedefinedType(struct_name.clone()));
        }

        let mut struct_ = StructDefinition {
            name: struct_name,
            fields: vec![],
        };

        self.consume_expected(TokenKind::BraceLeft);

        while !(matches!(self.next.kind, TokenKind::BraceRight)) {
            let field_name = self.parse_ident();
            self.consume_expected(TokenKind::Colon);
            let field_type = self.parse_type();

            if guards::ty_recursive(&struct_.name, &field_type) {
                self.errors
                    .push(ParseError::RecursiveType(struct_.name.clone()));
                continue;
            }

            struct_.fields.push(StructField {
                name: field_name,
                datatype: field_type,
            });

            if matches!(self.next.kind, TokenKind::Comma) {
                self.consume();
            } else {
                break;
            }
        }

        self.consume_expected(TokenKind::BraceRight);

        self.pending_types.remove(&struct_.name);
        self.structs.insert(struct_.name.clone(), struct_);
    }

    fn parse_enum(&mut self) {
        // TODO: a lot of logic is copied from parse_struct(). Remove duplication

        let enum_name = self.parse_ident();

        if !guards::is_reserved(&enum_name) {
            self.errors.push(ParseError::InvalidUdt(enum_name.clone()));
        } else if self.is_valid_udt(&enum_name) {
            self.errors
                .push(ParseError::RedefinedType(enum_name.clone()));
        }

        let mut enum_ = EnumDefinition {
            name: enum_name,
            variants: vec![],
        };

        self.consume_expected(TokenKind::BraceLeft);

        let mut curr_int_value = 0;
        let mut is_int_enum = true;
        let mut type_decided = false;

        while !(matches!(self.next.kind, TokenKind::BraceRight)) {
            let variant_name = self.parse_ident();
            let mut variant_value: EnumVariantValue = EnumVariantValue::Int(0);

            if matches!(self.next.kind, TokenKind::Equal) {
                self.consume();

                match self.next.kind.clone() {
                    TokenKind::StringLiteral(val) => {
                        self.consume();
                        if type_decided && is_int_enum {
                            self.custom_error("Expected an int, found string");
                        } else {
                            variant_value = EnumVariantValue::String(val.clone());
                            is_int_enum = false;
                        }
                    }
                    TokenKind::IntLiteral(val) => {
                        self.consume();

                        if type_decided && !is_int_enum {
                            self.custom_error("Expected a string, found int");
                        } else {
                            variant_value = EnumVariantValue::Int(val);
                            curr_int_value = val + 1;
                        }
                    }
                    _ => {
                        self.syntax_error(None);
                    }
                }
            } else {
                if !type_decided || is_int_enum {
                    variant_value = EnumVariantValue::Int(curr_int_value);
                    curr_int_value += 1;
                } else {
                    self.custom_error("Expected a string");
                }
            }

            type_decided = true;

            enum_.variants.push(EnumVariant {
                name: variant_name,
                value: variant_value,
            });

            if matches!(self.next.kind, TokenKind::Comma) {
                self.consume();
            } else {
                break;
            }
        }

        self.consume_expected(TokenKind::BraceRight);

        self.pending_types.remove(&enum_.name);
        self.enums.insert(enum_.name.clone(), enum_);
    }

    pub fn parse(mut self) -> Result<Program, Vec<ParseError>> {
        loop {
            self.consume();
            match self.current.kind {
                TokenKind::Struct => self.parse_struct(),
                TokenKind::Enum => self.parse_enum(),
                TokenKind::Eof => break,
                _ => self.syntax_error(None),
            }
        }

        self.errors.extend(
            self.pending_types
                .into_iter()
                .map(|ty| ParseError::UnknownType(ty)),
        );

        if self.errors.len() > 0 {
            Err(self.errors)
        } else {
            Ok(Program {
                structs: self.structs.into_values().collect(),
                enums: self.enums.into_values().collect(),
            })
        }
    }
}
