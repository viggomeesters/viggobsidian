# Installed runtime validator

`vault-validator` runs an explicitly installed executable without invoking Cargo,
downloading a runtime, or creating a target tree in a vault. This opt-in CLI is
source only until its reviewed executable and a deployment-local binding have
been installed. It does not replace existing writer providers or migrate plans.

An installer first builds and validates a validator outside the vault. It then
supplies the verified installed-package digest and executable digest explicitly:

```sh
vault-validator bind /canonical/vault /canonical/release/vault-layout INSTALLED_SHA256 BINARY_SHA256 /canonical/release/binding.json
```

The destination must be new, outside the selected vault, and beneath an existing
canonical directory. The command prints the binding's SHA-256 digest. Keep that
digest in the launcher or installer receipt, separately from the binding file:

```sh
vault-validator run /canonical/release/binding.json BINDING_SHA256 -- --help
```

The binding includes the exact installed metadata digest, package version and
source revision, managed tool-source hashes, executable path and digest, and
current host architecture/OS. Binding
refuses source drift from the installed manifest, missing validator sources,
symlinks and non-executables. Every execution checks the complete binding again.
Missing or changed inputs fail closed; there is no producer-build fallback.
Explicit input digests are the installer's trust anchors. This does not prove
reproducible compilation or establish that an arbitrary executable was built
from the selected sources.

The vault in the binding selects package sources to verify. Arguments are passed
through unchanged and may select another data vault with `--vault`; this binding
does not grant or enforce target-vault authorization. Each caller retains its
existing data-access and writer boundaries.

Execution copies at most 128 MiB into a private directory under the host temporary
root, verifies those bytes, and executes that copy with inherited standard IO and
the supplied arguments. This prevents changes to the installed executable path
after validation from substituting the executed bytes. The host temporary root
must be outside the vault. Normal success and error returns remove the copy;
an uncatchable process termination can leave that explicitly named temporary
directory for subsequent owner-aware cleanup. This CLI imposes no retention
policy on unrelated temporary files and cannot alone guarantee a whole-vault
500 MB temporary-directory limit.

## Deployment-owned providers

`bind-provider` authenticates an explicitly installed local Rust provider without
claiming that its source belongs to the generic package. The installer supplies
a source manifest with this shape (all values here are fictional):

```json
{
  "schema": "viggobsidian.provider-source.v1",
  "root": "/canonical/release/source",
  "tool": "example-provider",
  "version": "0.1.0",
  "revision": "reviewed-source-revision",
  "tree": "providers/example",
  "extra_sources": ["shared/common.rs"],
  "files": {
    "providers/example/Cargo.toml": "<sha256>",
    "providers/example/Cargo.lock": "<sha256>",
    "providers/example/src/main.rs": "<sha256>",
    "shared/common.rs": "<sha256>"
  }
}
```

The complete declared crate tree and optional nonoverlapping extra source trees
or files must match the digest map. Extra source roots are limited to 64, total
visited entries (including empty directories) to 65,536, depth to 64 components,
and aggregate source bytes to 128 MiB. Traversal streams directory entries;
symlinks and special files are refused. The installer owns the completeness of
the compilation input closure, including explicitly declared shared modules;
this CLI does not parse Rust imports or prove reproducible compilation.

```sh
vault-validator bind-provider SOURCE_MANIFEST MANIFEST_SHA BINARY BINARY_SHA OUTPUT
vault-validator run-provider OUTPUT BINDING_SHA -- PROVIDER_ARGS...
```

The source manifest and executable must be outside the source root. Output is a
new private file outside that root. The separate provider binding records the
source manifest, source identity/version/revision, binary digest and host, and
revalidates them on every run before executing the same private snapshot used
by package-bound validators. Explicit manifest/binary digests remain installer
trust anchors. There is no Cargo or download fallback.

Install versioned immutable source snapshots and retain releases needed by old
plans. Binding to mutable live sources would deliberately refuse after drift;
it is not an old-plan migration mechanism. A local adapter must explicitly set
a private temporary directory outside **both** source and target data vaults,
preserve writer authorization and old plan/journal contracts, and review atomic
activation and rollback separately. Provider arguments can select a data root:
the source binding grants no data-write authority. Generic package updates do
not automatically update these local provider releases or their bindings.

Existing plan and recovery pins remain unchanged. Before changing a local
writer that builds its own validator, implement and review its version-aware
activation/recovery contract separately. Existing navigation releases are
not a generic migration mechanism for other providers. Issue [#137](https://github.com/viggomeesters/viggobsidian/issues/137)
tracks the remaining distribution and adapter work.

Local checks: `cargo test --locked --bin vault-validator` and
`cargo clippy --locked --bin vault-validator -- -D warnings`. Use an explicit
bounded `CARGO_TARGET_DIR` outside vaults, disable incremental compilation where
appropriate, and remove owned build/test output after the last consumer.
