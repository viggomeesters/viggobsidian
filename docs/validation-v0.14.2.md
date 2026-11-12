# Installed procedure tests — v0.14.2

Issue #35 is a test-portability defect: four real procedure-preflight CLI tests
looked for the source repository's distribution.json above an installed crate.
The missing-file failure was reproduced with v0.13.0 and v0.14.1 outside a source
checkout. It did not represent a production procedure-preflight refusal.

The fixture loader now includes a finite snapshot of actual public procedure
assets. It creates the same isolated fictional vault, seals actual bytes/hashes
and calls the actual checker binary. Target selection, root binding, provider
availability, source adapters, missing files, symlinks, path escapes, descriptor
drift and unknown-transform refusal assertions remain unchanged. There is no
live-vault reader, host path, provider fallback, test skip or production change.

The root regression rejects duplicate keys and binds the exact target set,
source mapping and content to the current public procedure surface. The finite
updater allowlist and distribution include the snapshot as a regular crate-local
fixture. Regenerate after an intentional public procedure change:

```sh
cargo run --quiet --locked --bin procedure-test-fixture > crates/vault-layout/fixtures/procedure-assets.json
```

This generator reads only the repository's finite public asset list and writes
JSON to stdout. It never reads an installed vault or its private provider state.
No new dependencies, runtime services or GitHub Actions are introduced.

Acceptance requires both the workspace suite and the complete installed engine
suite in each selected target, plus pinned update previews, drift controls,
recovery receipts and independent review. Existing vault content findings remain
separate from code/test acceptance and must not be waived to make a release green.

Local verification before publication: 123 workspace tests (65 engine, 30 CLI
integration, 28 updater), including a concrete fixture-asset preview/drift/update/
rollback test. A dereferenced crate copy outside the source checkout passes all
65 engine and 30 integration cases. Fmt, Clippy with warnings denied and the
Rust source-policy check pass. Cargo audit uses the existing local database
(1471 advisories, 39 locked dependencies); no dependency changed.

One unrelated busy-lock intake test failed in an earlier parallel run; its
targeted rerun and the subsequent complete workspace runs passed. No assertion
or production lock behavior was altered to obtain a pass.
