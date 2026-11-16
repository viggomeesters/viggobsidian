# Read-only dossier retrieval and one-command ask

`dossier-retrieve` is a native Rust proof of concept for finding evidence in one
existing Wiki/Raw dossier. It is intentionally lexical retrieval, **not** a
generative model and not a semantic consistency checker. It supplies ranked,
readable excerpts so an agent can formulate a separately grounded answer.

## Managed one-command ask

Use either native entrypoint for the same strict route:

```sh
# Standalone retrieval package binary
dossier-retrieve ask --vault /absolute/vault --query 'Welke fictieve garantie geldt?'

# Canonical ViggObsidian native CLI
viggobsidian dossier-ask --vault /absolute/vault --query 'Welke fictieve garantie geldt?'
```

Both call the same Rust parser and managed route. `--vault` and `--query` are
mandatory. The optional selection hints are `--dossier RELATIVE/PATH`,
`--context-path RELATIVE/PATH` and `--topic LABEL`; they narrow selection but
never make an ambiguous owner authoritative. The optional `--cache-dir ABS`
selects an absolute private cache root; otherwise the route uses
`$XDG_CACHE_HOME/viggobsidian/dossier-retrieval` or
`$HOME/.cache/viggobsidian/dossier-retrieval`. If neither environment variable
can establish an absolute user-local location, the command refuses rather than
choosing a repository, current-directory or vault fallback. `--limit` is 1–20
and defaults to 8. Unknown, duplicate and positional arguments are refused.

The JSON response uses `dossier-ask.v1`. It identifies the selected owner and
cache outcome and returns only one of the managed outcomes: `complete`,
`needs_selection`, `not_applicable`, `no_evidence`, `refused` or `busy`.
Evidence is lexical and cited; before making a factual synthesis, read the
original Raw source for every material claim. A selection failure asks for an
explicit owner/context rather than scanning another dossier or falling back to
the current directory.

This command is read-only for the selected vault. Its cache is private derived
data, not a source of authority: do not commit, sync or publish it. It makes no
network or model call and does not process sources, write Wiki notes, or invoke
providers. Managed cache handling still rejects unsafe locations and preserves
vault/repository boundaries.

Selection first uses an explicit eligible dossier, then the nearest eligible
owner of an existing context path. Otherwise discovery is bounded to visible
numbered collections. A unique lexical topic/title match may select an owner
only after complete discovery. Budget exhaustion returns `needs_selection` with
`discovery_complete: false`, never a match from an incomplete candidate set.
Use the known dossier/context for large vaults; no full-vault background crawl
is started. Ordinary folders are never converted. Besides real `wiki/` plus
`raw/`, a direct-Wiki foldernote is eligible only at its exact same-name path
with both `layer: wiki` and array class `vault-wiki-direct`, plus real `raw/`.

The managed cache creates missing nested application directories privately,
without changing existing broad HOME/XDG parent permissions. Source edits,
additions, deletions and corrupt derived state trigger one bounded atomic
rebuild. Unsafe paths, permissions or source-identity errors do not. A busy
cache lock returns a retryable state without stale evidence or lock stealing.
This read route can run repeatedly for every eligible dossier; it needs no
per-dossier configuration or manual build command.


## Boundary and safety

- `--vault` selects the vault explicitly and `--dossier` is an explicit,
  vault-relative dossier path.
- The command never writes in the vault. `build` writes one disposable JSON
  index at an absolute path outside both the vault and this repository.
- It rejects `..`, backslashes, symlinks anywhere in the selected dossier, hidden
  files, oversized files and excessive file counts. It indexes Markdown under
  the dossier's `wiki` and `raw` folders and an explicitly declared direct-Wiki
  same-name foldernote.
- Each document gets a SHA-256 identity. `query` validates index schema and
  integrity, rescans the complete eligible Wiki/Raw inventory, reconstructs
  every document record from live source bytes, and rejects additions,
  removals, byte drift or altered cached derived fields. Results are generated
  from those reconstructed records; rebuild after any rejection.
- A Raw `source_file` requires a valid `source_sha256`, and the file is read
  securely and hash-checked at build and query time. A valid hash without a
  `source_file` is retained only as declared metadata: it remains searchable
  with an explicit unresolved-artifact provenance warning and is not presented
  as artifact verification.
- Raw `wiki_status`, `wiki_target`, Wiki-to-Raw links, target kind and target
  anchor resolution are reported conservatively as source warnings. A missing,
  unprocessed, unlinked, non-Wiki, missing-anchor or ambiguous-anchor source is
  never silently treated as verified evidence.
- Fenced text, comments and navigation boilerplate are excluded. Files whose
  names or metadata indicate history, snapshots, archives, changelogs or logs
  stay searchable but are substantially down-ranked.
- Dutch query handling is generic lowercasing, tokenization, stop-word removal,
  small suffix/compound normalization and whole-token matching. Multi-topic
  queries require more than one matching topic; Wiki sections are preferred and
  linked Raw notes remain available as traceable evidence. Navigation/activity
  sections and overlapping copies are suppressed.
- Credential-seeking queries that match a bounded set of common password, token,
  PIN, bearer-token and API/access/private-key patterns are refused with no
  excerpts. This is query-level protection, not general redaction: callers must
  not treat unrelated result text as scrubbed of credential-like values.

## Commands

Use a private scratch directory outside the vault/repository:

```sh
export TMPDIR=/home/viggo/.hermes/cache/scratch
export CARGO_TARGET_DIR=/home/viggo/.hermes/cache/scratch/viggobsidian-rag-target
cargo run --locked -p dossier-retrieval-core --bin dossier-retrieve -- \
  build --vault /absolute/vault --dossier 30-topics/example-dossier \
  --index /home/viggo/.hermes/cache/scratch/example-dossier-index.json

cargo run --locked -p dossier-retrieval-core --bin dossier-retrieve -- \
  query --vault /absolute/vault --dossier 30-topics/example-dossier \
  --index /home/viggo/.hermes/cache/scratch/example-dossier-index.json \
  --query 'Welke tekst noemt de garantie en welke onzekerheid blijft er?' --limit 8
```

Each result has the exact vault-relative Markdown `path`, section name, original
body line range, source SHA-256 and excerpt. `uncertainty_evidence` contains
only matching lines that use uncertainty language; it is evidence for the
caller, not an inferred fact. `source_warnings` identifies warnings for a
returned Raw excerpt; `provenance_warnings` carries all dossier-wide broken,
missing or unprocessed Raw↔Wiki signals even when the matching excerpt is Wiki.

## Deliberate limitations

The index is a disposable convenience cache, not an authority. The PoC does not
parse every Markdown dialect, inspect binary originals, prove a Wiki statement
contradicts another statement, decide source truth, or replace native vault
layout validation. It makes no network/model calls and uploads nothing.

The disposable index contains private source text. Keep its directory private
(e.g. mode 0700 on Unix), do not commit/sync/publish it, and apply the same
handling to query output. Query refusal is not a data-loss-prevention boundary.
Missing backlink diagnostics are structural hints, not proof that Wiki content
is wrong or that a Raw item was never processed. Links outside the selected
scope are not fetched automatically. Heading selection is deliberately a
bounded Markdown parser; use the original source when more context is needed.

## Implementation and verification

Issue #103 owns this bounded proof. New engine, CLI and tests are Rust. The
workspace's existing locked serde_json, sha2 and serde-saphyr dependencies are
reused; no vector backend, model provider or new registry dependency is added.
Disjoint heading chunks preserve pre-heading source input; Wiki locator targets
are followed as provenance rather than counted as prose search hits. Short
sections retain their full attribution/caveats; longer excerpts use a bounded
window and UTF-8-safe truncation. Returned line ranges identify the containing
original-file section, not an exact highlight of every displayed line.

Run the local gates (no GitHub Actions):

```sh
cargo test --workspace --locked -- --test-threads=1
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --release --locked -p dossier-retrieval-core --bin dossier-retrieve
cargo audit --file Cargo.lock
```

Fictional regressions cover stale inventory, source/artifact drift, rehashed
index tampering, source target kind and anchors, symlinks, history separation,
Dutch compound/cost queries, non-overlapping source sections, source line
locators, UTF-8 boundaries, and explicit credential-query refusal. A separate
private real-dossier evaluation must check actual evidence quality; fixture
tests alone do not establish useful retrieval or complete question coverage.
No plugin integration, package rollout, release or automatic Wiki writes are
included. The CLI provides the retrieval stage; an agent still formulates and
checks the final natural-language answer from the cited sources.

