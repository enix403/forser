use crate::items::{PrimitiveType, Program, StructDefinition, StructField, TyKind};
use crate::lexer::TokenStream;
use crate::token::Token;
use std::collections::HashMap;

pub struct Parser<L> {
    lexer: L,
    current: Token,
    next: Token,
    structs: HashMap<String, StructDefinition>,
}

impl<L> Parser<L>
where
    L: TokenStream,
{
    pub fn new(mut lexer: L) -> Self {
        Self {
            current: Token::Init,
            next: lexer.next_token(),
            lexer,
            structs: HashMap::new(),
        }
    }

    fn consume(&mut self) -> &Token {
        self.current = std::mem::replace(&mut self.next, self.lexer.next_token());
        &self.current
    }

    fn expect(&mut self, expected: Token) {
        let token = self.consume();
        if expected != *token {
            panic!("Expected token {:?}, found {:?}", expected, token);
        }
    }

    fn parse_ident(&mut self) -> &'_ str {
        match self.consume() {
            Token::Identifier(ident) => ident.as_str(),
            _ => panic!("Expected identifier"),
        }
    }

    fn parse_type(&mut self) -> TyKind {
        if self.next == Token::SquareLeft {
            self.consume();
            let ty = TyKind::Array(Box::new(self.parse_type()));
            self.expect(Token::SquareRight);
            ty
        } else {
            let name = self.parse_ident();

            match name {
                "string" => TyKind::Primitive(PrimitiveType::String),
                "int" => TyKind::Primitive(PrimitiveType::Int),
                _ => TyKind::UserDefined(name.into()),
            }
        }
    }

    fn parse_struct(&mut self) {
        let struct_name = self.parse_ident().to_string();
        self.expect(Token::BraceLeft);

        let mut struct_ = StructDefinition {
            name: struct_name,
            fields: vec![],
        };

        while !(matches!(self.next, Token::BraceRight)) {
            let field_name = self.parse_ident().to_string();
            self.expect(Token::Colon);
            let field_type = self.parse_type();

            struct_.fields.push(StructField {
                name: field_name,
                datatype: field_type,
            });

            if matches!(self.next, Token::Comma) {
                self.consume();
            } else if matches!(self.next, Token::BraceRight) {
                // do noting
            } else {
                panic!("Invalid syntax");
            }
        }

        self.expect(Token::BraceRight);

        // self.structs.push(struct_def);
        self.structs.insert(struct_.name.clone(), struct_);
    }

    pub fn parse(mut self) -> Program {
        loop {
            match self.consume() {
                Token::Struct => self.parse_struct(),
                Token::Eof => break,
                _ => (),
            }
        }

        Program {
            structs: self.structs.into_values().collect(),
        }
    }
}
