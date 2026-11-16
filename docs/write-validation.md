# Transaction validation

Use `vault-layout --vault ROOT --write-set` with a JSON request on stdin for
bounded writes. Keep `--strict PATH`, `--check-all` and `--check-work` for full
audits. An unrelated sibling's existing errors must not block a transaction.

The request schema is `viggobsidian.write-set.v1`, with `phase: baseline` and
`changes: [{"path":"30-topics/example/example.md","expect":"present"}]`.
Use `absent` for deleted files. Include every created, updated and deleted file,
including both ends of moves and changed parent navigation. Paths must be
visible relative normalized file paths without symlinks. Hidden provider state
is validated separately by its owning writer.

The two supported producer manifests, `.mail-source.json` and
`.reading-list-source.json`, may be declared as regular leaf files so complete
package moves include their real write set. Neither name may be a hidden
directory. Other hidden paths remain refused. Their owning source provider must
verify the complete inventory and manifest-pinned bytes before and after a move;
this path support does not authorize manifest edits, change capture identity or
exempt ordinary Raw notes from validation.

Capture the baseline before mutation and bind the entire JSON report into the
writer's sealed plan alongside exact preimages, checker binary/hash and input
configuration pins. After mutation, submit the same changes with `phase:
validate` and `baseline` containing that report. Exit 0 means accepted, 1 means
findings, 2 means invalid input or execution failure. Any failure requires the
writer's existing rollback; never rewrite plan pins to bypass drift checks.

Each changed file owns its direct folder. Raw/Wiki changes own their complete
dossier, including source inventories and processed-source references. Ancestor
navigation is checked without recursively visiting unrelated siblings. Changed
owners receive all strict checks. Only exact unchanged ancestor findings and
explicitly adopted historical formatting findings may remain. New boundary
findings fail. Preimage owners remain in scope after a move or deletion.

## Phased dossier adoption (v0.8)

No adoption state means strict validation. `.viggobsidian/adoption.json` is local
operator state, outside the ordinary content writer's allowed write set. The
checker binds its full state and byte hash (including absence) into the sealed
baseline. The adapter must pin this file before installing any content, and
must reject drift before writing as well as validate afterwards.

Register exact existing owners through `vault-layout --adoption-plan`, providing
JSON on stdin:

```json
{"schema":"viggobsidian.adoption-request.v1","owners":{"30-topics/example":"legacy"}}
```

Review the resulting full strict scan, existing file inventory and hashes before
passing that plan on stdin to `--adoption-apply`. This administration command
updates only local adoption state and retains a preimage receipt. It does not
change notes, approve migration or grant writer authority. Keep the receipt
private. Exact successful replays return `already-applied`. After an interrupted
administration, pass the same reviewed plan to `--adoption-recover`: it checks
the receipt, exact state preimage/postimage and pending/lock bytes before cleanup
and completion. Drift or foreign recovery members remain untouched. Do not run
recovery while the original operator is still active; the lock is local.

Legacy recognition is a review aid, never an automatic exemption. The exact
owner and recorded original files establish the boundary. The deepest explicit
owner wins; unrecorded new files never inherit historical exceptions. The finite
formatting allowlist excludes source inventories, manifests, missing source
bytes, source status/processing, identities, symlinks and link target integrity.
Full strict audits are unchanged and continue to show all historical debt.

A finding is tolerated only if it already exists in the sealed baseline and
the affected file retains its registered historical bytes. Append may preserve
the immutable original prefix, but every accumulated addition must independently
pass provenance-v1, Activity, mail and delivery checks. Start appended content
with a separate dated Activity-day and named, stamped bubbles with result goals
for agent contributions. A malformed append cannot reuse a deduplicated old
finding. Arbitrary revisions receive no historical byte exception: either the
result validates fully or use an explicitly authorized migration provider.
Non-file structural debt remains blocking; register the exact legacy source
files through their existing owner rather than introducing broad skip scopes.

After source-by-source semantic migration acceptance, request `strict` for that
owner through the same preview/apply protocol. Promotion requires a clean full
strict scan. Existing legacy snapshots cannot be refreshed; strict owners cannot
be downgraded or loosened by registering a legacy child. The ordinary content
writer cannot relocate a strict owner out of its registered boundary. A separate
maintenance migration must explicitly maintain adoption identity and provenance.
Changing a note's metadata never changes this administrative state.

Reports keep `owned_findings`, separate `legacy_findings`, blocking `findings`
and the selected adoption state. Accepted legacy writes are not proof of full
ViggObsidian conformance or completed migration. Old pre-v0.8 write-set baselines
require a new plan; never add missing pins to an existing sealed plan.

This interface supplies validation, not write authority. Local adapters retain
their existing routing, locking, source registry, preimage and rollback duties.
The caller must seal the baseline: it is evidence, not a trusted user exemption.

## Typed maintenance proofs (v0.9)

Version 0.31 adds three separately checked proofs. `verified-link-repair` takes
`before`, a sibling `.md.txt` `archive`, and an exact `mapping` of WikiLink target
strings. A file destination has `kind: file`, `uri`, an explicit absolute `root`
and source `sha256`; the native adapter verifies regular files, symlinks, root
containment, exact bytes and any preserved anchor. A historical destination has
`kind: git`, a full immutable `revision`, relative `path` and source `sha256`.
Its original label/heading stays visible as a historical locator, without a fake
text-file heading fragment. It verifies the actual Git object in the selected
vault. All non-token source bytes and fenced/indented examples remain unchanged;
unsupported embeds and ambiguous inline contexts fail closed.

Every repair or append owner must be paired in the same write set with
`source-preimage-archive`: `before: null`, exact `source` owner and full
`source_before` bytes. The archive is create-only, directly beside the owner and
byte-identical. An archive alone, a changed archive, a missing pair or any other
new member receives no compatibility. The precise paired archive is a typed
source artifact; this does not exempt arbitrary sibling payloads or change full
strict audits.

`activity-append` takes a separate `before`, sibling `archive` and `append`.
It checks a real final Activity section outside code/comments and runs the
existing legacy-addition provenance, chronology, result-goal and delivery gates
on the addition. It preserves the complete previous prefix. Callers can validate
and publish link repair first and independently plan the result contribution
from the new preimage. No proof grants write authority or changes schema,
adoption, configuration, source identity or canonical writer ownership.

A canonical migration adapter may include `proof` on a present change. This is
validation only; it grants no write authority. `--maintenance-transform` reads
`{path,proof}` on stdin and returns the sole permitted postimage without writing.

`owner-handoff` takes the exact `before` text and a `target` Markdown Wiki or
matching foldernote. It inserts one fixed visible knowledge-owner notice while
retaining the entire original metadata and body. Tasks and source events are
excluded. The destination is independently checked as a strict owner. Missing
targets, another handoff and cycles fail.

`locator-rebind` takes `before` and an exact locator `mapping`. It changes whole
WikiLink targets (including basenames), local Markdown-link URIs, complete
single-backtick file locators and declared `external-tasks[].path` bindings.
WikiLinks may also target a visible vault-root `.base` filename containing only
ASCII letters, digits, dots, hyphens and underscores. This exception applies only
to WikiLink destinations; inline basenames and metadata retain their existing
grammar. The native adapter checks the same-vault regular file and rejects
symlinks. Root files do not become writable through this destination support.
All other metadata, bound task IDs/status, labels, anchors, free prose, quotes,
HTML source and code examples remain immutable. A null local-URI mapping renders
fixed historical-unavailable text retaining the original URI. Markdown denotes
the link syntax; PDF, EML and CSV source targets are valid. Complex multiline
inline-code transitions fail closed. Raw, imported and provider sources are
excluded; a maintained source-index is permitted. See navigation.md for the
bounded locator grammar; this is no general rewrite or metadata waiver.

For an owner containing these deterministic transformations, the baseline seals
every other unchanged file's hash and preserves only identical old findings on
unchanged files or exact proofnotes. Other changed files remain fully strict,
including new navigation companions. This does not refresh adoption
snapshots, suppress full audits or permit clinical rewriting through a handoff.
Adapters still own authorization, original-byte archives, registry CAS, locks,
rollback and source-preserving moves. They must include all files in the exact
write set and must reject source/configuration drift before installation.
