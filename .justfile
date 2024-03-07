set shell := ["bash", "-c"]

default: 
    just -l
# Run the program
@run *ARGS="":
    cargo run --quiet -- {{ARGS}}

# Build the program
build OUTPUT="./target/release":
    cargo build --release -Z unstable-options --quiet --out-dir {{OUTPUT}}

bump BUMP_TYPE="patch":
    #!/usr/bin/env bash
    set -euxo pipefail
    cargo set-version --bump {{BUMP_TYPE}}
    git add Cargo.toml Cargo.lock
    VERSION=$(cargo pkgid | cut -d# -f2 | cut -d: -f2)
    git commit -m "Bump version to v$VERSION"
    git tag -a v$VERSION -m "Release v$VERSION"
    git push --follow-tags
    rr -o dist
