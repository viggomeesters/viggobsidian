# Source-bound Work task promotion plan

`vault-layout --vault ROOT --plan-work-task-promotion` reads one JSON request
from stdin. It prepares metadata for converting the existing primary component
Raw note into a task, without creating or editing any note, attachment, lock,
registry entry or receipt. It is not a write authority.

The exact request has four fields:

```json
{
  "schema": "viggobsidian.work-task-promotion.v1",
  "path": "21-program/2026-11-example/ex-00-project-wiki/raw/20261111-source/20261111-source.md",
  "source_sha256": "SHA256_OF_COMPLETE_CURRENT_NOTE",
  "request_id": "71000000-0000-4000-8000-000000000001"
}
```

The selected root, configured Work project roots, project/component metadata,
primary-note identity and source hash are verified. Only an unprocessed own
Event Note v1.1 source is supported: it already has the same source/task card
layout, so promotion requires no body rewrite. Bound, registered, generated or
processed sources retain their own writers. A source already carrying a task
code is refused. Existing source metadata and content are not returned as a
replacement document.

The deterministic `viggobsidian.work-task-promotion-plan.v1` result contains
`path`, `raw`, `source_sha256`, `task-id`, `task-code`, `name`, `pins`, `members`,
`project_id` and `component-id`. The shared existing intake allocator chooses
the next unused component task code and refuses invalid/duplicate existing
codes. A valid existing task ID is preserved; otherwise the invocation UUID
becomes `task:UUID`. An identity already belonging to another Raw task is refused.
The complete original name/title remains as the suffix of the coded task name.

`pins` bind each existing project/component/sibling primary-note snapshot to
its SHA-256; a missing primary has a null hash. `members` lists every visible
immediate Raw entry in sorted order, including files and packages. These detect
both changes to existing notes and new siblings. The selected source is bound
separately by its exact hash and the caller's editor CAS.

The local writer must hold the same `.work-intake.lock` used by native intake
through preparation, CAS publication, validation and receipt. It must verify
all pins and membership before and after publication, preserve the original
body, source identities, dates and attachments, and retain its sealed journal
for guarded recovery. A plan alone reserves no code. Without this transaction
and lock, callers cannot claim that the returned allocation is current.

This adds no status command, general metadata writer, source conversion waiver
or registry lease. Existing `--new-work-task` behavior remains create-only and
uses the same code allocator.
