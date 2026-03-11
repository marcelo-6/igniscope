<!-- markdownlint-disable -->
# Roadmap (Idea Log)

This is an idea log, not a locked release plan.

## v0.2.0 (Tag Parsing Foundation - First Priority)

- [ ] Parse tags from standalone tag exports.
- [ ] Parse tags from gateway backups that contain tags/providers.
- [ ] Define a deterministic `TagRecord` schema and ordering.
- [ ] Add tag output artifacts:
  - `tags.json`
  - `tags.csv`
- [ ] Add parser coverage metrics for tags (`parsed`, `unknown`, `skipped`).

## v0.2.1 (Tag Config)

- [ ] Extract tag configuration fields:
  - provider
  - full path
  - tag type / data type
  - value source
  - enabled
  - history settings
  - alarm settings
  - ?
- [ ] Extract nested structures (for example UDT instances/parameters where present).
- [ ] Extract tag-level scripts/event blocks when present.

## v0.3.0 (As-Built Docs v1)

- [ ] Generate deterministic Markdown as-built output: `as_built.md`.
- [ ] Include sections for:
  - gateway settings summary
  - project summary
  - tag provider inventory
  - tag configuration tables
  - script/query inventory references
  - SFC chart rendered?
- [ ] Add CLI flag(s) for as-built generation (for example `--as-built-md`).

## v0.3.1 (Spreadsheet As-Built Output)

- [ ] Generate deterministic Excel workbook: `as_built.xlsx`.
- [ ] Initial sheet set:
  - `Summary`
  - `Projects`
  - `TagProviders`
  - `Tags`
  - `NamedQueries`
  - `Scripts`
- [ ] Keep column schema stable and documented.

## v0.4.0 (Dependency Extraction)

- [ ] Extract references without visualization first (edge list model).
- [ ] Produce deterministic `references.json` with confidence metadata.
- [ ] Prioritize high-confidence edges:
  - perspective view -> script/query/path references
  - script -> named query references
  - resources -> tag path references

## v0.4.1 (Graph Outputs)

- [ ] Build graph artifacts from reference extraction:
  - `graph.json`
  - `tree.json`
- [ ] Add ECharts force-directed payload export from `graph.json`.

