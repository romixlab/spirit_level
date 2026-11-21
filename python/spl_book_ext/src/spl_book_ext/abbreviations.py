"""Abbreviations with their long form on hover, from the table of the `abbreviation-first-use` lint.

The project's `spirit_level.toml` holds the table (`[lints.abbreviation-first-use.long-forms]`); `spl book refs`
resolves it with the tiers and writes it as a JSON object of abbreviation to long form (setting `file`, default
`target/spl-book/abbreviations.json`). This extension adds every entry to each page as a definition of
Python-Markdown's `abbr` extension, which marks each use of the abbreviation in the page's text as
`<abbr title="long form">`; code spans and code blocks keep it as written. The theme's `content.tooltips` feature
shows the title on hover.

The `abbr` extension must be on: without it the long forms would silently be missing, so the build fails. A missing
or malformed table fails the build too, and names the command that writes it.
"""

from __future__ import annotations

import functools
import json
from pathlib import Path

from markdown import Extension
from markdown.preprocessors import Preprocessor

from spl_book_ext import card


class AbbreviationError(card.RefError):
    pass


@functools.cache
def _load(path: str, mtime: float) -> tuple[tuple[str, str], ...]:
    try:
        table = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as e:
        raise AbbreviationError(f"{path}: {e}") from e
    if not isinstance(table, dict):
        raise AbbreviationError(f"{path}: not a JSON object of abbreviation to long form")
    out = []
    for abbr, long in sorted(table.items()):
        if not isinstance(long, str) or not abbr or any(c in abbr for c in "[]\n") or "\n" in long:
            raise AbbreviationError(f"{path}: {abbr!r} = {long!r} is no abbreviation and long form on one line")
        out.append((abbr, long))
    return tuple(out)


def load(path: Path) -> tuple[tuple[str, str], ...]:
    """The table at `path` as (abbreviation, long form) pairs, read once per change of the file"""
    try:
        mtime = path.stat().st_mtime
    except OSError as e:
        raise AbbreviationError(f"{path}: {e}; run `spl book refs` before the build, it writes the table") from e
    return _load(str(path), mtime)


class Definitions(Preprocessor):
    """Appends each long form to the page as an `abbr` definition"""

    def __init__(self, md, path: Path):
        super().__init__(md)
        self.path = path

    def run(self, lines: list[str]) -> list[str]:
        if "abbr" not in self.md.parser.blockprocessors:
            raise AbbreviationError(
                "spl_book_ext.abbreviations needs the abbr extension: add `abbr = {}` to markdown_extensions"
            )
        table = load(self.path)
        if not table:
            return lines
        return [*lines, "", *(f"*[{abbr}]: {long}" for abbr, long in table)]


class AbbreviationsExtension(Extension):
    def __init__(self, **kwargs):
        self.config = {
            "file": ["target/spl-book/abbreviations.json", "the table of long forms, from the project root"],
        }
        super().__init__(**kwargs)

    def extendMarkdown(self, md):
        md.registerExtension(self)
        # after the glossary page (25): the table's long form comes last and wins over a glossary term's `long`
        md.preprocessors.register(Definitions(md, Path(self.getConfig("file"))), "spl_abbreviations", 24)


def makeExtension(**kwargs):
    return AbbreviationsExtension(**kwargs)
