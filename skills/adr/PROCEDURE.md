---
name: adr
description: "Create or maintain repository-local Architecture Decision Records from an actual decision, using the repository's existing convention and recording context, alternatives, consequences, and supersession evidence."
version: 2.0.0
author: Generic Skill Library
license: MIT
user-invocable: true
triggers:
  - "/adr"
  - "adr"
  - "architectuur besluit"
  - "decision record"
metadata:
  hermes:
    tags: [architecture, decisions, adr, documentation]
    related_skills: [domain-modeling, grill-with-docs, github-work, wayfinder]
created_by: agent
---

# ADR — Repository-Local Decision Capture

## Boundary

Record decisions in the active repository. Read its `AGENTS.md` and discover
existing ADR paths, numbering, templates, indexes, and supersession conventions
before writing. Never use a hard-coded repository, global vault, or retired
workflow location as fallback.

An explicit `/adr` creation request authorizes the bounded repository document
write when the decision and target repository are clear. Automatic suggestions
must satisfy all three gates:

1. hard to reverse;
2. surprising without context; and
3. the result of a real trade-off between plausible alternatives.

If no actual decision exists, ask for the missing choice or keep the issue in
its canonical task. Do not turn speculation into an accepted ADR.

## Workflow

1. **Resolve mode:** list, create, or supersede; resolve the exact repository.
2. **Discover convention:** search for ADR docs/config/indexes and follow their
   naming, status, and numbering. If none exists, propose `docs/adr/` with a
   minimal Nygard-style record; create it only when writes are in scope.
3. **Recover evidence:** identify the decision, forces, alternatives actually
   considered, accepted consequences, and source task/spec/discussion.
4. **Write one record:** preserve existing format. A minimal new convention uses
   Context, Decision, Alternatives Considered, Consequences, Status, and Date.
5. **Supersede explicitly:** link old and new records in both directions and
   retain the historical rationale; never rewrite history as if the old choice
   never existed.
6. **Verify:** read the record back, check links/index/numbering, run repository
   doc or generated-file checks, and confirm no unrelated file changed.

## Content Rules

- State why the choice exists, not an implementation playbook.
- Use canonical terms from `domain-modeling`/the repository glossary.
- Name real alternatives and why they lost; never fabricate a balanced list.
- Record positive and negative consequences, including operational cost.
- Link the originating GitHub Issue, PR, spec or retained evidence when available.
- Update generated indexes through their generator, never by hand.

## Final Evidence

Report repository, ADR path/id, status, decision, alternatives, consequences,
source evidence, superseded records, and validation commands. If no ADR was
written, state which gate or decision evidence was missing.
