set dotenv-load := true
set positional-arguments
set export

help:
    @just --list --unsorted

build:
    cargo build
alias b := build

test *ARGS:
    @just oasgen/test "$ARGS"

check:
    cargo check
alias c := check

fix:
    cargo clippy --fix

# Bump version. level=major,minor,patch
version level:
   #!/bin/bash -euxo pipefail
   git diff-index --exit-code HEAD > /dev/null || ! echo You have untracked changes. Commit your changes before bumping the version. || exit 1

   echo $(dye -c INFO) Make sure that it builds first.
   just test

   cargo set-version --bump {{ level }} --workspace --exclude swagger-ui2
   VERSION=$(rg -om1 "version = \"(.*)\"" --replace '$1' oasgen/Cargo.toml)

   git commit -am "Bump version {{level}} to $VERSION" && \
       git tag v$VERSION && \
       git push --tags

publish:
   cd core && cargo publish --all-features
   cd macro && cargo publish --all-features
   cd oasgen && cargo publish --all-features

patch: test
    just version patch
    just publish

doc:
     @just oasgen/doc
