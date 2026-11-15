## Why

A link pasted from a browser, a chat app or a social post carries a tracking query parameter: `utm_source=`, `?si=`, `fbclid=`, and similar. It tells an outside service where the link was shared from; it tells the reader nothing, and the id goes stale.

Parameters match as substrings, case-insensitive, in the Markdown files of `files` and in Rust comments, including a link target (`[text](url)`): that is where a pasted link lives. A code span, a fenced block, a double quote or a curly quote still hides a mention: writing the parameter to explain it, as this page does, is not a use.

```rust
// see [the guide](https://example.com/docs?utm_source=newsletter)   <- finding
// see [the guide](https://example.com/docs)                         <- passes
```

## How to fix

Strip the tracking parameters from the URL before pasting it; keep the path and any parameter the page actually needs.
