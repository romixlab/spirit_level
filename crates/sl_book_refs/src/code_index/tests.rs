use super::*;
use std::path::Path;

const FORGE: Forge<'static> = Forge {
    repository: "https://github.com/o/r/",
    commit: "abc123",
};

fn project() -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/projects/refs");
    Project::load(&dir).expect("the fixture project loads")
}

fn find<'a>(index: &'a Index, path: &str) -> &'a Entry {
    index
        .items
        .iter()
        .find(|e| e.path == path)
        .unwrap_or_else(|| {
            panic!(
                "{path} in {:#?}",
                index.items.iter().map(|e| &e.path).collect::<Vec<_>>()
            )
        })
}

#[test]
fn walks_the_library_module_tree() {
    let index = index(&project(), &[], FORGE).unwrap();
    let paths: Vec<&str> = index.items.iter().map(|e| e.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "tape::PIECE_BYTES_MAX",
            "tape::hello",
            "tape::hidden::reachable",
            "tape::inline::deep",
            "tape::moved::Sink",
            "tape::writer::Outcome",
            "tape::writer::Outcome::Done",
            "tape::writer::Outcome::Partial",
            "tape::writer::TapeWriter",
            "tape::writer::TapeWriter::written",
            "tape::writer::resume::TapeWriter::resume_on",
        ]
    );
    assert_eq!(index.format, FORMAT);
    assert_eq!(index.commit, "abc123");
}

#[test]
fn an_entry_has_its_lines_signature_docs_and_address() {
    let index = index(&project(), &["tape".to_owned()], FORGE).unwrap();
    let c = find(&index, "tape::PIECE_BYTES_MAX");
    assert_eq!(c.kind, Kind::Const);
    assert_eq!(c.file, "src/lib.rs");
    assert_eq!(c.lines, [14, 14]);
    assert_eq!(c.source, "pub const PIECE_BYTES_MAX: usize = 4096;");
    assert_eq!(c.doc, "The largest piece.");
    assert_eq!(
        c.url,
        "https://github.com/o/r/blob/abc123/src/lib.rs#L14-L14"
    );

    let r = find(&index, "tape::writer::resume::TapeWriter::resume_on");
    assert_eq!(r.kind, Kind::Fn);
    assert_eq!(r.lines, [8, 23]);
    assert_eq!(r.source, "pub fn resume_on(self, at: u64) -> Self");
    assert_eq!(r.doc, "Reopen the unfinished piece and go on writing it.");

    let s = find(&index, "tape::writer::TapeWriter");
    assert_eq!(
        s.source,
        "pub struct TapeWriter {\n    /// Bytes written so far.\n    pub written: u64,\n    kept: u64,\n}"
    );
}

#[test]
fn an_unknown_crate_names_the_libraries() {
    let e = index(&project(), &["tape_core".to_owned()], FORGE).unwrap_err();
    assert_eq!(
        e.to_string(),
        "`tape_core` is no library of the workspace; the libraries are tape"
    );
}

#[test]
fn source_cuts_a_long_body() {
    let lines: Vec<&str> = "    pub fn f(\n        a: u8,\n    ) {\n        x\n    }"
        .lines()
        .collect();
    assert_eq!(
        source(&lines, 1, 5, Some(3)),
        "pub fn f(\n    a: u8,\n) {\n    x\n}"
    );
    let long: Vec<String> = std::iter::once("pub fn f() {".to_owned())
        .chain((0..20).map(|i| format!("    {i};")))
        .chain(std::iter::once("}".to_owned()))
        .collect();
    let long: Vec<&str> = long.iter().map(String::as_str).collect();
    assert_eq!(source(&long, 1, 22, Some(1)), "pub fn f()");
}

#[test]
fn the_extension_reads_this_format() {
    let py = include_str!("../../../../python/spl_book_ext/src/spl_book_ext/code_refs.py");
    assert!(
        py.contains(&format!("\nFORMAT = {FORMAT}\n")),
        "spl_book_ext's FORMAT differs from {FORMAT}"
    );
}
