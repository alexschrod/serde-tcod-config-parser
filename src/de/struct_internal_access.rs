use crate::de::{Deserializer, Error};
use crate::lexer::Token;
use logos::Lexer;
use serde::de::{self, IntoDeserializer};

pub struct StructInternalAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    instance_name: Option<&'de str>,
    lexer: Option<Lexer<Token, &'de str>>,
}

impl<'a, 'de> StructInternalAccess<'a, 'de> {
    pub fn new(de: &'a mut Deserializer<'de>, instance_name: &'de str) -> Self {
        Self {
            de,
            instance_name: Some(instance_name),
            lexer: None,
        }
    }
}

impl<'de: 'a, 'a> de::MapAccess<'de> for StructInternalAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(
        &mut self,
        seed: K,
    ) -> Result<Option<<K as de::DeserializeSeed<'de>>::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.instance_name.is_some() {
            return seed
                .deserialize("instance_name".into_deserializer())
                .map(Some);
        }

        if self.de.lexer.token == Token::BraceClose {
            self.de.lexer.advance();
            return Ok(None);
        }

        if self.de.lexer.token != Token::Identifier {
            return unexpected_token!(self.de.lexer, "<field>");
        }
        let field = self.de.lexer.slice();

        self.lexer = Some(self.de.lexer.clone());
        self.de.lexer.advance();

        if self.de.lexer.token == Token::Assign
            || self.de.lexer.token == Token::Text
            || self.de.lexer.token == Token::BraceOpen
            || self.de.lexer.token == Token::Identifier
            || self.de.lexer.token == Token::BraceClose
        {
            seed.deserialize(field.into_deserializer()).map(Some)
        } else {
            self.lexer = None;
            Ok(None)
        }
    }

    fn next_value_seed<V>(
        &mut self,
        seed: V,
    ) -> Result<<V as de::DeserializeSeed<'de>>::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(instance_name) = self.instance_name.take() {
            return seed.deserialize(de::value::BorrowedStrDeserializer::new(instance_name));
        }

        match self.de.lexer.token {
            Token::Assign => {
                self.lexer = None;
                self.de.lexer.advance();

                match self.de.lexer.token {
                    Token::Text
                    | Token::Char
                    | Token::Integer
                    | Token::Hex
                    | Token::Float
                    | Token::BracketOpen => seed.deserialize(&mut *self.de),
                    _ => unexpected_token!(self.de.lexer, "<value>"),
                }
            }
            Token::Text | Token::BraceOpen | Token::Identifier | Token::BraceClose => {
                self.de.lexer = self.lexer.take().unwrap();
                seed.deserialize(&mut *self.de)
            }
            _ => unexpected_token!(self.de.lexer, "= or \"<name>\""),
        }
    }
}
