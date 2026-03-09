# ---------------------------------------------------------------------------- #
#                                 DEPENDENCIES                                 #
# ---------------------------------------------------------------------------- #

# Rust: https://rust-lang.org/tools/install
cargo := require("cargo")
rustc := require("rustc")

# ---------------------------------------------------------------------------- #
#                                    RECIPES                                   #
# ---------------------------------------------------------------------------- #

# Get version from Cargo.toml/Cargo.lock
#
# Alternative command:
# `cargo metadata --format-version=1 | jq '.packages[]|select(.name=="rust-template").version'`
version := `cargo pkgid | sed -rn s'/^.*#(.*)$/\1/p'`

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
coverage:
    cargo llvm-cov

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

# build release executable
[group('production')]
release: pre-release
    cargo build --release

# build and run
[group('production')]
run:
    cargo run