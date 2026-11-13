## Why

Two versions of one crate compile twice and give two incompatible copies of its types (a `syn::Ident` from syn 2 is not one from syn 3). In a big graph they are the cheapest build time to win back.

## How to fix

`cargo tree -d -i NAME` shows who pulls each version. Align the version requirements, move a dependency to the version its siblings use, or `cargo update`. When two versions cannot be avoided yet, list the crate in the parameter `allow` with an override reason (and an expiry).
