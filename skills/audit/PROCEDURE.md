---
name: audit
description: Use when the user wants findings from an existing app, repo, code change, data flow, document, or plan. Select the right audit lens, inspect evidence, and report severity-ordered findings without mutating unless fixes are explicitly requested.
version: 2.0.0
author: Hermes Agent
license: MIT
user-invocable: true
triggers:
  - "/audit"
  - "audit"
  - "doe een audit"
  - "audit deze"
  - "quality review"
  - "production readiness"
  - "security audit"
metadata:
  hermes:
    tags: [lane, audit, review, existing-artifact, findings]
    related_skills: [quality-audit, data-audit, codebase-inspection, grill-with-docs]
---

# Audit

Inspect the user's chosen app, repository, change, data flow, process, document or plan, independently of recently delivered work. Findings come first; this request does not authorize fixes. Establish the target and requested questions from the prompt and available context; ask only if the missing scope changes what must be inspected. Do not silently substitute the last delivered artifact.

Read [quality-evidence](../_bundle/contracts/quality-evidence.md) for scope, evidence, severity and acceptance. Select only the smallest useful specialist:

- Code/app/change/repo: [quality-audit](../quality-audit/PROCEDURE.md), selecting code, tests, architecture, API, performance, security or operability.
- Unknown codebase: inspect actual owner docs, structure and runtime configuration, then select the relevant quality lens. No missing codebase-inspection skill is assumed.
- Spreadsheet data: [xlsx](../xlsx/PROCEDURE.md), analysis-only mode.
- ETL/reports/lineage: inspect source-to-target mappings, transformations, units, nulls, duplicate handling and reconciliations; select quality-audit code/API/architecture lenses as needed. Do not require an unavailable data-audit provider.
- Domain/docs/ADR stress test: [grill-with-docs](../grill-with-docs/PROCEDURE.md), preserving read-only intent.
- UI/design/impeccable: quality-audit's frontend-impeccable-audit pack plus relevant [frontend](../frontend/PROCEDURE.md) verification rules.
- Full app, calculator or repo-health: the corresponding named pack in quality-audit; do not flatten those checks into a generic pass.

Idea shaping belongs to brainstorm; a just-delivered repair loop to recheck; future improvements to next; agent-process feedback to selfimprove. Explicit fixes select the actual owner's bounded implementation/hardening route, not implicit write permission from an audit lens.

## Result and handoff

Report selected modes, severity-ordered findings with evidence, suggested (not executed) corrections, checks performed and residual risk. If no findings exist, name the strongest evidence and its scope. Follow [work-handoff](../_bundle/contracts/work-handoff.md) only when durable follow-up is requested; findings-only work creates no task queue.
