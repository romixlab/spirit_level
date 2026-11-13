# List the recipes
default:
    @just --list

# The tests. `just test`: all of them. `just test --changed`: only the crates changed against main and their
# dependents (through tpm when installed, otherwise the full suite)
test *flags:
    if [ "{{flags}}" = "--changed" ] && command -v tpm >/dev/null 2>&1; then tpm test changed -- cargo nextest run --workspace; else {{just_executable()}} test-all; fi

# Every test: nextest, then the doctests (nextest does not run them)
test-all:
    cargo nextest run --workspace --no-fail-fast
    cargo test --workspace --doc -q

# Format, clippy, spirit-level on itself at the big tier (after installing the tools it needs), book freshness
lint:
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo run -q -p spirit_level_cli -- tools install --needed
    cargo run -q -p spirit_level_cli -- check
    cargo run -q -p spirit_level_cli -- book docs --check

# Install every pinned external tool into the tool cache
tools:
    cargo run -q -p spirit_level_cli -- tools install

# Regenerate the book's generated pages (lints, tiers, nav)
book:
    cargo run -q -p spirit_level_cli -- book docs

# Serve the book at http://localhost:8000
docs: book
    uv run zensical serve

# Build the book strict into site/
docs-build: book
    uv run zensical build --clean --strict

# Build with the deploy profile and put spirit-level on PATH
install:
    cargo build --profile deploy --locked -p spirit_level_cli
    if command -v tpm >/dev/null; then tpm install target/deploy/spirit-level; else cargo install --locked --path crates/spirit_level_cli; fi

# Bring this PC up to main: refuse on local changes, pull, install
deploy:
    git diff --quiet && git diff --cached --quiet
    git pull --ff-only
    just install
