# Structure capability experiment v1

Engine owner: [#92](https://github.com/viggomeesters/viggobsidian/issues/92).
Product design: [ViggVault #7](https://github.com/viggomeesters/obsidian-vigg-vault/issues/7).
The product's capability matrix lives in
[its design](https://github.com/viggomeesters/obsidian-vigg-vault/blob/main/docs/structure-capabilities.md).

The first executable slice is **matching entrance presence for explicitly chosen
visible folders**. `structure-core` holds the two existing `Scan::note`
`folder-note` diagnostics once. The native adapter still obtains regular-file,
missing and non-file facts with `symlink_metadata`; only these two diagnostic
literals move. Discovery, exceptions, YAML, folder traversal, symlinks, profiles,
writers and other findings retain their existing behavior. A portable positive
result certifies only the named visible-inventory capability, never a regular
filesystem file or the full native layout check.

This adds a small Rust rlib/cdylib and a bounded JSON proof CLI. It adds no
ViggVault command, vendored payload, automatic scan, helper, downloader, provider,
write or Actions workflow. The existing production native adapter does change
to share diagnostics; a normal reviewed engine release is required to distribute
that change. Its four bounded path-dependency files (manifest, library, WASM
source and fictional regression fixture) are in the finite package allowlist so
the installed native crate still builds offline. The prototype WASM binary and
proof CLI are not distributed to a vault or plugin.

## Input and result

`crates/structure-core/fixtures/dossier.json` freezes one fictional dossier and
the existing diagnostic codes/messages against base
`52d5dd2780ab539aa0ad87cd9f4aa1d92959e530`. A Rust filesystem test creates precisely
these visible facts, calls the real native `Scan::note` and compares them with
the fixture and shared snapshot result. It also verifies native rejection of a
symlink on Unix; that is deliberately absent from the portable claim.

Input `viggobsidian.structure-snapshot.v1` has exactly this required contract:

| Field | Meaning |
| --- | --- |
| `ruleset` | `visible-folder-entrance.v1`; no implied extra rules. |
| `profile` | Explicit `numbered-v1`; another selected profile is not applicable. |
| `capabilities` | Exactly `["visible-folder-entrance.v1"]`; not a filesystem attestation. |
| `complete` | `true` attests complete visible entries **inside this scope**, including non-Markdown files and empty folders. Core cannot discover omitted entries. |
| `capture` | Nonempty bounded `before` and identical `after` generation token. Host must detect changes until result delivery, not just while enumerating. |
| `scope` | Visible relative folder; root-wide scope deliberately unsupported. |
| `entries` | Unique `{path, kind}` pairs; kind `file` or `folder`. Includes scope and every visible ancestor inside scope. |
| `folders` | Nonempty unique explicit list of folders to check; no inference that all inventory folders should follow this policy. |

Paths use literal `/` separators, are case-sensitive, preserve Unicode without
case folding or normalization, and reject backslash, colon, NUL, empty/dot-leading
components and more than 1024 UTF-8 bytes. This is a narrow portable contract;
native OS pathname behavior is unchanged. A literal backslash filename cannot
be encoded here and must produce Incomplete in a future adapter, never be
silently rewritten. Scope, parent/kind conflicts, unknown kinds and duplicate
paths/selections are rejected. `foo` does not include sibling `foobar`.

For folder `fiction/raw`, the expected entrance is `fiction/raw/raw.md`.
An absent entry produces the existing `folder-note` missing message; a folder
at that exact path produces the existing non-regular message. A visible file
at the path produces no finding under this capability. It proves neither
frontmatter nor symlink absence nor policy eligibility. Findings are sorted by
selected folder path; no location or severity is invented.

Result `viggobsidian.structure-result.v1` separates:

- `complete`, with `capabilities`, `scope`, `checked_folders` and `findings`.
  Findings may be nonempty; an empty list is clean only for this named slice.
- `not_applicable`, reason `profile_not_selected`.
- `incomplete`, a reason and **no findings/clean report**. Unsupported versions,
  missing required capture, drift, invalid paths/inventory and exhausted budgets
  all take this route. The native proof CLI exits 2 for Incomplete.

## Capture design; no atomicity claim

A future Obsidian adapter must subscribe to create/delete/rename/modify events
before enumeration; retain a monotonic generation for the entire run; recursively
copy `TFolder.children` paths/kinds/parent relations into immutable data; reject
unrecognized entries, disagreement or unreadable scope; enumerate again and
compare an exact sorted inventory; and check the event generation again before
and after Worker completion. Changes or unload invalidate pending results.
Do not use a cached Markdown list: it omits empty folders/assets. Metadata cache
link resolution is not native resolution. Even double enumeration and event
checks are not an atomic filesystem snapshot; results describe the captured
visible view and cannot upgrade to native guarantees.

The core validates the host's asserted generation equality, not real host events.
The experiment uses a real Node Worker with a changing generation callback,
deadline, AbortSignal and actual awaited termination. Abort triggered by a
simulated unload proves the lifecycle mechanism, not Obsidian's `onunload` hook.
That integration, future browser Workers, pop-outs and host races remain
separate product acceptance before any command ships.

## Explicit budgets and proof costs

| Budget | Enforcement / evidence |
| --- | --- |
| Request 1 MiB UTF-8 JSON | Rust entrypoint and ABI reject before allocation; native CLI reads at most limit + 1. |
| 10,000 visible entries; 1,000 chosen folders | Rust checks before inventory/indexing; maximum admitted fixture tests both together. |
| 100 findings | More returns Incomplete, never truncated success. |
| Path 1024 bytes; capture token 128 bytes | Validated before indexed use. |
| 256 MiB WASM linear memory | Build `--max-memory=268435456`; real runtime grows to limit, rejects another page. |
| 10 seconds total host deadline | Prototype defaults to 10 s; injected 25 ms deadline proves stalled Worker termination. Future capture must share this single deadline. |
| Cancellation / unload | AbortSignal, no delayed success, awaited `Worker.terminate()`; Node prototype only. |
| 64 MiB JS old generation per test Worker | Node resource limit, **not** a total host RSS/browser memory guarantee. |

Inventory parsing/indexing costs O(request bytes + N log N + F log N), with a
BTreeMap/BTreeSet and owned JSON request plus output. No file contents, YAML,
hashing, async runtime or broad rule engine is loaded. The maximum-input
release-native/real-WASM parity test must finish below 10 seconds; it prints the
observed local duration. This is a bounded local proof, not a general performance
claim or full host memory bound. WASM memory covers linear allocations only;
captured JS objects, JSON serialization, message copies and browser heap need
separate measurement in a later plugin adapter. No wide-vault proof was run.

## Language/dependency gate and local reproduction

New core, CLI, filesystem fixtures and native tests are Rust. No registry
dependency versions/features are added: existing locked `serde_json` is reused.
`experiments/structure-parity.test.mjs` is the required Node WebAssembly/Worker
platform interface, including lifecycle failure injection and input transport.
It contains no structure rules. A second Rust WASM runtime would add unnecessary
dependencies. Toolchain remains repository-pinned Rust 1.92.0; lockfile change
only records the new local crate and its use by `vault-layout`.

Set `CARGO_TARGET_DIR` and `TMPDIR` to an existing private scratch directory before
all tests; the filesystem proof creates and removes fictional files there.

```sh
cargo test --workspace --locked -- --test-threads=1
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo audit
cargo build --locked -p structure-core --bin structure-snapshot-check --release
RUSTFLAGS='-C link-arg=--max-memory=268435456' cargo build --locked -p structure-core --lib --release --target wasm32-unknown-unknown
STRUCTURE_NATIVE="$CARGO_TARGET_DIR/release/structure-snapshot-check" STRUCTURE_WASM="$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/structure_core.wasm" node --test experiments/structure-parity.test.mjs
```

Proven platform scope is only the recorded local macOS native toolchain and Node
WebAssembly/Worker runtime. No Windows, Linux, minimum Obsidian version,
browser Worker, pop-out or mobile claim follows. Product #7 owns the capability
matrix; separate small owner issues must precede capture/product integration
or extraction of another rule family.

The bounded backlog is now owned by
[ViggVault #9](https://github.com/viggomeesters/obsidian-vigg-vault/issues/9)
(visible capture and browser Worker lifecycle),
[engine #93](https://github.com/viggomeesters/viggobsidian/issues/93)
(project-code layout and explicitly scoped registry facts), and
[engine #94](https://github.com/viggomeesters/viggobsidian/issues/94)
(Wiki/Raw shortcuts and typed resolution). These issues do not authorize or
start their implementation; this proof closes neither of their acceptance gates.

Recorded local validation on 13 November 2026: macOS 26.6.2 arm64, Node 25.8.0,
Rust 1.92.0. All 245 workspace Rust tests passed serially, including the actual
offline installed-crate build with its separate lockfile. All five real
native/WASM/Worker platform tests passed, including abort and drift during awaited
Worker termination. The 10,000-entry / 1,000-selection release-native plus WASM
evaluation took about 29 ms locally against a 10-second budget; this includes
native process launch and comparison, not real Obsidian capture. The actual
WASM memory-growth boundary rejected a page beyond 256 MiB. Fmt, Clippy with
warnings denied, cargo-audit and the repository diff Rust-policy gate passed.
Independent review found a cleanup race in the prototype; post-termination
freshness checks and two discriminating regression cases repaired it, and
independent rerun passed. No UI test, user note/settings mutation, installed
package update or cross-platform claim was made.
