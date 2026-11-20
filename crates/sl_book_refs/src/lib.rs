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

/// The cards' stylesheet: hidden until the script opens one, colours from the theme's variables
pub const CARD_CSS: &str = include_str!("../assets/refs.css");

/// The script that opens a reference's card beside it on hover, keyboard focus and a first tap
pub const CARD_SCRIPT: &str = include_str!("../assets/refs.js");

#[cfg(test)]
mod tests {
    /// The extension's sources, which write the classes the stylesheet styles
    const EXTENSION: [&str; 3] = [
        include_str!("../../../python/spl_book_ext/src/spl_book_ext/card.py"),
        include_str!("../../../python/spl_book_ext/src/spl_book_ext/glossary.py"),
        include_str!("../../../python/spl_book_ext/src/spl_book_ext/code_refs.py"),
    ];

    #[test]
    fn the_stylesheet_styles_every_class_the_extension_writes() {
        let py = EXTENSION.concat();
        for class in [
            "spl-ref",
            "spl-card",
            "spl-card-head",
            "spl-card-src",
            "spl-card-doc",
            "spl-card-where",
            "spl-card-more",
            "spl-cards",
            "spl-gloss",
        ] {
            assert!(
                py.contains(&format!("\"{class}")),
                "the extension writes no {class}"
            );
            assert!(
                super::CARD_CSS.contains(&format!(".{class}")),
                "the stylesheet lacks .{class}"
            );
        }
        for kind in ["term", "code", "file"] {
            assert!(
                py.contains(&format!("\"{kind}\", card_id)")),
                "the extension writes no spl-{kind}"
            );
            assert!(
                super::CARD_CSS.contains(&format!(".spl-{kind}")),
                "the stylesheet lacks .spl-{kind}"
            );
        }
    }
}
