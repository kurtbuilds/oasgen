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
   #git diff-index --exit-code HEAD > /dev/null || ! echo You have untracked changes. Commit your changes before bumping the version.

   #echo $(dye -c INFO) Make sure that it builds first.
   #just test

   cargo set-version --bump {{ level }} --workspace
   VERSION=$(toml get oasgen/Cargo.toml package.version) && \
       (cd macro && toml set Cargo.toml dependencies.oasgen-core.version $VERSION && cargo update) && \
       (cd oasgen && toml set Cargo.toml dependencies.oasgen-core.version $VERSION && toml set Cargo.toml dependencies.oasgen-macro.version $VERSION && cargo update) && \
       git commit -am "Bump version {{level}} to $VERSION" && \
       git tag v$VERSION && \
       git push origin v$VERSION
   git push

publish:
   cd core && cargo publish --features actix
   @echo $(dye -c INFO) Need to sleep so that crates.io has time to update.
   sleep 5
   cd macro && cargo publish
   @echo $(dye -c INFO) Need to sleep so that crates.io has time to update.
   sleep 5
   cd oasgen && cargo publish --features actix

patch: test
    just version patch
    just publish
