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

# Format, clippy, spl on itself at the big tier (after installing the tools it needs), book freshness
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
docs: book docs-assets
    uv run zensical serve

# Build the book strict into site/
docs-build: book docs-assets
    uv run zensical build --clean --strict

# Copy the logo files the book uses into docs/assets/logo (not in git)
docs-assets:
    mkdir -p docs/assets/logo
    cp assets/logo/mark-light.svg assets/logo/mark-dark.svg assets/logo/icon-small.svg assets/logo/favicon.ico assets/logo/png/icon-180.png docs/assets/logo/

# Build with the deploy profile and put spl on PATH
install:
    cargo build --profile deploy --locked -p spirit_level_cli
    if command -v tpm >/dev/null; then tpm install target/deploy/spl; else cargo install --locked --path crates/spirit_level_cli; fi

# Bring this PC up to main: refuse on local changes, pull, install
deploy:
    git diff --quiet && git diff --cached --quiet
    git pull --ff-only
    just install

# Export the logo from assets/logo/logo.typ: SVG of each variant, the PNG files, favicon.ico. Needs typst 0.13 or
# newer and the Montserrat font installed (SIL OFL, Google Fonts; not in the repo)
logo: logo-marks logo-lockups logo-icons logo-png

# The mark alone: light, dark, one colour (currentColor)
logo-marks:
    just _logo mark-light.svg mark light
    just _logo mark-dark.svg mark dark
    just _logo mark-mono.svg mark mono
    sed -i 's/#000000/currentColor/g' assets/logo/mark-mono.svg

# The mark with the wordmark, horizontal and stacked, each light, dark and one colour
logo-lockups:
    just _logo spirit_level-light.svg horizontal light
    just _logo spirit_level-dark.svg horizontal dark
    just _logo spirit_level-mono.svg horizontal mono
    just _logo spirit_level-stacked-light.svg stacked light
    just _logo spirit_level-stacked-dark.svg stacked dark
    just _logo spirit_level-stacked-mono.svg stacked mono
    sed -i 's/#000000/currentColor/g' assets/logo/spirit_level-mono.svg assets/logo/spirit_level-stacked-mono.svg

# The icons: the tile icon, the small one, its PNG sizes and favicon.ico
logo-icons:
    just _logo icon.svg icon
    just _logo icon-small.svg icon-small
    just _logo png/icon-small-16.png icon-small light 16
    just _logo png/icon-small-32.png icon-small light 32
    just _logo png/icon-small-48.png icon-small light 48
    magick assets/logo/png/icon-small-16.png assets/logo/png/icon-small-32.png assets/logo/png/icon-small-48.png assets/logo/favicon.ico

# The large PNG files: apple-touch and app icons, the lockups, the social card
logo-png:
    just _logo png/icon-180.png icon light 180
    just _logo png/icon-512.png icon light 512
    just _logo png/spirit_level-light-1280.png horizontal light 1280
    just _logo png/spirit_level-dark-1280.png horizontal dark 1280
    just _logo png/social.png social

# One variant: the output's extension picks the format, size is the longer side in pixels (empty: natural size)
_logo out kind theme="light" size="":
    typst compile --ppi 72 --input kind={{kind}} --input theme={{theme}} --input size={{size}} assets/logo/render.typ assets/logo/{{out}}
