---
name: grill-with-docs
description: "Use when stress-testing a plan or design against a repository's existing docs, domain language, contracts, ADRs, and code; ask one hard question at a time and capture resolved terminology/decisions."
version: 1.0.0
author: Hermes Agent (adapted from mattpocock/skills grill-with-docs pattern)
license: MIT
user-invocable: true
triggers:
  - "/grill-with-docs"
  - "grill with docs"
  - "grill-with-docs"
  - "stress-test against docs"
  - "check against docs"
metadata:
  hermes:
    tags: [planning, documentation, domain-language, adr, interview, architecture]
    related_skills: [grill-me, domain-modeling, adr, writing-plans, codebase-inspection, tdd]
created_by: agent
---

# Grill With Docs

Read `.agents/skill-adapters/grill-with-docs.md` from the explicitly selected target root when present for its documentary authority and storage bindings. Do not derive a target from this source root. Missing mandatory evidence or writer authority blocks the dependent conclusion/write, not independent critique of supplied material.

## Overview

`grill-with-docs` is `grill-me` plus documentary evidence. It stress-tests a plan against the repository's existing language, constraints, ADRs, contracts, and code before implementation starts.

Default behavior:

1. Inspect relevant docs/code first.
2. Answer anything the repo already answers.
3. Ask one hard unresolved question at a time, with a recommended answer.
4. When a term or decision crystallises, capture it in the right doc if the current task scope allows writes.

## When to Use

Load this when the user says:

- “grill with docs”;
- “check dit tegen docs/context/ADR”;
- “stress-test dit plan tegen de bestaande afspraken”;
- “maak de domeintaal scherper”;
- “zorg dat toekomstige agents dit snappen”;
- a repo plan depends on terminology, existing conventions, contracts, or architectural decisions.

Use plain `grill-me` for general/personal/product plans where repo docs are not the source of truth.

## Evidence to Inspect First

Choose the relevant subset; do not read the whole repo blindly.

General repo docs:

- `AGENTS.md`, `CLAUDE.md`, `CODEX.md`, `GEMINI.md`;
- `README.md`, `docs/**`, `CONTEXT.md`, `CONTEXT-MAP.md`;
- `docs/adr/**` or `adr/**`;
- architecture/design docs, roadmap docs, release gates.

Optional target-specific repository surfaces:

- `system/contracts/**` when the repo/vault defines contracts;
- `docs/skill-registry.yaml` / generated inventories for skill work;
- skill frontmatter and command adapters for agent-skill repos;
- Life OS vault docs only when explicitly in scope and with the vault safety rules loaded.

Code reality:

- search/read the relevant modules when a claimed behavior, term, or boundary can be checked in code;
- use tests and config as evidence of actual contracts;
- if docs and code conflict, surface the contradiction before planning.

## Interview Loop

Ask in this shape:

```markdown
### Vraag N — <decision branch>
**Docs/code evidence:** <what I found, with paths>
**Vraag:** <one unresolved hard question>
**Waarom dit blokkeert:** <what breaks if we guess wrong>
**Mijn aanbevolen antwoord:** <default recommendation>
**If accepted:** <doc/plan/code implication>
```

Rules:

- One question at a time.
- No questions that docs/code/tools can answer.
- Prefer concrete scenarios over abstract taste: “What happens when a trial customer upgrades and cancels in the same billing period?”
- Challenge fuzzy or overloaded language immediately.
- Stop once the plan is executable enough; do not interrogate forever.

## Domain Language Capture

If the session changes a canonical term, relationship, or bounded context—or
exposes a contradiction between language and code—load and follow
`domain-modeling`. Merely reading existing terminology does not invoke that
discipline.

If the repo has domain language docs, use them. If not, create lazily only when there is real language to capture and writes are in scope.

Recommended `CONTEXT.md` shape:

```md
# <Context Name>

<One or two sentences describing the context.>

## Language

**Canonical Term**:
One or two sentence definition of what it is.
_Avoid_: ambiguous synonym, outdated term
```

Rules:

- Keep `CONTEXT.md` as glossary/domain language, not implementation spec.
- Define project-specific concepts only; skip generic programming terms.
- Be opinionated: choose one canonical term and list avoided synonyms.
- In multi-context repos, use `CONTEXT-MAP.md` if present; otherwise ask before inventing a split.

## ADR Capture

Offer an ADR only when all three are true:

1. **Hard to reverse** — changing later is meaningful work.
2. **Surprising without context** — a future agent/developer would wonder why.
3. **Real trade-off** — there were plausible alternatives.

Use existing ADR conventions if present. Otherwise use a minimal ADR:

```md
# <Short decision title>

Status: Accepted
Date: YYYY-MM-DD

## Context
<Why this decision exists.>

## Decision
<What we chose.>

## Alternatives Considered
- <Option> — <why not now>

## Consequences
Positive:
- ...

Negative / trade-offs:
- ...
```

Use the repo-local `adr` skill when writing ADRs; it discovers the repository's
existing convention and applies the hard-to-reverse, surprising, real-trade-off
gate.

## Documentation Write Policy

Write docs inline only when:

- the current task explicitly allows repo writes;
- the resolved term/decision is stable enough to preserve;
- the target doc is clearly the source of truth;
- no vault/private/client boundary is crossed accidentally.

If writes are not safe or not in scope, return a patch/draft instead of editing.

Never update generated docs by hand. Update source files or generator scripts, then regenerate.

## How This Improves Implementation

After grilling:

- test names can use canonical domain words;
- modules/functions use consistent terms;
- agents stop re-asking already-documented decisions;
- TDD public interfaces are clearer;
- ADRs prevent future “helpful” reversals of deliberate choices.

## Common Pitfalls

1. **Skipping doc/code inspection.** Then this is just `grill-me` with extra words.
2. **Writing CONTEXT.md as a spec.** It is glossary/domain language only.
3. **Creating ADRs for obvious choices.** ADRs are for meaningful trade-offs.
4. **Batching questions.** One branch at a time.
5. **Trusting stale docs over code silently.** Call out drift.
6. **Writing into the Life OS vault by accident.** Load vault rules and use explicit scope.

## Verification Checklist

- [ ] Relevant docs/code inspected before first question.
- [ ] Repo evidence cited by path.
- [ ] Questions limited to unresolved branches.
- [ ] Recommended answer included per question.
- [ ] Resolved terms/decisions captured or explicitly left as draft.
- [ ] Generated docs not hand-edited.
