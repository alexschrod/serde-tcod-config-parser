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
                value: $l.slice().to_string(),
                token_type: format!("{:?}", $l.token),
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

/// This type represents all possible errors that can occur when deserializing libtcod config files.
#[derive(Debug, Snafu)]
pub enum Error {
    /// An error reported to us by `serde` itself.
    #[snafu(display("An error was reported by serde: {}", msg))]
    Serde {
        /// The message `serde` provided.
        msg: String,
    },
    /// A token that was unexpected was encountered.
    #[snafu(display(
        "Encountered token \"{}\" ({}) at position {:?}. Expected {}.",
        value,
        token_type,
        range,
        expected
    ))]
    UnexpectedToken {
        /// The value of the unexpected token.
        value: String,
        /// The type of token we (thought we) found
        token_type: String,
        /// The location in the source string where the token was encountered.
        range: Range<usize>,
        /// The token/value the deserializer was expecting at this location.
        expected: &'static str,
    },
    /// A different struct than was expected was encountered.
    #[snafu(display("Found struct {}, expected struct {}", name, expected))]
    UnexpectedStruct {
        /// The name of the encountered struct.
        name: String,
        /// The expected name of the struct.
        expected: String,
    },
    /// All structs must have an `instance_name` field. This field is used to hold the value within
    /// `libtcod_struct_name "libtcod_instance_name" { ... }`. Structs without an instance name will
    /// have their value set to `""`.
    #[snafu(display("libtcod config structs must have an 'instance_name' field"))]
    MissingInstanceName,
    /// An invalid `char` representation was encountered.
    InvalidChar {
        /// The cause of the invalid char.
        source: InvalidCharError,
    },
    /// This format supports multi-line strings, but they are not necessarily contiguous, so if such
    /// an non-contiguous variant is encountered on a string slice field, this error is returned.
    #[snafu(display("multi-line string is not supported for borrowed str fields"))]
    MultiLineStringOnBorrowedStr {
        /// The value of the token where this error was triggered.
        value: String,
        /// The location in the source string where the token was encountered.
        range: Range<usize>,
    },
}

/// This type represents all possible errors that can occur when deserializing the libtcod
/// config file char type.
#[derive(Debug, Snafu)]
pub enum InvalidCharError {
    /// A char represented as an integer could not be parsed.
    ParseInt { source: std::num::ParseIntError },
    /// An invalid escape sequence was used.
    InvalidEscapeSequence { value: String },
    /// Something not representable as a char was given.
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

/// A re-declaration of `Result` that sets sensible defaults for `T` and `E`
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

/// A structure that deserializes libtcod config file values into Rust values.
pub struct Deserializer<'de> {
    lexer: Lexer<Token, &'de str>,
}

impl<'de> Deserializer<'de> {
    /// Create a libtcod config file deserializer from a `&str`.
    ///
    /// Typically it is more convenient to use the [`Deserializer::from_str`] function instead
    ///
    /// [`Deserializer::from_str`]: #method.from_str
    pub fn new(source: &'de str) -> Self {
        use logos::Logos;

        let lexer = Token::lexer(source);
        Self { lexer }
    }

    /// Creates a libtcod config file deserializer from a `&str`.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str<T: de::Deserialize<'de>>(s: &'de str) -> Result<T> {
        T::deserialize(&mut Deserializer::new(s))
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
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("not supported")
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
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

    fn deserialize_i8<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, i64)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, u8)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, u16)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, u32)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Integer, u64)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Float, f32)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visit_number!(self.lexer, Float, f64)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
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

    fn deserialize_str<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        if self.lexer.token == Token::Text {
            let result = self.lexer.slice();
            let result = &result[1..][..result.len() - 2];
            self.lexer.advance();

            if self.lexer.token == Token::Text {
                return Err(Error::MultiLineStringOnBorrowedStr {
                    value: self.lexer.slice().to_string(),
                    range: self.lexer.range(),
                });
            }

            visitor.visit_borrowed_str(result)
        } else {
            unexpected_token!(self.lexer, "\"<string>\"")
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
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

    fn deserialize_option<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
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
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        if !fields.contains(&"instance_name") {
            return Err(Error::MissingInstanceName);
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
                return unexpected_token!(self.lexer, "\"<instance_name>\" or {");
            }
        }

        if self.lexer.token != Token::BraceOpen {
            return unexpected_token!(self.lexer, "{");
        }

        self.lexer.advance();

        visitor.visit_map(StructInternalAccess::new(&mut self, lex_name.unwrap_or("")))
    }

    fn deserialize_ignored_any<V>(mut self, visitor: V) -> Result<<V as Visitor<'de>>::Value, Self::Error> where
        V: Visitor<'de> {
        unimplemented!("Ignoring items currently not supported.")
    }
}
