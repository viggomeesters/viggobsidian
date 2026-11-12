---
name: devil-review
description: "Toets een afrondingsclaim adversarieel; rapporteer bewijs, blockers en een A/B/C/D-verdict."
version: 1.0.0
author: Hermes Agent
license: MIT
user-invocable: true
triggers:
  - "/devil-review"
  - "devil review"
  - "devil's advocate"
  - "adversarial review"
  - "probeer dit kapot te maken"
  - "nog niet klaar"
metadata:
  hermes:
    tags: [adversarial, review, pre-ship, risk, verdict]
    related_skills: [quality-audit, hardening-pass, next-improvements]
---

# Devil Review

`devil-review` is the adversarial review lane. It answers: **if this is not actually done, where would it break?**

## Boundary

Use this for **attacking the done-claim**. Do not use it for systematic lens-based inspection (`quality-audit`), safe final fixes (`hardening-pass`), or future backlog (`next-improvements`).

`devil` is a compatibility alias. Prefer `devil-review` as the canonical name.

## Workflow

Read [quality-evidence](../_bundle/contracts/quality-evidence.md) for common evidence and effect boundaries; the adversarial inversion and A/B/C/D verdict below remain this mode's responsibility.

1. Read the stated acceptance, done-claim, diff, tests, logs, docs, and relevant runtime evidence.
2. Name the main assumption behind the work.
3. Invert that assumption: “What if this is false?”
4. Look for concrete failure paths: literal errors, untested branches, rollback gaps, hidden coupling, stale docs, user-impact mismatches, missing proof.
5. Separate in-scope defects from genuine follow-up work.
6. Return a verdict. Do not soften blockers into backlog just to preserve momentum.

## Verdicts

- `A`: ship/accept as-is. No blockers found in checked evidence.
- `B`: ship only after listed in-scope fixes.
- `C`: not done; needs another implementation pass.
- `D`: stop; direction, safety, or product premise is wrong.

## Output

- `Verdict`: A/B/C/D with one-line rationale.
- `Main assumption attacked`: the assumption and inversion.
- `Blocking findings`: in-scope issues that must be fixed before done.
- `Non-blocking concerns`: real but not blocking for the current scope.
- `Evidence checked`: files, commands, screenshots, logs, or docs inspected.
- `In-scope fixes`: only if any blockers are fixable now.

## Common pitfalls

1. Repeating a normal audit checklist. This lane must attack the claim.
2. Inventing hypothetical risks without evidence or plausible failure path.
3. Treating all concerns as blockers.
4. Letting in-scope defects become “future improvements”.
