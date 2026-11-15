## Why

A reader takes two names for two things. A page that says "directory" in one paragraph and "folder" in the next sends them looking for the difference. One concept has one name throughout the docs.

The lint reads the project's word list, the `words` parameter: each entry names one concept, its names separated by commas, the preferred name first. Where the names are compared (each page, or with `across = "all"` all pages together) and more than one is in use, every use of a name other than the preferred one is reported. When the preferred name is in use nowhere compared, the most used name is preferred. Words are matched singular and plural alike; inline code is not read.

The finding is a candidate: the same word can name two concepts.

```toml
[lints.term-synonyms]
level = "warn"
params = { words = ["directory, folder", "tier, profile"], across = "all" }
```

## How to fix

Use the preferred name throughout. Where two names really mean two things, take the entry out of the list, or say on the page how they differ.
