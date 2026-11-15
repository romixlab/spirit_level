//! The pages of a project as a reader meets them: each page's headings, prose, code and visual breaks in order
//! ([`page`]), its reading metrics ([`stats`]), the words it names things with ([`terms`]), and the Zensical book
//! it belongs to ([`book`]).
//!
//! Pages are read from their Markdown sources, Zensical's admonitions and content tabs with their bodies
//! ([`dialect`]). A site built from templates holds much of a page outside its Markdown: such a site's built
//! HTML is read in place of its sources ([`site`], [`html`]), into the same page model.

pub mod book;
pub mod dialect;
pub mod html;
pub mod page;
pub mod site;
pub mod stats;
pub mod terms;
pub mod text;
pub mod walk;

pub use book::Book;
pub use page::{Kind, Page, Source};
pub use site::{Site, SiteError};
pub use stats::{ReadingSpeed, Stats};

use std::path::Path;

/// Which Markdown files are pages, and which of them are reference pages: globs relative to the project root
#[derive(Clone, Copy, Debug)]
pub struct Selection<'a> {
    /// The pages
    pub files: &'a [String],
    /// Files left out
    pub skip: &'a [String],
    /// Reference pages; every other page is a guide
    pub reference: &'a [String],
}

impl Selection<'_> {
    /// The kind of a page
    pub fn kind(&self, rel: &str) -> Kind {
        if self
            .reference
            .iter()
            .any(|g| sl_core::glob::matches(g, rel))
        {
            Kind::Reference
        } else {
            Kind::Guide
        }
    }
}

/// Every selected page that ends in `.md` and can be read, parsed, in path order; `members` are the directories of
/// the workspace members (nested cargo projects that are not members are left out)
pub fn pages(root: &Path, members: &[&str], sel: &Selection<'_>) -> Vec<Page> {
    walk::files(root, members, sel.files, sel.skip)
        .into_iter()
        .filter(|f| {
            Path::new(f)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("md"))
        })
        .filter_map(|f| {
            let text = walk::read_page(root, &f)?;
            Some(Page::parse(f.clone(), sel.kind(&f), &text))
        })
        .collect()
}
