use crate::items::{DataType, PrimitiveType, Program, StructDefinition, StructField};
use crate::lexer::TokenStream;
use crate::token::Token;
use std::cell::{Cell, RefCell};

pub struct Parser<L> {
    lexer: L,
    current: Token,
    next: Token,
    pub structs: Vec<StructDefinition>,
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
            structs: vec![],
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

    fn parse_ident(&mut self) -> String {
        match self.consume() {
            Token::Identifier(ident) => ident.clone(),
            _ => panic!("Expected identifier"),
        }
    }

    fn parse_type(&mut self) -> DataType {
        let name = self.parse_ident();

        match name.as_str() {
            "string" => DataType::Primitive(PrimitiveType::String),
            "int" => DataType::Primitive(PrimitiveType::Int),
            _ => DataType::UserDefined(name),
        }
    }

    fn parse_struct(&mut self) {
        let struct_name = self.parse_ident();
        self.expect(Token::BraceLeft);

        let mut struct_def = StructDefinition {
            name: struct_name,
            fields: vec![],
        };

        while !(matches!(self.next, Token::BraceRight)) {
            let field_name = self.parse_ident();
            self.expect(Token::Colon);
            let field_type = self.parse_type();

            struct_def.fields.push(StructField {
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

        self.structs.push(struct_def);
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
            structs: self.structs
        }
    }
}
