## Why

A check that only runs on one person's PC is skipped the day they are in a hurry. A CI workflow runs the same checks on every push and pull request, and its badge in the README shows whether main passes.

The workflow is read from the repo's own files (the parameter `path`, `.github/workflows/ci.yml` by default); what GitHub's settings do with it is not checked.

What counts as the repo's checks depends on the repo:

- With a justfile that has `lint` and `test` recipes, a `run:` step calls `just lint` and one calls `just test` (or a variant such as `just test-all`). CI then runs exactly what a person runs.
- Without them, a `run:` step calls `cargo clippy` and one calls `cargo test` or `cargo nextest`.

```yaml
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: taiki-e/install-action@v2
        with:
          tool: just
      - run: just lint
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: taiki-e/install-action@v2
        with:
          tool: just,cargo-nextest
      - run: just test
```

## How to fix

Add the workflow, or the step the finding names, to one of its jobs. A workflow with another name sets the parameter `path`.
