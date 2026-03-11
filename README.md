<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->
<div align="center">

<h4>A CLI tool that parses Ignition project exports and gateway backups.</h4>

<a href="https://github.com/marcelo-6/igniscope/relseases"><img src="https://img.shields.io/github/v/release/marcelo-6/igniscope?logo=github" alt="GitHub Release"></a>
<a href="https://crates.io/crates/igniscope/"><img src="https://img.shields.io/crates/v/igniscope?logo=Rust" alt="Crate Release"></a>
<a href="https://codecov.io/gh/marcelo-6/igniscope"><img src="https://codecov.io/gh/marcelo-6/igniscope/graph/badge.svg?token=TPJMXTJ5ZQ&amp;logo=Codecov&amp;logoColor=white" alt="Coverage"></a>
<br>
<a href="https://github.com/marcelo-6/igniscope/actions?query=workflow%3A%22CI%22"><img src="https://img.shields.io/github/actions/workflow/status/marcelo-6/igniscope/ci.yml?branch=main&amp;logo=GitHub%20Actions&amp;logoColor=white&amp;label=CI" alt="Continuous Integration"></a>
<a href="https://github.com/marcelo-6/igniscope/actions?query=workflow%3A%22CD%22"><img src="https://img.shields.io/github/actions/workflow/status/marcelo-6/igniscope/cd.yml?logo=GitHub%20Actions&amp;logoColor=white&amp;label=CD" alt="Continuous Deployment"></a>
<a href="https://docs.rs/igniscope/"><img src="https://img.shields.io/docsrs/igniscope?logo=Rust&amp;logoColor=white" alt="Documentation"></a>

</div>

## Features

- Deterministic zip archive parsing for project exports (`.zip`) and gateway backups (`.gwbk`)
- Clear, typed error handling with stable exit codes

## Installation

From source (current recommended way):

```bash
cargo install --path .
```

This installs the `igniscope` binary.

Planned:

- crates.io publish

## How To Use

Print a summary:

```bash
igniscope summarize ./path/to/archive.zip
```

Print a more verbose summary:

```bash
igniscope -v summarize ./path/to/archive.gwbk
```

Generate analysis artifacts:

```bash
igniscope analyze ./path/to/archive.gwbk --out-dir ./out
```

Expected outputs in `--out-dir`:

- `analytics.json`
- `report.md`

## Project Goals

- Port over my code from python into my first Rust project
- Keep parsing and outputs deterministic so results are reproducible
- Dependency edges (view/script/query linking).
- AST-based script analysis.

## Roadmap

- See [ROADMAP.md](ROADMAP.md) for the evolving idea log with tentative version targets.
