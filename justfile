# ---------------------------------------------------------------------------- #
#                                 DEPENDENCIES                                 #
# ---------------------------------------------------------------------------- #

# Rust: https://rust-lang.org/tools/install
cargo := require("cargo")
rustc := require("rustc")
# git-cliff := require("git-cliff")

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
release_notes_file := "RELEASE_NOTES.md"
release_branch := "main"

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

# publish crate in CD (no local pre-release checks, assumes CI already passed).
[group('cd')]
publish-release:
    @printf '\033[1;34m[info]\033[0m publishing crate to crates.io...\n'
    cargo publish

# build and run
[group('production')]
run:
    cargo run

# print the next semantic version inferred from conventional commits.
[group('cd')]
release-next-version:
    @next="$(git-cliff --config cliff.toml --bumped-version --unreleased --tag-pattern '{{semver_tag_pattern}}')"; \
    if [ -z "$next" ]; then \
      printf '\033[1;31m[error]\033[0m could not infer next version from git history.\n'; \
      exit 1; \
    fi; \
    echo "$next"

# print the full manual release checklist.
[group('cd')]
release-plan:
    @printf '\033[1;34m[info]\033[0m Recommended release flow:\n'; \
    printf '  \033[1;33m1)\033[0m just release-prepare X.Y.Z\n'; \
    printf '  \033[1;33m2)\033[0m review Cargo.toml, Cargo.lock, CHANGELOG.md, and local {{release_notes_file}}\n'; \
    printf '  \033[1;33m3)\033[0m git add Cargo.toml Cargo.lock CHANGELOG.md\n'; \
    printf '  \033[1;33m4)\033[0m git commit -m "chore(release): prepare vX.Y.Z"\n'; \
    printf '  \033[1;33m5)\033[0m git push branch and wait for CI on PR/main\n'; \
    printf '  \033[1;33m6)\033[0m after merge, checkout/pull {{release_branch}}\n'; \
    printf '  \033[1;33m7)\033[0m just release-tag X.Y.Z\n'; \
    printf '  \033[1;33m8)\033[0m monitor CD workflow + crates.io + GitHub Release assets\n'

# run local quality checks before preparing a release (optional but recommended).
[group('cd')]
release-preflight: pre-release
    @printf '\033[1;32m[ok]\033[0m local release preflight passed.\n'

# ensure working tree is clean before creating release artifacts.
[group('cd')]
release-assert-clean:
    @if ! git diff --quiet || ! git diff --cached --quiet; then \
      printf '\033[1;31m[error]\033[0m git working tree is not clean.\n'; \
      printf '\033[1;33m[next]\033[0m commit/stash changes before running release recipes.\n'; \
      exit 1; \
    fi; \
    printf '\033[1;32m[ok]\033[0m working tree is clean.\n'

# ensure release tag is created from the configured release branch.
[group('cd')]
release-assert-main:
    @branch="$(git rev-parse --abbrev-ref HEAD)"; \
    if [ "$branch" != "{{release_branch}}" ]; then \
      printf '\033[1;31m[error]\033[0m current branch is "%s", expected "{{release_branch}}".\n' "$branch"; \
      printf '\033[1;33m[next]\033[0m checkout {{release_branch}} and pull latest merged commit before tagging.\n'; \
      exit 1; \
    fi; \
    printf '\033[1;32m[ok]\033[0m on release branch "{{release_branch}}".\n'

# set package version in Cargo.toml and Cargo.lock.
[group('cd')]
release-bump new_version="":
    @set -eu; \
    target="{{new_version}}"; \
    if [ -z "$target" ]; then \
      target="$(git-cliff --config cliff.toml --bumped-version --unreleased --tag-pattern '{{semver_tag_pattern}}')"; \
    fi; \
    if ! echo "$target" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then \
      printf '\033[1;31m[error]\033[0m version must match X.Y.Z (got %s).\n' "$target"; \
      exit 1; \
    fi; \
    sed -i -E '0,/^version = ".*"/s//version = "'"$target"'"/' Cargo.toml; \
    awk -v v="$target" 'BEGIN{pkg=0;done=0} /^\[\[package\]\]/{pkg=0} /^name = "igniscope"$/ {pkg=1} pkg && /^version = "/ && !done {$0="version = \"" v "\""; done=1} {print}' Cargo.lock > Cargo.lock.tmp; \
    mv Cargo.lock.tmp Cargo.lock; \
    printf '\033[1;32m[ok]\033[0m version bumped to %s in Cargo.toml and Cargo.lock.\n' "$target"; \
    printf '\033[1;36m[next]\033[0m just release-changelog %s\n' "$target"

# generate CHANGELOG.md with unreleased entries as the release version.
[group('cd')]
release-changelog new_version=version:
    @set -eu; \
    target="{{new_version}}"; \
    git-cliff --config cliff.toml --unreleased --tag "$target" --tag-pattern '{{semver_tag_pattern}}' --output CHANGELOG.md; \
    printf '\033[1;32m[ok]\033[0m updated CHANGELOG.md for v%s.\n' "$target"; \
    printf '\033[1;36m[next]\033[0m just release-notes %s\n' "$target"

# extract release notes for one version from CHANGELOG.md.
[group('cd')]
release-notes new_version=version out_file=release_notes_file:
    @set -eu; \
    target="{{new_version}}"; \
    awk -v v="$target" '\
      $0 ~ "^## \\[" v "\\]" {in_section=1; print; next} \
      $0 ~ "^## \\[" && in_section {exit} \
      in_section {print}' CHANGELOG.md > "{{out_file}}"; \
    if [ ! -s "{{out_file}}" ]; then \
      printf '\033[1;31m[error]\033[0m failed to extract release notes for v%s from CHANGELOG.md.\n' "$target"; \
      exit 1; \
    fi; \
    printf '\033[1;32m[ok]\033[0m wrote release notes for v%s to {{out_file}}.\n' "$target"; \
    printf '\033[1;36m[next]\033[0m local preview only; keep {{out_file}} untracked.\n'

# prepare version + changelog + local release notes preview.
[group('cd')]
release-prepare new_version="": release-assert-clean
    @set -eu; \
    target="{{new_version}}"; \
    if [ -z "$target" ]; then \
      target="$(git-cliff --config cliff.toml --bumped-version --unreleased --tag-pattern '{{semver_tag_pattern}}')"; \
    fi; \
    just release-bump "$target"; \
    just release-changelog "$target"; \
    just release-notes "$target"; \
    printf '\n\033[1;32m[ok]\033[0m release preparation complete for v%s.\n' "$target"; \
    printf '\033[1;34m[info]\033[0m Next steps:\n'; \
    printf '  \033[1;33m1)\033[0m review Cargo.toml, Cargo.lock, CHANGELOG.md, and {{release_notes_file}}\n'; \
    printf '  \033[1;33m2)\033[0m git add Cargo.toml Cargo.lock CHANGELOG.md\n'; \
    printf '  \033[1;33m3)\033[0m git commit -m "chore(release): prepare v%s"\n' "$target"; \
    printf '  \033[1;33m4)\033[0m git push and wait for CI on PR/main\n'; \
    printf '  \033[1;33m5)\033[0m after merge: git checkout {{release_branch}} && git pull\n'; \
    printf '  \033[1;33m6)\033[0m just release-tag %s\n' "$target"

# create and push an annotated release tag from main.
[group('cd')]
release-tag new_version: release-assert-clean release-assert-main
    @set -eu; \
    target="{{new_version}}"; \
    just release-verify-tag "v$target"; \
    if git show-ref --tags --verify --quiet "refs/tags/v$target"; then \
      printf '\033[1;31m[error]\033[0m tag v%s already exists.\n' "$target"; \
      exit 1; \
    fi; \
    if git ls-remote --tags origin "refs/tags/v$target" | grep -q .; then \
      printf '\033[1;31m[error]\033[0m remote tag v%s already exists on origin.\n' "$target"; \
      exit 1; \
    fi; \
    git tag -a "v$target" -m "v$target"; \
    git push origin "v$target"; \
    printf '\033[1;32m[ok]\033[0m created and pushed tag "v%s".\n' "$target"; \
    printf '\033[1;34m[info]\033[0m Next steps:\n'; \
    printf '  \033[1;33m1)\033[0m check GitHub Actions CD workflow for v%s\n' "$target"; \
    printf '  \033[1;33m2)\033[0m check crates.io publish result\n'; \
    printf '  \033[1;33m3)\033[0m check GitHub Release assets + notes\n'

# validate that a tag value matches Cargo.toml version (for CI/CD use).
[group('cd')]
release-verify-tag tag:
    @set -eu; \
    raw="{{tag}}"; \
    target="${raw#v}"; \
    if ! echo "$target" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then \
      printf '\033[1;31m[error]\033[0m tag "%s" is not semver (expected vX.Y.Z or X.Y.Z).\n' "$raw"; \
      exit 1; \
    fi; \
    cargo_version="$(sed -nE 's/^version = "([^"]+)"/\1/p' Cargo.toml | head -1)"; \
    if [ "$target" != "$cargo_version" ]; then \
      printf '\033[1;31m[error]\033[0m tag version "%s" does not match Cargo.toml version "%s".\n' "$target" "$cargo_version"; \
      exit 1; \
    fi; \
    printf '\033[1;32m[ok]\033[0m tag "%s" matches Cargo.toml version "%s".\n' "$raw" "$cargo_version"

# build and package the release binary for current runner OS.
[group('cd')]
release-artifact out_dir="dist":
    @set -eu; \
    cargo build --release; \
    mkdir -p "{{out_dir}}"; \
    if [ -f target/release/igniscope.exe ]; then \
      cp target/release/igniscope.exe "{{out_dir}}/igniscope-x86_64-pc-windows-msvc.exe"; \
      printf '\033[1;32m[ok]\033[0m wrote {{out_dir}}/igniscope-x86_64-pc-windows-msvc.exe\n'; \
    elif [ -f target/release/igniscope ]; then \
      cp target/release/igniscope "{{out_dir}}/igniscope-x86_64-unknown-linux-gnu"; \
      printf '\033[1;32m[ok]\033[0m wrote {{out_dir}}/igniscope-x86_64-unknown-linux-gnu\n'; \
    else \
      printf '\033[1;31m[error]\033[0m release binary not found under target/release.\n'; \
      exit 1; \
    fi
