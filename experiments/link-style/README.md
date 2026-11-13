# Read-only link-style feasibility spike

Issue [#66](https://github.com/viggomeesters/viggobsidian/issues/66), supporting
[AnGo Companion #1](https://github.com/viggomeesters/obsidian-ango-companion/issues/1).
This is a source experiment, not an AnGo update, full ViggObsidian validator,
plugin registration, release or Community installation.

The branch prepares package version 0.22.0 and its exact hashed core sources for
review. It does not tag, publish or install that package into an existing vault.

The existing Work link-presentation rules now live in `crates/link-style-core`.
The native filesystem/fix adapter uses the same Rust `edits` function. Its existing
eight rule tests moved unchanged. The snapshot boundary is read-only: it accepts
Markdown strings and relative paths, never a filesystem, writer or process handle.
Only the original CLI retains its explicit optional fix behavior.

The existing Rust/Serde dependencies are reused, with no new registry packages.
JavaScript here is the required WebAssembly/Worker/Obsidian-Vault API boundary and
its platform integration tests; there is no parallel JavaScript rules engine.
The WASM artifact has zero imports. All validation runs in a separate worker,
which is terminated after a result, timeout, cancellation or error. No executable,
rule or schema is fetched or downloaded by the experiment.

## Reproduce

Use Rust 1.92.0 and Node 25.8.0 (the tested versions). Build outputs and temporary
files belong in an explicitly selected private scratch directory, outside the
source checkout. The normal workspace MSRV remains 1.89; this spike's WASM run
was verified on 1.92, not on every supported toolchain or browser.

```sh
export SPIKE_TARGET=/absolute/path/to/private-scratch/link-style-target
export TMPDIR=/absolute/path/to/private-scratch
rustup target add wasm32-unknown-unknown
cargo test --locked --workspace --target-dir "$SPIKE_TARGET"
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --target-dir "$SPIKE_TARGET" -- -D warnings
cargo build --locked -p link-style-core --bin snapshot-check --target-dir "$SPIKE_TARGET"
RUSTFLAGS='-C link-arg=--max-memory=268435456' cargo build --locked \
  -p link-style-core --lib --release --target wasm32-unknown-unknown --target-dir "$SPIKE_TARGET"
LINK_STYLE_WASM="$SPIKE_TARGET/wasm32-unknown-unknown/release/link_style_core.wasm" \
LINK_STYLE_NATIVE="$SPIKE_TARGET/debug/snapshot-check" \
  node --test experiments/link-style/worker-parity.test.mjs
```

The native integration test compares the CLI's entire ordered report with the
snapshot result, including path, line, kind, old/new text, inventory and template
counts. Fixtures cover ambiguous/missing links, Unicode, CRLF, fenced/inline code,
quoted and Raw evidence, Wiki metadata and template exclusion. Node tests execute
the actual WASM inside a worker using a narrow browser-message API shim, compare
six native/WASM results, then exercise synthetic Vault reads/events and cleanup.
The separate real Obsidian browser-worker observation is recorded in `evidence.json`.
No production-vault data is part of fixtures or public evidence.

For a coordinated real-host rerun, load the reviewed `vault-adapter.mjs` and
`host-proof.mjs` as local modules in the designated Obsidian test vault, then call
`runHostProof(app, validateVaultSnapshot, workerSource, wasmBytes)`. Pass the
reviewed worker file text and built WASM bytes explicitly. The helper refuses an
existing `fixtures/viggobsidian-link-style-spike` path, creates two fictional notes,
captures only that namespace using real Vault API calls, and runs the real browser
Worker. It deletes only its recorded fixtures in `finally`, revokes the Blob URL,
and checks active-note/plugin-state preservation. It does not install a plugin.

## Boundary and limits

Request schema `viggobsidian.link-style-snapshot.v1` contains `complete: true`,
`inventory: [relativeMarkdownPath]` and `notes: [{path, markdown}]`. Optional
`targets` selects notes while retaining the complete inventory for resolution;
it does not prove that unselected notes passed. Missing/duplicate documents,
unsafe paths, unsupported schema, invalid targets and resource limits produce
`status: incomplete` without a report. Valid evaluation produces
`status: complete` plus the original CLI report. A complete result can have
findings; it is never a claim that all ViggObsidian checks passed.

Limits: 10,000 notes, 1 MiB per note, 16 MiB total Markdown, 24 MiB encoded request.
The reproduced WASM build caps linear memory at 256 MiB; traps become incomplete.
The adapter reads sequentially, checks observed inventory/stats and Vault mutation
events before and after evaluation, and releases subscriptions/workers in `finally`.
The event/stat checks detect observed changes; Obsidian's Vault API is not an atomic
filesystem snapshot. A caller's `complete` flag is an attestation, not independent
proof of inventory completeness. Capturing a folder namespace evaluates only that
declared namespace; it must not be presented as full-vault link resolution.

No UI, auto-on-save, incremental cache, other rule set, private adapter, canonical
writer, apply/rollback, ID migration or plugin publication is included. The ABI is
experimental single-worker request/response buffers, not a stable public SDK.
Severity remains absent because the original report defines kind/path/line,
not severity. A future UI must explicitly define that presentation mapping.

Before a product integration: independent review, broader snapshot/size testing,
packaging/CSP verification in supported Obsidian versions and real catalog-install
acceptance remain necessary. The real-host observation below is not a catalog test.

Official API references consulted:

- [Rust wasm32-unknown-unknown](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html)
- [Obsidian Vault API](https://github.com/obsidianmd/obsidian-api/blob/master/obsidian.d.ts)
- [Web Workers](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers)
