# Optional host skills

ViggObsidian also owns the generic `devops-work` skill for Azure Boards and Azure
Repos work. It is an additive counterpart to `github-work`; neither name is an
alias. The explicit source allowlist is [host-skills.json](../schemas/host-skills.json).
The [skill source](../skills/devops-work/SKILL.md) contains no organization,
customer, repository or account bindings. Those belong in the selected target's
local adapter.

This optional host export does not extend the ten-core vault package,
`schemas/skill-library.json`, its source-lock transaction or the vault profile
registers. Do not insert an eleventh row into those registers or edit installed
release files to install it. A later vault-package adoption requires its own
reviewed migration; this export grants no datawriter authority.

## Install a reviewed immutable export

Use the host's installed `skill-installer` skill and its
`scripts/install-skill-from-github.py` helper. Install from the merged, reviewed
full commit ID, never a moving branch. With `SKILL_INSTALLER` resolved to that
installed helper and `REVIEWED_COMMIT` set to the verified forty-character commit:

```sh
python3 "$SKILL_INSTALLER" --repo viggomeesters/viggobsidian \
  --ref "$REVIEWED_COMMIT" --path skills/devops-work --method download
```

The helper uses `$CODEX_HOME/skills` or the default `~/.codex/skills`. It aborts on
an existing destination; inspect ownership and preserve the existing bytes before
any separately authorized replacement. Use the host's private scratch directory
for the helper's temporary downloads, following its local temporary-file policy.
Installation is a source export and does not require a new engine release or a
vault updater invocation.

Record repository, full commit, source path, source file hashes and installed
file hashes in the host's existing installation evidence. Compare every installed
file with the reviewed source and the finite allowlist before calling it an
immutable installation. Keep those bytes unchanged; updates use a newly reviewed
commit and deliberate replacement, not edits in the installed directory. The
installer itself does not provide a persistent provenance registry.

Verify that the installed `devops-work/SKILL.md` exists in the actual host skill
directory and appears in the next skill-discovery turn. Existing chat context is
not proof of rediscovery; distinguish files installed from the host reporting the
new capability. The standard installer makes the skill available on the next
turn. No Azure operation is performed by installation.

## Validation

Run the skill creator's `quick_validate.py skills/devops-work` and the repository's
locked Rust tests, formatting and Clippy checks. The repository tests validate
the source allowlist, skill identity and separation from the core package.
Behavioral review should exercise a project with several repositories, an
existing parent/task, an uncertain create response, a plan-only request and a
merge that does not satisfy deployment acceptance, using fictional artifacts and
without live Azure writes.
