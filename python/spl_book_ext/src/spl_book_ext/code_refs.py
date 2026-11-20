"""`` [`TapeWriter::resume_on`] ``: a public item of the book's crates, named as rustdoc's intra-doc links name it.

The items come from the code index (setting `index`, default `target/spl-book/code_index.json`), which
`spl book refs` writes from the code before each build. A name matches an item whose path ends with it; a name that
starts with a crate's name matches only in that crate, so `ww_tape_fs::writer::resume` and `writer::resume` both
find one function. A trailing `()` or `!` is shown and ignored. A name that no item or several items have raises
`CodeRefError`, which fails the build: a renamed or removed item cannot leave a stale link behind.

The link opens the item's lines on the forge at the commit the index was written from; the card shows its
signature, or its whole source when short, and the first paragraph of its docs. The setting `crates` is read by
`spl book refs` alone: the library crates indexed.
"""

from __future__ import annotations

import functools
import json
import xml.etree.ElementTree as etree
from pathlib import Path

from markdown import Extension
from markdown.inlinepatterns import InlineProcessor

from spl_book_ext.card import prose, reference, text

# [`Name`], [`a::b::Name`], [`f()`], [`m!`]: rustdoc's shortcut form, not followed by a link target
PATTERN = r"\[`((?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Za-z_][A-Za-z0-9_]*)(\(\)|!)?`\](?![\[(])"
FORMAT = 1


class CodeRefError(Exception):
    pass


class Index:
    def __init__(self, path: Path):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except FileNotFoundError as e:
            raise CodeRefError(f"{path} is missing: run `spl book refs` before the build") from e
        except (OSError, ValueError) as e:
            raise CodeRefError(f"{path}: {e}") from e
        if not isinstance(data, dict) or data.get("format") != FORMAT:
            raise CodeRefError(f"{path} is not a code index of format {FORMAT}: update spl or spl_book_ext together")
        self.items: list[dict] = data["items"]
        self.crates = {i["path"].split("::")[0] for i in self.items}

    def find(self, name: str) -> dict:
        want = name.split("::")
        if want[0] in self.crates and len(want) > 1:
            found = [
                i
                for i in self.items
                if i["path"].split("::")[0] == want[0] and i["path"].split("::")[-(len(want) - 1) :] == want[1:]
            ]
        else:
            found = [i for i in self.items if i["path"].split("::")[-len(want) :] == want]
        if not found:
            raise CodeRefError(f"[`{name}`]: no public item of the book's crates has this name")
        if len(found) > 1:
            paths = ", ".join(i["path"] for i in found)
            raise CodeRefError(f"[`{name}`]: several items have this name ({paths}); name more of the path")
        return found[0]


@functools.cache
def _load(path: str, mtime: float) -> Index:
    return Index(Path(path))


def load(path: Path) -> Index:
    """The index at `path`, read once per change of the file"""
    try:
        mtime = path.stat().st_mtime
    except FileNotFoundError as e:
        raise CodeRefError(f"{path} is missing: run `spl book refs` before the build") from e
    return _load(str(path), mtime)


class CodeRefPattern(InlineProcessor):
    def __init__(self, index: Index):
        super().__init__(PATTERN)
        self.index = index

    def handleMatch(self, m, data):
        name, suffix = m.group(1), m.group(2) or ""
        item = self.index.find(name)
        code = etree.Element("code")
        code.text = text(name + suffix)
        span, card = reference(code, item["url"], "code")
        head = etree.SubElement(card, "span", {"class": "spl-card-head"})
        head.text = text(f"{item['kind']} {item['path']}")
        src = etree.SubElement(card, "code", {"class": "spl-card-src"})
        src.text = text(item["source"])
        if item["doc"]:
            doc = etree.SubElement(card, "span", {"class": "spl-card-doc"})
            prose(doc, item["doc"])
        where = etree.SubElement(card, "span", {"class": "spl-card-where"})
        where.text = text(f"{item['file']}:{item['lines'][0]}")
        return span, m.start(0), m.end(0)


class CodeRefExtension(Extension):
    def __init__(self, **kwargs):
        self.config = {
            "index": ["target/spl-book/code_index.json", "the code index, from the project root"],
            "crates": [[], "the library crates spl book refs indexes; empty: every library of the workspace"],
        }
        super().__init__(**kwargs)

    def extendMarkdown(self, md):
        index = load(Path(self.getConfig("index")))
        # before the backtick pattern (190), which would take the code span first
        md.inlinePatterns.register(CodeRefPattern(index), "spl_code_ref", 195)


def makeExtension(**kwargs):
    return CodeRefExtension(**kwargs)
