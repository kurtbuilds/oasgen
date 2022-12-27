set dotenv-load := true
set positional-arguments

help:
    @just --list --unsorted

build:
    cargo build
alias b := build

test:
    @just oasgen/test

check:
    cargo check
alias c := check

fix:
    cargo clippy --fix

# Bump version. level=major,minor,patch
version level:
    git diff-index --exit-code HEAD > /dev/null || ! echo You have untracked changes. Commit your changes before bumping the version.
    cargo set-version --bump {{level}}
    cargo update # This bumps Cargo.lock
    VERSION=$(toml get -r core/Cargo.toml package.version) && \
        git commit -am "Bump version {{level}} to $VERSION" && \
        git tag v$VERSION && \
        git push origin v$VERSION
    git push

publish:
    cargo publish

patch: test
    just version patch
    just publish
