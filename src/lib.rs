//! A best-effort implementation of the now deprecated libtcod config file parser functionality
//! from [`libtcod`].
//!
//! # Raison d'être
//! I was porting an old, abandoned C++ game and ran into a problem when its tile set definitions
//! used the [`libtcod`] config files parser. The [`tcod`] crate had't implemented a wrapper for
//! parsing these, and the parsing feature had been deprecated from [`libtcod`] anyway, so there
//! was little hope that it would ever get implemented.
//!
//! Since the feature was deprecated from [`libtcod`], I figured spending time and energy adding
//! this feature to the [`tcod`] crate by wrapping functionality that was going to disappear in
//! future versions was pointless. The format is very simple, so I figured I'd make a lexer for it
//! (using the brilliant [`logos`] crate), and then implement a [`serde`] deserializer for it, so
//! using it would basically feel the same as using any other [`serde`]-based deserializer.
//!
//! # Incompatibilities
//! Should it be required, these can probably be somewhat mitigated in the future, but for now,
//! I didn't need these features, or I couldn't be bothered to work around them.
//!
//! ## No support for dynamic declarations
//! The original format allows declaring structs and fields that don't exist in the actual type
//! declarations being deserialized. I decided I didn't need this for my own needs, and so this
//! feature is missing.
//!
//! ## No support for arbitrary order of contained structs
//! Because the original parser was event-driven, the order that things appear in the file is mostly
//! irrelevant. While serde is very powerful, there are some limitations that I decided to enforce
//! just to make my job easier. In particular, when a type has multiple inner structs, e.g.
//! ```
//! #[derive(Deserialize)]
//! #[serde(rename = "outer")]
//! struct Outer {
//!     name: String,
//!     inner1: Vec<Inner1>,
//!     inner2: Vec<Inner2>,
//! }
//!
//! #[derive(Deserialize)]
//! #[serde(rename = "inner1")]
//! struct Inner1 {
//!     name: String,
//! }
//!
//! #[derive(Deserialize)]
//! #[serde(rename = "inner2")]
//! struct Inner2 {
//!     name: String,
//! }
//! ```
//! this deserializer requires that all the instances of each inner struct is grouped together,
//! meaning that you can have
//! ```ignore
//! outer {
//!     inner1 {
//!     }
//!     inner1 {
//!     }
//!     inner2 {
//!     }
//!     inner2 {
//!     }
//! }
//! ```
//! but you cannot have
//! ```ignore
//! outer {
//!     inner1 {
//!     }
//!     inner2 {
//!     }
//!     inner1 {
//!     }
//!     inner2 {
//!     }
//! }
//! ```
//!
//! ## No support for libtcod-specific types
//!
//! The `color` and `dice` types are unsupported as of now.
//!
//! [`libtcod`]: https://github.com/libtcod/libtcod
//! [`tcod`]: https://crates.io/crates/tcod
//! [`logos`]: https://crates.io/crates/logos
//! [`serde`]: https://crates.io/crates/serde
//! [`Deserializer`]: de/struct.Deserializer.html
pub mod de;

mod lexer;
