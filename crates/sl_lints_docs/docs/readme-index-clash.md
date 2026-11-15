## Why

Zensical turns both `README.md` and `index.md` into a folder's page; its docs call the result of having both in the same folder undefined. One of the two is built and the other sits in the repository unread, and which one wins can change with a Zensical release.

The lint looks for a docs folder that holds both names. The project root is outside the docs folder and never counts: a repository's own `README.md` is for GitHub, not the book.

## How to fix

Keep one of the two files. `index.md` is Zensical's own convention for a folder's page; fold any content worth keeping from `README.md` into it and delete `README.md`, or the other way round if the project prefers `README.md` throughout its docs folder.
