## Why

Emoji in docs and comments are decoration that agents add by habit: they do not grep, they render differently on every terminal, and they carry no meaning a word would not carry better. The status marks of a feature tracker (✅ 🚧 📋 🐛 💡 ⛔ 🔍 🧪) are allowed by default; other marks a project uses on purpose go into `allow`.

The rule checks pictographs, symbols and dingbats (not arrows, math or box drawing) in the Markdown files of `files` and in Rust comments; code spans and quoted text are left out.

```markdown
- 🚀 Faster startup     <- reported
- ✅ `CFG-1` Tiers      <- allowed by default (a FEATURES.md status mark)
- 🧪 `CFG-2` Expiries   <- allowed by default too
```

## How to fix

Say it in words, or add the mark to `allow` when the project uses it with a fixed meaning.
