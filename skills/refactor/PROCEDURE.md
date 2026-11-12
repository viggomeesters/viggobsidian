---
name: refactor
description: Structured code refactoring with impact analysis, safety checks, and verification
user-invocable: true
persona: sparring
triggers:
  - "/refactor"
  - "refactor this"
  - "extract function"
  - "rename across"
  - "deduplicate"
  - "simplify this code"
tools:
  - name: refactor_code
    description: "Systematically refactor code with impact analysis, safety checks, and verification."
    parameters:
      type: object
      properties:
        files:
          type: array
          items:
            type: string
          description: "List of files or glob patterns to refactor."
        goal:
          type: string
          description: "Description of the refactoring goal (e.g. 'Extract user validation')."
      required: ["files", "goal"]
---

# Refactor

For an explicitly selected target, read `.agents/skill-adapters/refactor.md` when present for local verification and owner boundaries. Never infer that target from this skill's installation directory. A missing adapter blocks only a dependent owner-specific effect, not ordinary source inspection.

Improve the requested code structure while preserving observable behavior. Read
the target repository's instructions, domain terms and existing tests. A review
or plan-only request remains read-only; an explicit refactor request authorizes
its bounded implementation without a second routine approval round.

## Establish impact

Resolve the target and purpose from the request: extract, rename, deduplicate,
simplify or deepen a module. Ask for a target only when it cannot be recovered
from the current context. Inspect actual callers/imports, relevant indirect
consumers, public contracts and test coverage before choosing a change. Report
only material scope and compatibility risk; a file-count table is optional.

Choose the smallest useful move. Small covered internal edits can proceed
with focused checks. Wider changes need a short sequence and verification at
each boundary. A public API, persisted format or behavior change is a separate
scope decision; do not disguise it as cleanup. Follow existing authorization and
repository gates, asking only for a consequential unresolved choice.

For module/interface deepening, read [deep-module-design](references/deep-module-design.md)
before choosing the seam or replacing tests. Preserve domain language and ADRs.
Prefer a small interface hiding useful behavior, a real adapter boundary, narrower
parameters, logic at its owner or a proven obsolete-path deletion. Avoid shallow
wrappers, speculative adapters, utility dumping grounds and mocks of internals.

## Preserve behavior and execute

Use existing behavioral tests as the baseline. If risky behavior lacks coverage,
add public-interface characterization tests and observe them pass before the
refactor; a preservation test need not manufacture a failure. New or changed
behavior follows [TDD](../tdd/PROCEDURE.md) only when authorized.

Make one coherent structural step at a time, then run its relevant checks. If
one fails, diagnose and repair the scoped failure or recover this task's own
step safely; do not continue dependent changes on broken behavior or immediately
return a routine repair to the user. Preserve unrelated dirty work. Stop only for a
real scope/authority gap or unresolved behavior requirement.

For renames/extractions, check exports, imports, dynamic references, configuration
and known external consumers. Delete an old path only after proving it unused or
completing the authorized migration; lack of a static import alone is insufficient.
Remove obsolete imports and commented-out code within scope. Keep useful comments
and distinct test coverage. Do not mix unrelated modernization into the refactor.

## Verify and report

Run the affected behavioral checks and the repository's required suite/build.
Check changed call sites and public compatibility; report any unavailable checks
or remaining consumers. Finish with the structural result, changed paths, actual
verification and material residual risk. Follow repository-local delivery rules;
push retains explicit authorization.

Use [audit](../audit/SKILL.md) for an inspection without edits,
[debug](../debug/PROCEDURE.md) for a broken behavior and [recheck](../recheck/SKILL.md)
for the work just delivered.
