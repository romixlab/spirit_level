import json

import markdown
import pytest

from spl_book_ext.code_refs import CodeRefError
from spl_book_ext.glossary import GlossaryError

GLOSSARY = """
[[term]]
name = "piece"
forms = ["pieces"]
short = "One file of a tape, written in `blocks`."
chapter = "../pieces/"

[[term]]
name = "CRC"
long = "cyclic redundancy check"
short = "A checksum."
"""


def item(path, kind="fn", doc="Does it. See [`Other`]."):
    return {
        "path": path,
        "kind": kind,
        "file": "src/lib.rs",
        "lines": [3, 9],
        "source": f"pub {kind} {path.split('::')[-1]}()",
        "doc": doc,
        "url": "https://github.com/o/r/blob/abc/src/lib.rs#L3-L9",
    }


@pytest.fixture
def book(tmp_path):
    (tmp_path / "glossary.toml").write_text(GLOSSARY)
    index = {
        "format": 1,
        "commit": "abc",
        "items": [
            item("tape::writer::TapeWriter::resume_on"),
            item("tape::chain::resume"),
            item("tape::writer::resume"),
            item("tape_fs::writer::resume"),
            item("tape::hello", "macro"),
        ],
    }
    (tmp_path / "index.json").write_text(json.dumps(index))
    return tmp_path


def render(book, text, abbr=False):
    exts = ["spl_book_ext.glossary", "spl_book_ext.code_refs", *(["abbr"] if abbr else [])]
    return markdown.markdown(
        text,
        extensions=exts,
        extension_configs={
            "spl_book_ext.glossary": {"file": str(book / "glossary.toml")},
            "spl_book_ext.code_refs": {"index": str(book / "index.json")},
        },
    )


def test_a_term_links_its_entry_with_a_card(book):
    html = render(book, "Each of the [[pieces]] and [[the file|piece]].")
    assert '<span class="spl-ref spl-term" tabindex="0"><a href="/glossary/#term-piece">pieces</a>' in html
    assert '<a href="/glossary/#term-piece">the file</a>' in html
    assert "<strong>piece</strong>: <span>One file of a tape, written in <code>blocks</code>.</span>" in html


def test_an_unknown_term_fails(book):
    with pytest.raises(GlossaryError, match=r"\[\[chunk\]\]: no such term"):
        render(book, "A [[chunk]].")


def test_a_code_span_keeps_brackets_as_written(book):
    assert "<code>[[chunk]]</code>" in render(book, "Write `[[chunk]]` to mark a term.")


def test_the_glossary_page_lists_every_entry(book):
    html = render(book, "# Glossary\n\n<!-- glossary -->\n")
    assert html.index('id="term-crc"') < html.index('id="term-piece"')
    assert "<strong>CRC (cyclic redundancy check)</strong>: A checksum." in html
    assert 'written in <code>blocks</code>. <a href="../pieces/">More</a>' in html


def test_abbreviations_get_their_long_form_with_abbr_on(book):
    assert '<abbr title="cyclic redundancy check">CRC</abbr>' in render(book, "Each CRC is checked.", abbr=True)
    assert "*[CRC]" not in render(book, "Each CRC is checked.")


def test_a_word_naming_two_terms_fails(book):
    (book / "glossary.toml").write_text(GLOSSARY + '\n[[term]]\nname = "slice"\nforms = ["Piece"]\nshort = "x"\n')
    with pytest.raises(GlossaryError, match='"Piece" names two terms, "piece" and "slice"'):
        render(book, "x")


def test_a_code_link_shows_signature_docs_and_opens_its_lines(book):
    html = render(book, "Call [`TapeWriter::resume_on`] after a crash.")
    assert '<a href="https://github.com/o/r/blob/abc/src/lib.rs#L3-L9"><code>TapeWriter::resume_on</code></a>' in html
    assert '<span class="spl-card-head">fn tape::writer::TapeWriter::resume_on</span>' in html
    assert '<span class="spl-card-doc">Does it. See <code>Other</code>.</span>' in html
    assert '<span class="spl-card-where">src/lib.rs:3</span>' in html


def test_a_crate_prefix_narrows_the_match(book):
    assert "tape_fs::writer::resume" in render(book, "[`tape_fs::writer::resume`]")
    assert "<code>hello!</code>" in render(book, "[`hello!`]")


def test_an_ambiguous_name_lists_the_candidates(book):
    with pytest.raises(CodeRefError, match=r"several items .*tape::chain::resume, tape::writer::resume"):
        render(book, "[`resume`]")


def test_an_unknown_item_fails(book):
    with pytest.raises(CodeRefError, match="no public item"):
        render(book, "[`Gone`]")


def test_a_rustdoc_link_with_a_target_is_left_alone(book):
    assert "spl-ref" not in render(book, "[`Gone`](https://example.org)")


def test_a_missing_index_names_the_command(book):
    (book / "index.json").unlink()
    with pytest.raises(CodeRefError, match="run `spl book refs`"):
        render(book, "x")


EVIL = "<script>alert(1)</script> <img src=x onerror=alert(1)>"


def assert_text(html):
    assert "<script" not in html and "<img" not in html, html
    assert "&lt;script&gt;alert(1)&lt;/script&gt;" in html and "&lt;img src=x onerror=alert(1)&gt;" in html, html


def test_a_doc_comment_is_text(book):
    index = json.loads((book / "index.json").read_text())
    index["items"].append(item("tape::Evil", "struct", doc=EVIL))
    (book / "index.json").write_text(json.dumps(index))
    assert_text(render(book, "[`Evil`]"))


def test_a_glossary_definition_and_name_are_text(book):
    (book / "glossary.toml").write_text(f'[[term]]\nname = "{EVIL}"\nforms = ["evil"]\nshort = "{EVIL}"\n')
    assert_text(render(book, "An [[evil]] term."))


def test_a_signature_with_generics_comes_out_whole(book):
    index = json.loads((book / "index.json").read_text())
    generic = item("tape::open", doc="Opens a `Sink<S>`.")
    generic["source"] = "pub fn open<S: Sink>(s: S) -> Result<(TapeWriter, Resumed), WriteError<S::Error>>"
    index["items"].append(generic)
    (book / "index.json").write_text(json.dumps(index))
    html = render(book, "[`open`]")
    assert "pub fn open&lt;S: Sink&gt;(s: S) -&gt; Result&lt;(TapeWriter, Resumed), WriteError&lt;S::Error&gt;&gt;" in html
    assert "<code>Sink&lt;S&gt;</code>" in html
