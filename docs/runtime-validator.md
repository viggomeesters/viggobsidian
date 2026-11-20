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

Existing plan and recovery pins remain unchanged. Before changing a local
writer that builds its own validator, implement and review its version-aware
activation/recovery contract separately. Existing navigation releases are
not a generic migration mechanism for other providers. Issue [#137](https://github.com/viggomeesters/viggobsidian/issues/137)
tracks the remaining distribution and adapter work.

Local checks: `cargo test --locked --bin vault-validator` and
`cargo clippy --locked --bin vault-validator -- -D warnings`. Use an explicit
bounded `CARGO_TARGET_DIR` outside vaults, disable incremental compilation where
appropriate, and remove owned build/test output after the last consumer.
