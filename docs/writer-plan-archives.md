# Completed canonical-writer plan archives

`writer-plan-archive` is an explicit native maintenance command for the existing
host-local canonical Python writer's plan format (version 1). It does not write
vault content, replace a provider, change proof pins, run a scheduler, or impose
an automatic retention policy. Use it only with authorization from the actual
writer owner. Receipts, journals, layout ledgers, locks and validator binaries
remain at their original paths.

```sh
cargo build --release --locked --bin writer-plan-archive
writer-plan-archive preview /absolute/vault /absolute/writer-state-base 14
writer-plan-archive archive /absolute/vault /absolute/writer-state-base 14
writer-plan-archive restore /absolute/vault /absolute/writer-state-base RUN_ID
```

The state base is the writer's **actual** `VAULT_SKILL_STATE` value or its default
`~/.local/state/vault-skills`, supplied explicitly as an absolute path. The tool
never reads or guesses a private provider configuration. It uses the writer's
SHA-256(root UTF-8) first-16-hex directory identity. The supplied root must be
canonical and contain `AGENTS.md`; symlink components are refused. `/usr/bin/gzip`
is the required existing host compressor on macOS/Linux. There is no shell
execution or package installation. Own logic and tests are Rust; pinned `libc`
provides the platform's no-follow file-open flag.

Preview reads optimistically after releasing the writer lock; it is advisory and
may become stale. Archive checks candidates again while holding the writer's
existing nonblocking OS `flock` on `writer.lock`. It releases that lock after each
individual plan, before taking it for the next. Lock contention immediately
refuses instead of waiting or breaking another owner's lock. Both plan and
receipt must be at least the requested number of whole 24-hour days old; the
explicit command defaults to 14. A global unfinished `*.journal.json` refuses
maintenance. Plans without a matching committed/written receipt, plans with a
layout recovery ledger, invalid seals, missing receipt output keys or unsupported
format/numeric values remain untouched. The per-file decompression/read limit is
512 MiB. Preview reports refusal reasons per skipped plan.

Archive preserves the original plan bytes as one `.plan.json.gz` plus a small
JSON manifest in `completed-plan-archives-v1` inside that same host-local state
directory. The manifest binds root, run ID, original byte count, original SHA-256,
writer plan seal and exact receipt byte hash. The receipt remains uncompressed.
The compressed file and manifest are independently fsynced and published without
replacement; their parent directory is fsynced. The tool decompresses and checks
the archive and receipt, rereads the original, and checks its file identity before
unlinking only the original `.plan.json`. An interruption leaves the original,
the durable verified archive, or both. Retrying verifies existing final archive
files; conflicting/corrupt finals refuse without removing the original.

Exact tool scratch names (`.RUN_ID.gzip-stage`, `RUN_ID.publish-stage`, and
`RUN_ID.plan.publish-stage`) can be discarded and regenerated only while holding
the writer lock, after regular-file/no-follow/identity checks. No lock is ever
unlinked. Other state files and scratch names are not collected. A crash after
restore publication may leave both the original and scratch; retry removes only
the scratch and refuses to replace the existing original.

Restore verifies all archive bindings and the still-present original receipt,
then publishes the original bytes at `RUN_ID.plan.json` using an atomic hard link
that refuses every existing target. It retains the archive. Restoring does not
apply or reseal a plan, bypass an old proof pin, or guarantee that an old plan
still passes the writer's current apply/inspection checks. To inspect, first
restore and then use the existing writer's supported inspect command. Changed
receipt bytes require owner investigation; hashes must never be refreshed to
force acceptance.

This follows the repository's trusted-local-user/cooperating-process boundary.
The shared flock coordinates cooperating writers and maintenance commands; it
is not a sandbox against a malicious same-user process replacing parent
directories concurrently. No-follow opens and FD/path identity checks detect
ordinary symlink/file substitution, but do not claim openat-based hostile-race
isolation. Preserve the host-local state directory's existing private ownership
and permissions. Stop via interrupt/termination if needed; retry the same command
under the lock after the interrupted process has exited. Already archived plans
remain valid and are absent from the next candidate inventory.

This standalone binary is distributed from an explicitly reviewed source
revision, with source commit, source hash and binary hash recorded by the host.
It is outside the vault package manifest and must not be copied over immutable
release assets. A source checkout or successful build alone does not establish a
published release or an installed host binary. Build output and disposable test
fixtures must be removed once required compact evidence and the installed binary
have been retained.

Local validation: `cargo test --locked --bin writer-plan-archive`, targeted
Clippy with `-D warnings`, and `rustfmt --check`. Tests cover actual OS flock
compatibility, terminal evidence, retention, recovery refusals, exact round trips,
corruption, no-clobber restores, symlinks and interrupted scratch/final
publication. No GitHub Actions are used.
