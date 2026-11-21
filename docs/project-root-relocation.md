# Project root relocation: adoption handoff

This is an operator-only, local Unix maintenance capability for
`15-projects` → `20-projects`, in an existing `numbered-v1` vault with
explicit adoption owners under the source root. It
rebinds the existing adoption proofs and removes only the source entry from
`integration.json`'s legacy project roots. It does **not** constitute a complete
vault cutover: the Obsidian host owns the link-aware content move; each local
provider owns its registrations, leases and routing; the package updater owns
derived appearance. The CLI does not execute or authenticate those steps.
There is no second profile and no new writer authority.
The engine and the local registry have separate locks: this handoff alone
cannot block an active or newly starting host transaction, even when the
engine write-set preflight refuses operations. This foundation is not a
supported production cutover without a separately implemented and tested
host hold, registry rebinding and link-aware rehearsal.

## Before starting

1. Choose the exact vault and verify the name and path in the running Obsidian
   instance. Plan and rehearse with fictional files. Keep the reviewed plan and
a recoverable snapshot private, including config and adoption preimages and
   any affected link sources. Quiesce ordinary app edits, sync/other instances,
   local writers and readers that depend on the affected paths. A local advisory
   lock cannot pause them. Drain active plans, claims and pending operations;
   never dismiss them by deleting a lock.
2. Require a local, canonical registry/provider maintenance plan before moving
   registered paths: enumerate exact affected items and memberships, preserve
   IDs and historical aliases, reserve target aliases, reject pending work and
   conflicting claims, and rebind locators transactionally with repeatable
   recovery. Include paths in active source/task bindings and pending plans.
   A registry refresh may create a new identity; do not use it for relocation.
   Do not edit a registry database with this CLI. Without this host capability,
   do not begin a migration of registered data.
3. Inventory path-pinned consumers (including content-independent Bases,
   absolute Finder URLs, scripts, local adapters and appearance selectors).
   Review exact old/new bindings separately from historical evidence. Where
   old and new selectors already exist with different styles, choose their
   winning semantics explicitly rather than globally replacing text. Do not
   change managed CSS or installed state by hand.
4. Resolve and record incoming links, headings and embeds in Obsidian before the
   move, and check the actual link-update setting. A move is eligible only if
   the resulting project tree will be byte-identical, and other registered
   legacy files will not change. This includes the bytes of the root note
   whose basename changes. Nonqualifying link rewrites need a separate,
   provenance-preserving migration with its own typed proof; there is no
   force mode or rebaseline workaround.

## Plan, move and verify

Pass this exact request to `vault-layout --vault ABSOLUTE_ROOT
--project-relocation plan` on stdin:

```json
{"schema":"viggobsidian.project-relocation-request.v1","source":"15-projects"}
```

Save the returned JSON outside managed collections. Review the complete file
and directory inventory (including hidden files), project/component route IDs
against the global code resolver,
adoption preimage hash and path-only postimage with unchanged file proofs.
Then pass that JSON to `--project-relocation begin`, always with the same
explicit absolute vault. It re-plans under the exclusive gate; a target root,
case-equivalent name, changed inventory, unfinished package transaction,
adopt/update lock or different active plan blocks it. The durable pending journal blocks ordinary write-set and
adoption CLI operations, task creation/staging, link-style edits and package
applies/rollbacks using this version until the handoff is complete. It does not substitute for the
external provider hold or protect older software.

After `prepared-for-obsidian`, in that verified running vault, use Obsidian
FileManager to rename the folder to `20-projects`, then use FileManager again
to rename `20-projects/15-projects.md` to `20-projects/20-projects.md`.
Use the host's link-aware app route, such as `app.fileManager.renameFile` on
the exact resolved folder/note. Check expected type and destination absence
before each rename, await each operation, then wait for reindexing and verify
that the same references still resolve, including heading/embed targets.
Never use shell rename or copy-and-delete for live moves. If the Obsidian route
is unavailable, stop; there is no filesystem fallback.

Pass the same plan to `--project-relocation validate`. This verifies the root
and root-note moves, identical moved inventory and project routes, unchanged
external adopted legacy files, and pinned maintenance metadata; it does not
verify Obsidian's backlink index or host registry receipts. Complete the
separately authorized host registry rebind and check identity, lifecycle,
current aliases and receipt readback. Only after this and the link checks
pass, use `--project-relocation apply`. It atomically replaces each of the
two local JSON files, preserving their Unix permission bits, reads back both,
and finalizes the private (0600) journal as a receipt. Unix permissions are
not Git or sync protection: confirm the plan, journal and receipt are outside
those publication paths or explicitly excluded before beginning.
An individual file write is atomic; the two writes and Obsidian operation
together are **not** an atomic filesystem transaction.

Keep consumers paused until the host checks pass: verify project/component
code resolution, active task/source lookups, fresh write-set previews for mixed
legacy and strict owners, and the updated integration profile. Where local
appearance changes are needed, preserve unrelated routes and declarations;
plan/apply the generated snippet through the existing package updater, and
verify the derived hash and actual selectors before resuming affected
consumers. Keep prior adoption receipts, historical paths and source evidence
unchanged. Pre-cutover write-set plans must be regenerated; they are pinned to
previous paths and adoption state.

## Recovery and refusal

| Observed state | Action with the original reviewed plan |
| --- | --- |
| Source tree and metadata intact; host made no changes | `abort` releases the engine hold. Clear any external reservation through its own authority. |
| Folder moved, root note not renamed | Finish the remaining rename through FileManager, or undo through the app and verify the original state before aborting. The engine refuses apply/abort while partial. |
| Host registry/provider rebinding partially complete | Keep both maintenance holds. The canonical provider alone may resume or undo its transaction. Do not run engine `apply` or `recover` before verifying its completed receipt; abort only after verifying that the host and content were restored to their preimages. |
| Exact destination tree; both metadata files still at preimage | Verify host state and links; `recover` completes the handoff just like apply. |
| Exact destination tree; adoption replaced, integration still at preimage | Verify host state and use `recover`; it writes only the missing integration image. |
| Exact destination and metadata postimages; receipt finalized | An immediate retry returns `already-applied`. Do not replay completed plans after normal changes (including the subsequent appearance update). |
| Unknown drift, unexpected link rewrite, mismatched journal or conflicting operations | Keep all affected consumers paused. Inspect the saved snapshot, original plan, registry/Obsidian evidence and exact differences. Do not create a new adoption baseline, overwrite unfamiliar files or delete locks. |

A process killed while writing an unpublished scratch file can leave a
`.viggobsidian/.relocation-*.tmp` orphan. It is not a journal or authority;
recover from the original plan and published journal/receipt. Cleanup is a
separate, operator-verified step after proving no active process owns the file.

## Evidence and limits

The Rust integration fixture exercises the real CLI, pre/post adoption and
integration states, link-edit refusal, partial move refusal, interrupted
metadata recovery, route resolution and ordinary write-set behavior. The
fixture simulates the external app's two rename effects. It does not establish
live Obsidian link handling or local registry integration. Those require a
host-owned rehearsal with fictional records and a verified FileManager route.
The feature adds no dependency; it uses the existing Rust JSON and SHA-256
dependencies, standard Unix directory locks and atomic same-directory rename
for metadata only.
