---
name: improve
description: "Alias voor hardening-pass: herstel verifieerbare gaten binnen de huidige scope."
version: 1.0.0
author: Hermes Agent
license: MIT
user-invocable: true
triggers:
  - "/improve"
  - "improve"
  - "9000x second pass"
  - "weet je zeker dat dit klaar is"
metadata:
  hermes:
    tags: [alias, compatibility, hardening]
    related_skills: [hardening-pass]
---

# improve → hardening-pass

This skill is a backward-compatible alias.

## Canonical target

Use `hardening-pass` for the real workflow. Do not treat `improve` as a separate lane.

## Action

Immediately follow [hardening-pass](../hardening-pass/PROCEDURE.md):

- Inspect the almost-done work.
- Execute only safe local in-scope hardening.
- Verify after every improvement.
- Separate outside-scope proposals.

## Output

Return the normal `hardening-pass` output: safe in-scope fixes applied, checks rerun, remaining risk, and ship/no-ship status.

## Migration note

Prefer writing new docs, tasks, and future references with `hardening-pass`. Keep this alias small and boring.
