use crate::de::{Deserializer, Error};
use crate::lexer::Token;
use serde::de;

pub struct StructSeqAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    type_name: Option<&'de str>,
}

impl<'a, 'de> StructSeqAccess<'a, 'de> {
    pub fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self {
            de,
            type_name: None,
        }
    }
}

impl<'de: 'a, 'a> de::SeqAccess<'de> for StructSeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> Result<Option<<T as de::DeserializeSeed<'de>>::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.de.lexer.token == Token::Identifier {
            if let Some(type_name) = self.type_name {
                if type_name != self.de.lexer.slice() {
                    return Ok(None);
                }
            } else {
                self.type_name = Some(self.de.lexer.slice());
            }
            seed.deserialize(&mut *self.de).map(Some)
        } else if self.de.lexer.token == Token::BraceClose {
            Ok(None)
        } else {
            unexpected_token!(self.de.lexer, "<type> <typename> or }")
        }
    }
}
