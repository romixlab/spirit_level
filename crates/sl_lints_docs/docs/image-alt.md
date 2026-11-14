## Why

Alternative text is what a screen reader reads aloud, what a search engine indexes, and what the reader sees when the image does not load. An image without it tells those readers nothing.

```markdown
![](board-top.png)                          <- nothing to read
![The board from the top, USB on the left](board-top.png)
```

In Markdown an image always has its text between the brackets, so empty brackets are reported. An HTML `<img>` is reported when it has no `alt` attribute at all. `alt=""` marks an image that is only decoration, and is left alone.

## How to fix

Write what the image shows, in the words a reader would need if they could not see it. For an image that only decorates, use HTML with `alt=""`.
