"""The cards references show, collected once per page, and the safe renderer that writes their contents.

A reference is a plain link: to the glossary entry, or to the lines on the forge. It names its card by id
(`data-spl-card`, `aria-describedby`), so without the script it still works as a link. Each distinct card is written
once per page, at the end of the page's content in a `spl-cards` block the stylesheet hides; the script shows a card
beside its reference on hover, on keyboard focus and on a first tap.

Everything that reaches a card is escaped here or rendered by `render`, a Markdown converter with raw HTML off, no
images, and links only to `http` and `https` addresses: a doc comment holding `<script>` or an `onerror` attribute
comes out as text.
"""

from __future__ import annotations

import functools
import hashlib
import html
import re
import xml.etree.ElementTree as etree
from collections.abc import Callable

import markdown
from markdown.treeprocessors import Treeprocessor

_ATTR = "_spl_cards"
_SAFE_SCHEMES = ("https://", "http://")


class Cards:
    """The distinct cards of one page, in order of first reference: id to HTML"""

    def __init__(self) -> None:
        self.html: dict[str, str] = {}

    def add(self, key: str, build: Callable[[str], str]) -> str:
        """The id of the card for `key`, built by `build(id)` at its first reference on the page"""
        digest = hashlib.sha1(key.encode("utf-8")).hexdigest()[:6]
        slug = re.sub(r"[^a-z0-9]+", "-", key.lower()).strip("-")[:48]
        card_id = f"spl-card-{slug}-{digest}"
        if card_id not in self.html:
            self.html[card_id] = build(card_id)
        return card_id


class _CardsBlock(Treeprocessor):
    """Appends the page's cards after its content"""

    def run(self, root: etree.Element) -> None:
        cards = page_cards(self.md)
        if not cards.html:
            return
        block = etree.SubElement(root, "div", {"class": "spl-cards"})
        block.text = self.md.htmlStash.store("".join(cards.html.values()))


def install(md: markdown.Markdown) -> None:
    """Gives `md` the page's card registry and the block that writes it; both extensions call it"""
    if "spl_cards" not in md.treeprocessors:
        # after the inline patterns (20), which add the cards, and before prettify (10)
        md.treeprocessors.register(_CardsBlock(md), "spl_cards", 15)
    reset(md)


def reset(md: markdown.Markdown) -> None:
    """A new page: no cards yet"""
    setattr(md, _ATTR, Cards())


def page_cards(md: markdown.Markdown) -> Cards:
    cards = getattr(md, _ATTR, None)
    if cards is None:
        cards = Cards()
        setattr(md, _ATTR, cards)
    return cards


def reference(content: etree.Element | str, href: str, kind: str, card_id: str) -> etree.Element:
    """The link a reference becomes: to `href`, naming its card"""
    link = etree.Element(
        "a", {"class": f"spl-ref spl-{kind}", "href": href, "data-spl-card": card_id, "aria-describedby": card_id}
    )
    if isinstance(content, str):
        link.text = content
    else:
        link.append(content)
    return link


def frame(card_id: str, kind: str, body: str) -> str:
    """A card around its contents, which are escaped or rendered already"""
    return f'<div class="spl-card spl-card-{kind}" id="{card_id}" role="tooltip">{body}</div>'


def part(cls: str, inner: str) -> str:
    return f'<div class="{cls}">{inner}</div>'


def escape(text: str) -> str:
    return html.escape(text, quote=True)


class _Safe(Treeprocessor):
    """Links only to http and https; images become their alternative text"""

    def run(self, root: etree.Element) -> None:
        for el in list(root.iter()):
            if el.tag == "a" and not el.get("href", "").startswith(_SAFE_SCHEMES):
                el.tag = "span"
                el.attrib.clear()
            elif el.tag == "img":
                alt = el.get("alt", "")
                el.tag = "span"
                el.attrib.clear()
                el.text = alt


@functools.cache
def _renderer() -> markdown.Markdown:
    md = markdown.Markdown(
        extensions=["pymdownx.superfences", "pymdownx.highlight"],
        extension_configs={"pymdownx.highlight": {"pygments_lang_class": True}},
    )
    md.preprocessors.deregister("html_block")
    md.inlinePatterns.deregister("html")
    md.treeprocessors.register(_Safe(md), "spl_safe", 1)
    return md


def render(text: str) -> str:
    """Markdown as safe HTML, code blocks highlighted as the site highlights them"""
    md = _renderer()
    md.reset()
    return md.convert(text)


def render_inline(text: str) -> str:
    """One paragraph of Markdown as safe HTML, without its paragraph"""
    out = render(text)
    if out.startswith("<p>") and out.endswith("</p>") and out.count("<p>") == 1:
        return out[3:-4]
    return out


def code(text: str, lang: str) -> str:
    """Code highlighted as the site highlights a fenced block"""
    longest = max((len(m) for m in re.findall(r"`+", text)), default=0)
    fence = "`" * max(3, longest + 1)
    return render(f"{fence}{lang}\n{text}\n{fence}")
