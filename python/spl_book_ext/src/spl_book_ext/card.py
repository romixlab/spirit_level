"""The card both references draw: the reference as a link, and a box shown on hover and on focus."""

from __future__ import annotations

import re
import xml.etree.ElementTree as etree

# rustdoc's intra-doc links in a doc comment, `[Self::x]`, `[text](target)` and `[text][ref]`, shown as their text
_DOC_LINK = re.compile(r"\[([^\[\]]+)\](?:\([^()]*\)|\[[^\[\]]*\])?")


def reference(text: etree.Element | str, href: str, kind: str) -> tuple[etree.Element, etree.Element]:
    """A `spl-ref` span holding a link to `href` showing `text`, and the empty card to fill."""
    span = etree.Element("span", {"class": f"spl-ref spl-{kind}", "tabindex": "0"})
    link = etree.SubElement(span, "a", {"href": href})
    if isinstance(text, str):
        link.text = text
    else:
        link.append(text)
    card = etree.SubElement(span, "span", {"class": "spl-card", "role": "tooltip"})
    return span, card


def prose(parent: etree.Element, text: str) -> None:
    """`text` from a doc comment or a definition into `parent`: links shown as their text, `code` as code."""
    text = _DOC_LINK.sub(lambda m: m.group(1), text)
    last: etree.Element | None = None
    for i, part in enumerate(text.split("`")):
        if i % 2 == 1:
            last = etree.SubElement(parent, "code")
            last.text = part
        elif last is None:
            parent.text = (parent.text or "") + part
        else:
            last.tail = (last.tail or "") + part
