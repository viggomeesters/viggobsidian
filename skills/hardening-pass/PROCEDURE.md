---
name: hardening-pass
description: "Verbeter en verifieer het bijna afgeronde resultaat binnen de geautoriseerde huidige scope."
version: 1.0.0
author: Hermes Agent
license: MIT
user-invocable: true
triggers:
  - "/hardening-pass"
  - "hardening pass"
  - "final pass"
  - "9000x second pass"
  - "weet je zeker dat dit klaar is"
  - "safe local improvements"
metadata:
  hermes:
    tags: [hardening, final-pass, automation, guardrails, verification]
    related_skills: [quality-audit, devil-review, next-improvements]
---

# Hardening Pass

Read `.agents/skill-adapters/hardening-pass.md` from the explicitly selected target root when present for its acceptance, rollout and writer bindings. Source location does not choose a target. A required missing provider blocks its dependent effect; never invent a workflow or use another vault as fallback.

`hardening-pass` is the final executing improvement lane. It answers: **before we call this done, what safe in-scope improvements can we still make and prove now?**

This internal mode is reached through public `recheck` or an explicit finish request to `audit`. `improve` is a historical name, not a guaranteed native command. This mode may harden authorized work, not merely give a second opinion.

## Boundary

Use this for **making the current work safer/stronger now**. Do not use it for read-only quality findings (`quality-audit`), adversarial done-claim attack (`devil-review`), or future backlog shaping (`next-improvements`).

## Contract Loading

Prefer the repo-local contract when present:

```text
system/contracts/automation/9000x-automation-audit-contract.yaml
system/contracts/automation/domains/
system/docs/9000x-automation-improvement-checklist.md
```

When those optional profiles do not exist in the target repository, use its actual acceptance criteria and current owner instructions. Do not load a retired iCloud or global-vault workflow fallback, invent a missing contract or change writer authority.

Pick the closest domain profile when relevant: `agent-workflow`, `vault`, `memory`, `projects`, `automations`, or `software-delivery`.

## Workflow

Read [quality-evidence](../_bundle/contracts/quality-evidence.md) for scope, findings and post-repair proof. This executing mode does not expand the existing authorization.

1. Inspect current work state: changed files, task scope, recent verification, and what was about to be called done.
2. Load the 9000x audit contract and relevant domain profile when available.
3. Check automation opportunity, set-and-forget readiness, guardrails, validation, docs, observability, ownership, recovery, context durability, and autonomy limits.
4. Execute only safe local improvements that are inside scope, reversible, and verifiable.
5. Ask permission for external, irreversible, privacy-sensitive, costly, or scope-expanding actions.
6. Do not automate judgment-heavy work when human taste, consent, privacy, or safety should remain in control.
7. Run the relevant checks after every executed improvement.

## Rollout / rollback

For user-visible, deployment-facing, data-affecting or runtime changes, read [rollout-readiness](references/rollout-readiness.md).

## Output

- `Automated`: what was safely improved or automated now.
- `Not automated`: what was deliberately left manual, with the reason.
- `Evidence`: checks, commands, files, logs, or validation that prove the result.
- `Proposals`: improvements outside current scope or requiring approval.
- `Next bottleneck`: the next highest-leverage constraint.

## Compatibility

- Canonical skill: `hardening-pass`.
- Legacy alias: `improve` / `/improve`.
- If the user says “second opinion” but expects actual safe fixes, use `hardening-pass`; if they only want a read-only opinion, use `quality-audit` or `devil-review`.
