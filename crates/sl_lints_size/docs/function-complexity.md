## Why

Cyclomatic complexity counts the independent paths through a function: 1, plus one for each `if`, loop, `?`, `&&`, `||` and each `match` arm after the first. Above about 25 a function cannot be tested path by path or held in one's head.

## How to fix

Pull branches out into functions, turn chains of conditions into a table or an enum with methods, return early instead of nesting.
