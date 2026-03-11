# ---------------------------------------------------------------------------- #
#                                 DEPENDENCIES                                 #
# ---------------------------------------------------------------------------- #

# Rust: https://rust-lang.org/tools/install
cargo := require("cargo")
rustc := require("rustc")
git-cliff := require("git-cliff")

# ---------------------------------------------------------------------------- #
#                                    RECIPES                                   #
# ---------------------------------------------------------------------------- #

# Get version from Cargo.toml/Cargo.lock
#
# Alternative command:
# `cargo metadata --format-version=1 | jq '.packages[]|select(.name=="rust-template").version'`
version := `cargo pkgid | sed -rn s'/^.*#(.*)$/\1/p'`

# coverage threshold to fail (CI)
coverage_threshold := "70"

# semver tag pattern
semver_tag_pattern := "^v?[0-9]+\\.[0-9]+\\.[0-9]+$"

# show available commands
[group('project-agnostic')]
default:
    @just --list

# evaluate and print all just variables
[group('project-agnostic')]
just-vars:
    @just --evaluate

# print system information such as OS and architecture
[group('project-agnostic')]
system-info:
    @echo "architecture: {{arch()}}"
    @echo "os: {{os()}}"
    @echo "os family: {{os_family()}}"

# lint the sources
[group('development')]
lint:
    cargo fmt --all --check
    cargo clippy -- --deny warnings

# build the program
[group('development')]
build:
    cargo build

# analyze the current package and report errors, but don't build object files (faster than 'build')
[group('development')]
check:
    cargo check

# remove generated artifacts
[group('development')]
clean:
    cargo clean

# show test coverage (requires https://lib.rs/crates/cargo-llvm-cov)
[group('development')]
coverage threshold=coverage_threshold:
    cargo llvm-cov --fail-under-lines {{threshold}} --show-missing-lines --quiet
alias cov := coverage

# run ci workflow (lint, check, test, cov) (requires https://lib.rs/crates/cargo-llvm-cov)
[group('ci')]
ci: lint check
    cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info --fail-under-lines {{coverage_threshold}} --quiet

# generate the full changelog into CHANGELOG.md.
[group('cd')]
changelog:
    git-cliff --config cliff.toml --tag-pattern '{{semver_tag_pattern}}' --output CHANGELOG.md

# dry run changelog generation.
[group('development')]
changelog-dry-run:
    next="$(git-cliff --config cliff.toml --bumped-version --unreleased --tag-pattern '{{semver_tag_pattern}}')"; \
    echo "Project version {{version}} -> next ${next}"
    git-cliff --config cliff.toml --unreleased --tag "${next}" --tag-pattern '{{semver_tag_pattern}}'

# show dependencies of this project
[group('development')]
dependencies:
    cargo tree

# generate the documentation of this project
[group('development')]
docs:
    cargo doc --open

# build and install the binary locally
[group('development')]
install: build test
    cargo install --path .

# show version of this project
[group('development')]
version:
    @echo "Project {{version}}"
    @rustc --version
    @cargo --version

# run tests
[group('development')]
test: lint
    cargo test

# check, test, lint
[group('development')]
pre-release: check test lint

# dry runs the publish crate
[group('development')]
publish-dry-run: pre-release
    cargo publish --dry-run

# build release executable
[group('production')]
release: pre-release
    cargo build --release

# publish crate
[group('production')]
publish: pre-release
    cargo publish

# build and run
[group('production')]
run:
    cargo run