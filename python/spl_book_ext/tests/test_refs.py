import html as html_lib
import json
import re

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

BLOB = "https://github.com/o/r/blob/abc/"

RESUME_RS = "fn x() {}\n// --8<-- [start:call]\nlet w = writer.resume_on(cut)?;\n// --8<-- [end:call]\n"


def item(path, kind="fn", doc="Does it.", lines=(3, 9)):
    return {
        "path": path,
        "kind": kind,
        "file": "src/lib.rs",
        "lines": list(lines),
        "source": f"pub {kind} {path.split('::')[-1]}()",
        "doc": doc,
        "url": f"{BLOB}src/lib.rs#L{lines[0]}-L{lines[1]}",
    }


LONG_DOC = "Reopen the piece.\n\nIt finds the cut and goes on: see [`Recovery`] and [`Gone`].\n\n" + "\n\n".join(
    f"Paragraph {i} " + "word " * 30 for i in range(10)
)


@pytest.fixture
def book(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    (tmp_path / "glossary.toml").write_text(GLOSSARY)
    (tmp_path / "tests").mkdir()
    (tmp_path / "tests" / "resume.rs").write_text(RESUME_RS)
    index = {
        "format": 2,
        "commit": "abc",
        "blob": BLOB,
        "items": [
            item("tape::writer::TapeWriter::resume_on", doc=LONG_DOC),
            item("tape::chain::resume"),
            item("tape::writer::resume"),
            item("tape_fs::writer::resume"),
            item("tape::hello", "macro"),
            item("tape::Recovery", "enum", lines=(20, 30)),
            item("tape::Evil", "struct", doc="Bad <script>alert(1)</script> and <img src=x onerror=alert(2)>.\n\n[click](javascript:alert(3))"),
        ],
        "files": [
            {"path": "tests/resume.rs", "lines": 4, "sections": {"call": [3, 3]}},
            {"path": "src/lib.rs", "lines": 40},
        ],
    }
    (tmp_path / "index.json").write_text(json.dumps(index))
    return tmp_path


def render(book, text, abbr=False, **code):
    exts = ["spl_book_ext.glossary", "spl_book_ext.code_refs", *(["abbr"] if abbr else [])]
    return markdown.markdown(
        text,
        extensions=exts,
        extension_configs={
            "spl_book_ext.glossary": {"file": "glossary.toml"},
            "spl_book_ext.code_refs": {"index": "index.json", **code},
        },
    )


def cards(html):
    return re.findall(r'<div class="spl-card spl-card-\w+" id="([^"]+)"', html)


def test_a_term_is_a_plain_link_naming_its_card(book):
    html = render(book, "Each of the [[pieces]] and [[the file|piece]].")
    (cid,) = cards(html)
    assert f'<a aria-describedby="{cid}" class="spl-ref spl-term" data-spl-card="{cid}" href="/glossary/#term-piece">pieces</a>' in html
    assert ">the file</a>" in html
    assert "<strong>piece</strong>: One file of a tape, written in <code>blocks</code>." in html


def test_a_card_is_written_once_per_page(book):
    html = render(book, "[`resume_on`], [`TapeWriter::resume_on`] and [[piece]], [[pieces]].")
    assert len(cards(html)) == 2
    assert html.count('data-spl-card="') == 4
    assert html.index('class="spl-cards"') > html.index("pieces</a>")


def test_the_next_page_has_its_own_cards(book):
    md = markdown.Markdown(
        extensions=["spl_book_ext.code_refs"], extension_configs={"spl_book_ext.code_refs": {"index": "index.json"}}
    )
    assert len(cards(md.convert("[`resume_on`]"))) == 1
    md.reset()
    assert cards(md.convert("No references.")) == []


def test_an_unknown_term_fails(book):
    with pytest.raises(GlossaryError, match=r"\[\[chunk\]\]: no such term"):
        render(book, "A [[chunk]].")


def test_a_code_span_keeps_brackets_as_written(book):
    assert "<code>[[chunk]]</code>" in render(book, "Write `[[chunk]]` to mark a term.")
    assert "spl-ref" not in render(book, "Plain `resume.rs` and `Recovery`.")


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


def test_an_item_card_renders_its_docs_and_highlights_its_signature(book):
    html = render(book, "Call [`TapeWriter::resume_on`] after a crash.")
    assert f'href="{BLOB}src/lib.rs#L3-L9"><code>TapeWriter::resume_on</code></a>' in html
    assert '<div class="spl-card-head">fn tape::writer::TapeWriter::resume_on</div>' in html
    assert 'class="language-rust highlight"' in html
    # an intra-doc link that resolves opens the item's lines; one that does not stays plain code
    assert f'<a href="{BLOB}src/lib.rs#L20-L30"><code>Recovery</code></a>' in html
    assert "<code>Gone</code>" in html and ">Gone</a>" not in html
    assert '<div class="spl-card-where">src/lib.rs:3</div>' in html


def test_long_docs_show_the_summary_and_what_fits(book):
    html = render(book, "[`resume_on`]", card_doc_chars_max=350)
    assert "Reopen the piece." in html and "Paragraph 0" in html and "Paragraph 1" not in html
    assert "The docs go on in the source" in html
    assert "Paragraph 9" in render(book, "[`resume_on`]", card_doc_chars_max=100000)


def test_docs_are_text_never_markup(book):
    html = render(book, "[`Evil`]")
    assert "<script" not in html and "<img" not in html
    assert "&lt;script&gt;alert(1)&lt;/script&gt;" in html
    assert "onerror=alert(2)&gt;" in html
    assert "javascript:" not in html


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


def test_a_section_card_shows_its_lines(book):
    html = render(book, "See [`resume.rs:call`].")
    assert f'href="{BLOB}tests/resume.rs#L3-L3"><code>resume.rs:call</code></a>' in html
    assert '<div class="spl-card-head">tests/resume.rs, section call</div>' in html
    assert "resume_on" in html and "fn x()" not in html
    assert '<div class="spl-card-where">tests/resume.rs:3-3</div>' in html


def test_a_file_card_is_bounded(book):
    html = render(book, "See [`tests/resume.rs`].", card_lines_max=2)
    assert f'href="{BLOB}tests/resume.rs"' in html
    assert "2 more lines in the source" in html


def test_a_stale_file_or_section_fails(book):
    with pytest.raises(CodeRefError, match="no file of the project"):
        render(book, "[`gone.rs`]")
    with pytest.raises(CodeRefError, match="marks no section `cut` \\(it marks: call\\)"):
        render(book, "[`resume.rs:cut`]")


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
    index["items"].append(item("tape::Evil2", "struct", doc=EVIL))
    (book / "index.json").write_text(json.dumps(index))
    assert_text(render(book, "[`Evil2`]"))


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
    # highlighted, the signature is split into spans; its text is whole and no part of it is a tag
    assert "<S:" not in html and "<S::" not in html
    text = html_lib.unescape(re.sub(r"<[^>]+>", "", html))
    assert "pub fn open<S: Sink>(s: S) -> Result<(TapeWriter, Resumed), WriteError<S::Error>>" in text
    assert "<code>Sink&lt;S&gt;</code>" in html
