## Why

Zensical's theme features are flags in `project.theme.features`, and some of them do not work together: `navigation.prune` strips the navigation down to what is visible, which `navigation.expand` needs in full; `navigation.indexes` and `toc.integrate` both want the same space in the sidebar. A project that turns both flags of a pair on gets no error from `zensical build`, only a theme that silently drops or garbles one of them.

`navigation.instant` (and the instant-navigation flags built on it) reads the site's `sitemap.xml` to know what to fetch. Without `project.site_url` set, Zensical builds an empty sitemap, and instant navigation never finds a page to load.

## How to fix

Keep only one feature of a conflicting pair in `project.theme.features`. For a feature in `needs-site-url`, set `project.site_url` to the book's published URL.
