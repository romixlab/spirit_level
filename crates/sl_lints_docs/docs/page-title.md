## Why

Zensical takes a page's title from its navigation entry, its front matter `title`, or its first `#` heading. A page with none of them is shown under its file name, `getting_started` instead of "Getting started". A page with two `#` headings has two titles: the browser tab shows the first, and the second looks like a new page.

The lint checks every page the book builds: it needs a `#` heading or a `title:` in its front matter, and at most one `#` heading (none when the front matter sets the title).

## How to fix

Open the page with one `# Title` heading. Make every other top-level heading a `##` section, or move it to a page of its own.
