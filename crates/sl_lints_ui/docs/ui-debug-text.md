## Why

`{:?}` prints a value the way Rust sees it: `Os { code: 2, kind: NotFound, message: "No such file or directory" }`, quotes and enum variants included. That is for developers. A person reads what happened and what to do next, in plain words.

The lint reports the format strings in the arguments of `calls` that show a value with `:?` or `:#?`. Findings are estimates: an id shown in its debug form on purpose is fine and gets a waiver.

## How to fix

Show the value with `Display`, or the fields that matter. An error says what failed and what the person can do: "Cannot open the project: the file is gone; pick another one".
