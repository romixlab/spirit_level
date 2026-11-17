## Why

A book with a `site_url` says where readers find it. If it is built and published by hand, the site falls behind the repo, and a page that broke since the last upload goes live unnoticed. A workflow that builds the book strict and publishes it keeps the site at the address equal to main, and a broken page or link fails the run before it reaches anyone.

The workflows are read from `.github/workflows/`; GitHub's Pages settings are not checked. One workflow must do both, since `actions/deploy-pages` publishes only an artifact of its own run:

- a `run:` step builds strict: `zensical build --strict` or `mkdocs build --strict`, or `just RECIPE` where the recipe's body does;
- a step uses `actions/deploy-pages`, usually in a job after the build has uploaded the site with `actions/upload-pages-artifact`.

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: astral-sh/setup-uv@v6
      - run: uv run zensical build --strict
      - uses: actions/upload-pages-artifact@v3
        with:
          path: site/
  deploy:
    needs: build
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
    steps:
      - uses: actions/deploy-pages@v4
```

A book without a `site_url` is not checked.

## How to fix

Add the workflow, or the missing build or deploy step to the workflow that has the other one. Set the repository's Pages source to "GitHub Actions" once.
