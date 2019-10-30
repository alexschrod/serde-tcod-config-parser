use crate::de::{Deserializer, Error};
use crate::lexer::Token;
use serde::de;

pub struct PrimitiveSeqAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> PrimitiveSeqAccess<'a, 'de> {
    pub fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de: 'a, 'a> de::SeqAccess<'de> for PrimitiveSeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> Result<Option<<T as de::DeserializeSeed<'de>>::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.de.lexer.token {
            Token::Text | Token::Integer | Token::Float | Token::Char | Token::BracketOpen => {
                let result = seed.deserialize(&mut *self.de).map(Some);
                if result.is_err(){
                    return result;
                }

                if self.de.lexer.token != Token::Comma && self.de.lexer.token != Token::BracketClose
                {
                    return unexpected_token!(self.de.lexer, "<value> or ]");
                } else if self.de.lexer.token == Token::Comma {
                    self.de.lexer.advance();
                }

                result
            }
            Token::BracketClose => Ok(None),
            _ => unexpected_token!(self.de.lexer, "<value> or ]"),
        }
    }
}
