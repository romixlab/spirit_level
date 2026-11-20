//! A book's references in prose: glossary terms and code items, each a link with a card shown on hover and focus.
//!
//! - `[[piece]]` names a term of the book's glossary ([`glossary`]): the card shows its short definition, the link
//!   leads to its entry on the glossary page.
//! - `` [`TapeWriter::resume_on`] `` names a public item of the book's crates, as rustdoc's intra-doc links do
//!   ([`code_index`]): the card shows its signature and first doc paragraph, the link opens its lines on the forge
//!   at the commit the book is built from.
//!
//! The Markdown extension that renders both is the Python package `spl_book_ext` (Zensical loads Python-Markdown
//! extensions). Its settings in `zensical.toml` are read here too ([`Settings`]), so `spl book refs` and the
//! `glossary-terms` lint find the same files the build does. The card's stylesheet and script are [`CARD_CSS`] and
//! [`CARD_SCRIPT`], written into the book by `spl book refs`.

pub mod code_index;
pub mod glossary;
mod settings;

pub use settings::{CODE_REFS_EXTENSION, CodeRefs, GLOSSARY_EXTENSION, GlossaryRefs, Settings};

/// The cards' stylesheet: hidden until the reference is hovered or focused, colours from the theme's variables
pub const CARD_CSS: &str = include_str!("../assets/refs.css");

/// The script that keeps an open card inside the window
pub const CARD_SCRIPT: &str = include_str!("../assets/refs.js");
