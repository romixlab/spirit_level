# `just jobs=N <recipe>` caps build and test parallelism for that run; unset leaves CARGO_BUILD_JOBS and
# NEXTEST_TEST_THREADS as inherited, or at cargo's default (the CPU count)
jobs := ""
export CARGO_BUILD_JOBS := if jobs != "" { jobs } else { env_var_or_default('CARGO_BUILD_JOBS', num_cpus()) }
export NEXTEST_TEST_THREADS := if jobs != "" { jobs } else { env_var_or_default('NEXTEST_TEST_THREADS', 'num-cpus') }

# List the recipes
default:
    @just --list

# The tests. `just test`: all of them. `just test --changed`: only the crates changed against main and their
# dependents (through tpm when installed, otherwise the full suite)
test *flags:
    {{ if flags == "--changed" { "changedtests run" } else { just_executable() + " test-all" } }}

# Every test: nextest, then the doctests (nextest does not run them), then spl_book_ext's
test-all:
    cargo nextest run --workspace --no-fail-fast
    cargo test --workspace --doc -q
    {{just_executable()}} test-py

# The tests of the Markdown extension spl_book_ext (python/spl_book_ext), with uv
test-py:
    uv run --directory python/spl_book_ext pytest -q

# Format, clippy, spl on itself at the big tier (`just tools` installs the tools it needs), the lint codes, book and README picture freshness
lint:
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo run -q -p spirit_level_cli -- check
    cargo run -q -p spirit_level_cli -- codes --check --base main
    cargo run -q -p spirit_level_cli -- book docs --check
    cargo run -q -p spirit_level_cli -- book sample assets/sample assets/check.svg --only code --check

# Rebuild the checked-in firmware ELF of the panic-site fixtures (needs the thumbv7em-none-eabihf target); the
# checkout's path is remapped, so the DWARF does not depend on where it was built
panic-fixture:
    RUSTFLAGS="--remap-path-prefix={{justfile_directory()}}=/spirit_level" cargo build --release --target thumbv7em-none-eabihf --manifest-path crates/sl_lints_embedded/tests/projects/panic_sites/Cargo.toml --target-dir target/panic-fixture
    cp target/panic-fixture/thumbv7em-none-eabihf/release/panic_sites crates/sl_lints_embedded/tests/projects/panic_sites/firmware.elf
    cp target/panic-fixture/thumbv7em-none-eabihf/release/panic_sites crates/sl_lints_embedded/tests/projects/panic_sites_service/firmware.elf

# Install every adapter's tool that is missing or outside its tested range, with cargo-binstall: the tool fixtures
# run them all and fail on a missing one (cargo-udeps also needs the pinned nightly); then this repo's MSRV
# toolchain, the pinned nightly and cross targets with rustup; then list them
tools:
    cargo run -q -p spirit_level_cli -- tools needed --all --spec --missing | xargs -r cargo binstall -y
    cargo run -q -p spirit_level_cli -- tools needed --toolchains --missing | xargs -r -n1 rustup toolchain install --profile minimal
    cargo run -q -p spirit_level_cli -- tools needed --all --toolchains --missing | xargs -r -n1 rustup toolchain install --profile minimal
    cargo run -q -p spirit_level_cli -- tools needed --targets --missing | xargs -r rustup target add
    cargo run -q -p spirit_level_cli -- tools needed --all

# Regenerate the book's generated pages (lints, tiers, nav) and the README's picture of a check
book:
    cargo run -q -p spirit_level_cli -- book docs
    cargo run -q -p spirit_level_cli -- book sample assets/sample assets/check.svg --only code

# Serve the book at http://localhost:8000
docs: book docs-assets
    uv run zensical serve

# Build the book strict into site/
docs-build: book docs-assets
    uv run zensical build --clean --strict

# What the book needs beside its pages, none of it in git: the logo files in docs/assets/logo, the footer's build
# stamp and the script that shows its age
docs-assets:
    cargo run -q -p spirit_level_cli -- book stamp --name spirit_level docs-overrides/partials/stamp.html docs/javascripts/book-stamp-age.js
    mkdir -p docs/assets/logo
    cp assets/logo/mark-light.svg assets/logo/mark-dark.svg assets/logo/spirit_level-light.svg assets/logo/spirit_level-dark.svg assets/logo/icon-small.svg assets/logo/favicon.ico assets/logo/png/icon-180.png docs/assets/logo/

# Build with the deploy profile and put spl on PATH
install:
    cargo build --profile deploy --locked -p spirit_level_cli
    if command -v tpm >/dev/null; then tpm install target/deploy/spl; else cargo install --locked --path crates/spirit_level_cli; fi

# For `tpm dist build` on a build node: spl, static musl (no glibc skew between the build node and a target)
dist out:
    cargo build --profile deploy --locked --target x86_64-unknown-linux-musl -p spirit_level_cli
    install -D -m 755 -t {{out}}/bin target/x86_64-unknown-linux-musl/deploy/spl

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
