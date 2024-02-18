use crate::items::{PrimitiveType, Program, StructDefinition, StructField, TyKind};
use crate::lexer::TokenStream;
use crate::token::{Token, TokenKind};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        expected: Option<TokenKind>,
        found: Token,
    },
    RecursiveType(String),
    UnknownType(String),
}

pub struct Parser<L> {
    lexer: L,
    current: Token,
    next: Token,
    errors: Vec<ParseError>,

    // Items
    structs: HashMap<String, StructDefinition>,

    // Validation
    /// Set of user defined types yet to be found
    pending_types: HashSet<String>,
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
        return self.structs.contains_key(name);
    }

    fn parse_type(&mut self) -> TyKind {
        if self.next.kind == TokenKind::SquareLeft {
            self.consume();
            let ty = TyKind::Array(Box::new(self.parse_type()));
            self.consume_expected(TokenKind::SquareRight);
            ty
        } else {
            let name = self.parse_ident();

            match name.as_str() {
                "string" => TyKind::Primitive(PrimitiveType::String),
                "int" => TyKind::Primitive(PrimitiveType::Int),
                _ => {
                    // If the referenced type is not yet defined, then add it to
                    // pending_types to be checked later
                    if !self.is_valid_udt(&name) {
                        self.pending_types.insert(name.clone());
                    }

                    TyKind::UserDefined(name)
                }
            }
        }
    }

    fn is_ty_recursive(parent: &str, ty: &TyKind) -> bool {
        match ty {
            TyKind::UserDefined(udt) => udt == parent,
            TyKind::Primitive(..) | TyKind::Array(..) => false,
        }
    }

    fn parse_struct(&mut self) {
        let struct_name = self.parse_ident();
        self.consume_expected(TokenKind::BraceLeft);

        let mut struct_ = StructDefinition {
            name: struct_name,
            fields: vec![],
        };

        while !(matches!(self.next.kind, TokenKind::BraceRight)) {
            let field_name = self.parse_ident();
            self.consume_expected(TokenKind::Colon);
            let field_type = self.parse_type();

            if Self::is_ty_recursive(&struct_.name, &field_type) {
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

    pub fn parse(mut self) -> Result<Program, Vec<ParseError>> {
        loop {
            self.consume();
            match self.current.kind {
                TokenKind::Struct => self.parse_struct(),
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
            })
        }
    }
}
