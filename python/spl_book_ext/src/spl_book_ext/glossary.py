"""`[[term]]`: a term of the book's glossary in prose, linked to its entry and shown on hover.

The glossary file (setting `file`, default `docs/glossary.toml`) holds one `[[term]]` table per term: its `name`,
the `forms` prose also uses (plurals), a one-sentence `short` definition, the `chapter` that explains it, and for an
abbreviation its `long` form. `[[pieces]]` finds `piece` by its forms; `[[the tail|tail]]` shows other words for a
term. A term the file lacks, or a word naming two terms, raises `GlossaryError`, which fails the build.

The glossary page holds the line `<!-- glossary -->`, which becomes every entry in order of name, each with the
anchor `#term-NAME` that references link to (setting `page`, default `/glossary/`). With the `abbr` extension on,
every page also gets each abbreviation's long form, shown on hover wherever the abbreviation is written.
"""

from __future__ import annotations

import functools
import html
import re
import tomllib
import xml.etree.ElementTree as etree
from pathlib import Path

from markdown import Extension
from markdown.inlinepatterns import InlineProcessor
from markdown.preprocessors import Preprocessor

from spl_book_ext.card import prose, reference, text

PATTERN = r"\[\[([^\[\]|`\n]+)(?:\|([^\[\]|`\n]+))?\]\]"
MARKER = "<!-- glossary -->"
_KEYS = {"name", "forms", "short", "chapter", "long"}


class GlossaryError(Exception):
    pass


def slug(name: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")


class Glossary:
    def __init__(self, path: Path, page: str):
        try:
            data = tomllib.loads(path.read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError) as e:
            raise GlossaryError(f"{path}: {e}") from e
        self.page = page
        terms = data.get("term", [])
        for t in terms:
            unknown = set(t) - _KEYS
            if unknown or "name" not in t or "short" not in t:
                raise GlossaryError(f"{path}: term {t.get('name', '?')!r} needs name and short, knows only {sorted(_KEYS)}")
        self.terms: list[dict] = sorted(terms, key=lambda t: t["name"].lower())
        self.by_form: dict[str, dict] = {}
        for t in terms:
            for form in [t["name"], *t.get("forms", [])]:
                key = form.lower()
                if key in self.by_form and self.by_form[key] is not t:
                    other = self.by_form[key]["name"]
                    raise GlossaryError(f'{path}: "{form}" names two terms, "{other}" and "{t["name"]}"')
                self.by_form[key] = t

    def find(self, form: str) -> dict:
        t = self.by_form.get(form.strip().lower())
        if t is None:
            raise GlossaryError(f"[[{form}]]: no such term in the glossary")
        return t

    def href(self, t: dict) -> str:
        return f"{self.page}#term-{slug(t['name'])}"


@functools.cache
def _load(path: str, mtime: float, page: str) -> Glossary:
    return Glossary(Path(path), page)


def load(path: Path, page: str) -> Glossary:
    """The glossary at `path`, read once per change of the file"""
    try:
        mtime = path.stat().st_mtime
    except OSError as e:
        raise GlossaryError(f"{path}: {e}") from e
    return _load(str(path), mtime, page)


def _inline(text: str) -> str:
    """A definition as HTML: escaped, `code` as code"""
    parts = html.escape(text).split("`")
    return "".join(f"<code>{p}</code>" if i % 2 else p for i, p in enumerate(parts))


class TermPattern(InlineProcessor):
    def __init__(self, glossary: Glossary):
        super().__init__(PATTERN)
        self.glossary = glossary

    def handleMatch(self, m, data):
        shown, name = m.group(1), m.group(2) or m.group(1)
        t = self.glossary.find(name)
        span, card = reference(shown, self.glossary.href(t), "term")
        head = etree.SubElement(card, "strong")
        head.text = text(t["name"] + (f" ({t['long']})" if t.get("long") else ""))
        head.tail = text(": ")
        rest = etree.SubElement(card, "span")
        prose(rest, t["short"])
        return span, m.start(0), m.end(0)


class GlossaryPage(Preprocessor):
    """The marker on the glossary page becomes the entries; with `abbr` on, every page gets the long forms."""

    def __init__(self, md, glossary: Glossary):
        super().__init__(md)
        self.glossary = glossary

    def run(self, lines: list[str]) -> list[str]:
        out: list[str] = []
        for line in lines:
            if line.strip() != MARKER:
                out.append(line)
                continue
            for t in self.glossary.terms:
                name = html.escape(t["name"])
                if t.get("long"):
                    name += f" ({html.escape(t['long'])})"
                more = f' <a href="{html.escape(t["chapter"])}">More</a>' if t.get("chapter") else ""
                out.append(
                    f'<p class="spl-gloss" id="term-{slug(t["name"])}"><strong>{name}</strong>: '
                    f"{_inline(t['short'])}{more}</p>"
                )
                out.append("")
        if "abbr" in self.md.parser.blockprocessors:
            abbrs = [f"*[{t['name']}]: {t['long']}" for t in self.glossary.terms if t.get("long")]
            if abbrs:
                out += ["", *abbrs]
        return out


class GlossaryExtension(Extension):
    def __init__(self, **kwargs):
        self.config = {
            "file": ["docs/glossary.toml", "the glossary's entries, from the project root"],
            "page": ["/glossary/", "where the glossary page is served"],
        }
        super().__init__(**kwargs)

    def extendMarkdown(self, md):
        glossary = load(Path(self.getConfig("file")), self.getConfig("page"))
        md.preprocessors.register(GlossaryPage(md, glossary), "spl_glossary_page", 25)
        # after the backtick pattern (190), so a code span keeps `[[x]]` as written; before the link pattern
        # (160), which would take the brackets apart
        md.inlinePatterns.register(TermPattern(glossary), "spl_glossary_term", 185)


def makeExtension(**kwargs):
    return GlossaryExtension(**kwargs)
