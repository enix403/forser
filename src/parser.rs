use crate::items::{PrimitiveType, Program, StructDefinition, StructField, TyKind};
use crate::lexer::TokenStream;
use crate::token::{Token, TokenKind};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        expected: Option<TokenKind>,
        found: Token,
    },
}

pub struct Parser<L> {
    lexer: L,
    current: Token,
    next: Token,
    is_error: bool,
    errors: Vec<ParseError>,
    structs: HashMap<String, StructDefinition>,
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
            is_error: false,
            errors: vec![],
            structs: HashMap::new(),
        }
    }

    fn consume(&mut self) -> &Token {
        self.current = std::mem::replace(&mut self.next, self.lexer.next_token());
        &self.current
    }

    fn record_error(&mut self, expected: Option<TokenKind>) {
        self.is_error = true;
        self.errors.push(ParseError::UnexpectedToken {
            expected,
            found: self.current.clone(),
        });
    }

    fn consume_expected(&mut self, expected: TokenKind) {
        let token = self.consume();
        if expected != token.kind {
            self.record_error(Some(expected));
        }
    }

    fn parse_ident(&mut self) -> String {
        let token = self.consume();

        match token.kind {
            TokenKind::Identifier(ref ident) => ident.clone(),
            _ => {
                self.record_error(Some(TokenKind::Identifier("".to_string())));
                String::new()
            }
        }
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
                _ => TyKind::UserDefined(name),
            }
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

            struct_.fields.push(StructField {
                name: field_name,
                datatype: field_type,
            });

            if matches!(self.next.kind, TokenKind::Comma) {
                self.consume();
                continue;
            }

            self.consume_expected(TokenKind::BraceRight);
            break;
        }

        self.structs.insert(struct_.name.clone(), struct_);
    }

    pub fn parse(mut self) -> Result<Program, Vec<ParseError>> {
        loop {
            match self.consume().kind {
                TokenKind::Struct => self.parse_struct(),
                TokenKind::Eof => break,
                _ => (),
            }
        }

        if self.is_error {
            Err(self.errors)
        } else {
            Ok(Program {
                structs: self.structs.into_values().collect(),
            })
        }
    }
}
