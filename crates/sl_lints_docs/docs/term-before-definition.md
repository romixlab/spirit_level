## Why

A page that uses a term before it says what the term means leaves the reader to guess. The guess shapes how they read every sentence until the definition, and a wrong guess has to be unlearned.

The lint takes two kinds of definition: bold text of one to four words (`A **tier** is ...`) and a sentence that opens with the term and "means", "stands for", "refers to" or "is short for". It reports a term when the page's prose uses it earlier, singular and plural alike. A heading may name the term before its definition. Bold that only stresses a word or labels a paragraph (`**Note:**`) is left out through the `ignore` list.

The finding is a candidate: bold text is not always a definition.

```markdown
Each tier extends another.          <!-- used here -->

A **tier** is a set of lints.       <!-- defined here -->
```

## How to fix

Move the definition up to the first use, or define the term where the reader first meets it. Bold that defines nothing goes into the `ignore` list.
