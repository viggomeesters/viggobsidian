# Immutable source exports

Some application consumers bind a source file's exact path, fenced YAML records and whole-file hash. Preserve those bytes. A dated Raw package can declare `export_artifacts: source-export.json` on its ordinary event note. This does not waive the event header, provenance, processing target or Wiki checks.

The manifest uses `schema: viggobsidian.source-export.v1`, `package_note` (the exact owning event filename), a nonempty consumer slug and an `artifacts` array. Every row declares `path`, `bytes`, `sha256` and `format`. Only direct secondary `.md` files are eligible. The checker rejects escapes, symlinks, duplicate paths, changed or missing bytes, unknown formats and operational task/Wiki/processing metadata. A single invalid declaration invalidates the whole inventory; undeclared Markdown retains the normal source-note checks.

Supported source formats are explicit exporter contracts:

- `migration-records-v1`: migration-collection frontmatter with mapping version and source set; complete fenced YAML mappings with unique IDs, matching literal Bron-ID and unique record anchors. A record heading matches its ID or its exported title.
- `derived-month-summary-v1`: a migraine-derived-summary with current-derived status, month, derivation date, source summary and daily-source metric table. Its values are historical derived data, not verified clinical observations.
- `reported-health-log-v1`: a reported, dated health-log with person, source, capture timestamp, original supplied input and source-boundary sections. It is immutable provider input, distinct from the maintained event review.

This inventory describes source payloads, never a source-processing result. A pending package stays pending. An event can only be marked processed through its normal writer and precise same-owner Wiki target. Reading or hash-verifying a payload does not by itself establish that all source content has been incorporated. No arbitrary supporting-Markdown exception or consumer import is created by the checker.
