use crate::lexer::Token;
use logos::Lexer;
use paste;
use serde::de::Error as DeError;
use serde::de::{self, Visitor};
use serde::forward_to_deserialize_any;
use snafu::{ResultExt, Snafu};
use std::fmt::Display;
use std::ops::Range;

#[macro_use]
mod macros {
    macro_rules! unexpected_token {
        ($l: expr, $e: expr) => {
            Err(Error::UnexpectedToken {
                token: $l.token,
                value: $l.slice().to_string(),
                range: $l.range(),
                expected: $e,
            })
        };
    }

    macro_rules! visit_number {
        ($l: expr, $to: ident, $ty: ident) => {
            if $l.token == Token::$to {
                paste::expr! {
                    let result = $l.slice().parse().unwrap();
                    $l.advance();
                    visitor.[<visit_$ty>](result)
                }
            } else {
                unexpected_token!($l, "<number>")
            }
        };
    }
}

mod struct_internal_access;
use struct_internal_access::*;

mod struct_sequence_access;
use struct_sequence_access::*;

mod primitive_sequence_access;
use primitive_sequence_access::*;

#[derive(Debug, Snafu)]
pub enum Error {
    Io {
        source: std::io::Error,
    },
    Serde {
        msg: String,
    },
    UnexpectedToken {
        token: Token,
        value: String,
        range: Range<usize>,
        expected: &'static str,
    },
    #[snafu(display("Found struct {}, expected struct {}", name, expected))]
    UnexpectedStruct {
        name: String,
        expected: String,
    },
    #[snafu(display("TCOD parser structs must have a 'name' field"))]
    MissingName,
    InvalidChar {
        source: InvalidCharError,
    },
    #[snafu(display("multi-line string is not supported for borrowed str fields"))]
    MultiLineStringOnBorrowedStr {
        token: Token,
        value: String,
        range: Range<usize>,
    },
}

#[derive(Debug, Snafu)]
pub enum InvalidCharError {
    ParseInt { source: std::num::ParseIntError },
    InvalidEscapeSequence { value: String },
    InvalidCharValue { value: String },
}

impl DeError for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Serde {
            msg: format!("{}", msg),
        }
    }
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

/// A convenience method for deserializing a type from a string.
pub fn from_str<'de, T: de::Deserialize<'de>>(s: &'de str) -> Result<T, Error> {
    T::deserialize(&mut Deserializer::new_from_str(s))
}

pub struct Deserializer<'de> {
    lexer: Lexer<Token, &'de str>,
}

impl<'de> Deserializer<'de> {
    pub fn new_from_str(source: &'de str) -> Self {
        use logos::Logos;

        let lexer = Token::lexer(source);
        Self { lexer }
    }
}

impl<'de: 'a, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bytes
        byte_buf
        unit
        unit_struct
        newtype_struct
        tuple
        tuple_struct
        map
        enum
        identifier
        ignored_any
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("not supported")
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.lexer.token == Token::Identifier {
            self.lexer.advance();
            visitor.visit_bool(true)
        } else {
            unexpected_token!(self.lexer, "<identifier>")
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, i64)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, u8)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, u16)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, u32)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, u64)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Float, f32)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Float, f64)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.lexer.token == Token::Integer {
            // Decimal notation
            let result = self
                .lexer
                .slice()
                .parse::<u8>()
                .context(ParseInt)
                .context(InvalidChar)? as char;
            self.lexer.advance();
            visitor.visit_char(result)
        } else if self.lexer.token == Token::Hex {
            // Hexadecimal notation
            let result = u8::from_str_radix(&self.lexer.slice()[2..], 16)
                .context(ParseInt)
                .context(InvalidChar)? as char;
            self.lexer.advance();
            visitor.visit_char(result)
        } else if self.lexer.token == Token::Char {
            let result = self.lexer.slice();
            let result = &result[1..][..result.len() - 2];
            let chars = result.chars().collect::<Vec<_>>();
            let octal = chars.len() > 1 && chars.iter().skip(1).all(|c| c.is_digit(8));

            let result = match result {
                c if c.starts_with("\\x") => {
                    // Hexadecimal notation
                    let c = &c[2..];
                    u8::from_str_radix(c, 16)
                        .context(ParseInt)
                        .context(InvalidChar)? as char
                }
                c if octal => {
                    // Octal notation
                    let c = &c[1..];
                    u8::from_str_radix(c, 8)
                        .context(ParseInt)
                        .context(InvalidChar)? as char
                }
                c if c.starts_with('\\') && c.len() == 2 => {
                    // Special characters
                    match &c[1..] {
                        "n" => '\n',
                        "t" => '\t',
                        "r" => '\r',
                        "\\" => '\\',
                        "\"" => '"',
                        "'" => '\'',
                        s => {
                            return Err(InvalidCharError::InvalidEscapeSequence {
                                value: s.to_string(),
                            })
                            .context(InvalidChar)
                        }
                    }
                }
                c => c.parse().unwrap(),
            };

            self.lexer.advance();
            visitor.visit_char(result)
        } else {
            unexpected_token!(self.lexer, "\"<char>\"")
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.lexer.token == Token::Text {
            let result = self.lexer.slice();
            let result = &result[1..][..result.len() - 2];
            self.lexer.advance();

            if self.lexer.token == Token::Text {
                return Err(Error::MultiLineStringOnBorrowedStr {
                    token: self.lexer.token,
                    value: self.lexer.slice().to_string(),
                    range: self.lexer.range(),
                });
            }

            visitor.visit_borrowed_str(result)
        } else {
            unexpected_token!(self.lexer, "\"<string>\"")
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.lexer.token != Token::Text {
            return unexpected_token!(self.lexer, "\"<string>\"");
        }

        let mut result = String::new();
        while self.lexer.token == Token::Text {
            let slice = self.lexer.slice();
            let slice = &slice[1..][..slice.len() - 2];
            result.push_str(slice);
            self.lexer.advance();
        }
        visitor.visit_string(result)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.lexer.token == Token::Identifier {
            visitor.visit_seq(StructSeqAccess::new(&mut self))
        } else if self.lexer.token == Token::BracketOpen {
            self.lexer.advance();
            let result = visitor.visit_seq(PrimitiveSeqAccess::new(&mut self))?;

            if self.lexer.token != Token::BracketClose {
                return unexpected_token!(self.lexer, "]");
            }
            self.lexer.advance();

            Ok(result)
        } else {
            unexpected_token!(self.lexer, "[ or identifier")
        }
    }

    fn deserialize_struct<V>(
        mut self,
        type_name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if !fields.contains(&"name") {
            return Err(Error::MissingName);
        }

        if self.lexer.token != Token::Identifier {
            return unexpected_token!(self.lexer, "<typename>");
        }

        let lex_type_name = self.lexer.slice();
        if lex_type_name != type_name {
            return Err(Error::UnexpectedStruct {
                name: lex_type_name.to_string(),
                expected: type_name.to_string(),
            });
        }

        self.lexer.advance();

        let mut lex_name = None;
        match self.lexer.token {
            Token::Text => {
                let lex_name_bit = self.lexer.slice();
                lex_name = Some(&lex_name_bit[1..][..lex_name_bit.len() - 2]);

                self.lexer.advance();
            }
            Token::BraceOpen => {}
            _ => {
                return unexpected_token!(self.lexer, "\"<name>\" or {");
            }
        }

        if self.lexer.token != Token::BraceOpen {
            return unexpected_token!(self.lexer, "{");
        }

        self.lexer.advance();

        visitor.visit_map(StructInternalAccess::new(&mut self, lex_name.unwrap_or("")))
    }
}
