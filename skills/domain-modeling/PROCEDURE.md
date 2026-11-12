---
name: domain-modeling
description: "Use when canonical domain terms, relationships, bounded contexts, or hard-to-reverse domain decisions must be sharpened; challenge fuzzy language with concrete scenarios, cross-check code, and maintain the repository's glossary/ADR sources without turning them into a spec."
version: 1.0.0
author: Generic Skill Library (adapted from Matt Pocock's domain-modeling)
license: MIT
metadata:
  source: https://github.com/mattpocock/skills/tree/main/skills/engineering/domain-modeling
  source_commit: 2ab958093e83e0ec752e6c1c5932da465bf23e0c
  hermes:
    tags: [domain-modeling, ubiquitous-language, glossary, adr, architecture]
    related_skills: [grill-with-docs, adr, wayfinder, brainstorm, tdd]
created_by: agent
---

# Domain Modeling

## Purpose

Actively sharpen the language and conceptual boundaries a repository uses to
describe its problem domain. This is an internal, model-invoked discipline: it
does not add another public slash command.

Reading an existing glossary for vocabulary is ordinary repository inspection.
Use this skill only when a term, relationship, context boundary, or durable
domain decision is being changed or resolved.

## 1. Find the canonical context

Read the repository's `AGENTS.md` and discover its existing conventions before
writing. Look for `CONTEXT-MAP.md`, `CONTEXT.md`, domain glossaries, architecture
docs, ADR indexes, code namespaces, schemas, and relevant GitHub decisions.

- If a context map exists, use it to locate the owner of each term.
- If one glossary exists, extend its existing format.
- If none exists and writes are authorized, create the smallest `CONTEXT.md`
  only after the first term is actually resolved.
- Never impose Matt's folder layout or use another repository/vault as fallback.

Completion criterion: the bounded context and canonical glossary/decision
locations are evidenced, or the absence of a convention is explicit.

## 2. Sharpen one ambiguity at a time

Call out contradictions and overloaded language immediately. Propose one
canonical term and distinguish it from neighboring concepts. Use a concrete
scenario that stresses identity, lifecycle, ownership, timing, multiplicity, or
failure behavior—for example, what happens when two actors modify the same
aggregate or when cancellation is partial rather than total.

Ask one unresolved question at a time when human judgment is required. Do not
manufacture the user's domain decision.

Completion criterion: each changed term has one precise meaning, avoided
synonyms where useful, and at least one scenario that demonstrates its boundary.

## 3. Cross-check language against reality

Search relevant code, tests, schemas, API contracts, fixtures, and existing
docs. When language and implementation disagree, surface the contradiction and
classify it rather than silently choosing:

- glossary is stale;
- code is stale or incorrect;
- the same word represents two contexts; or
- the decision is still unresolved.

Do not rename or refactor implementation as part of glossary maintenance unless
that implementation work is separately authorized and task-scoped.

Completion criterion: every material contradiction has an explicit owner and
disposition; none is hidden by documentation edits.

## 4. Maintain clean sources of truth

A domain glossary contains domain language, not implementation instructions,
feature requirements, plans, or scratch notes. Prefer this compact shape unless
the repository already defines another:

```markdown
## <Canonical term>

<One or two sentences defining the concept and its boundary.>

Avoid: <ambiguous or deprecated synonym, when useful>
Example: <one concrete scenario>
```

Keep implementation behavior in specs/tests, work status in GitHub Issues/PRs, and
architecture trade-offs in ADRs. Link between these artifacts instead of
copying their contents.

Completion criterion: each fact lives in one canonical artifact and the
glossary contains no implementation plan or duplicated decision rationale.

## 5. Escalate durable trade-offs to ADR

Load `adr` only when all three conditions hold:

1. **Hard to reverse** — changing the choice later has meaningful cost.
2. **Surprising without context** — a future maintainer would reasonably ask
   why this choice exists.
3. **Real trade-off** — plausible alternatives were considered and consequences
   were accepted.

Otherwise capture the resolved term in the glossary or the remaining question
in its canonical task/decision evidence. An explicit user request for an ADR may
proceed when an actual decision exists, but never fabricate alternatives.

Completion criterion: any ADR candidate passes all three gates or has an
explicit user request and sufficient decision evidence.

## Integration

- `grill-with-docs` invokes this discipline when its interview changes canonical
  terms or exposes context contradictions.
- Wayfinder prototype/grilling decisions may use it to resolve fog, but the
  human remains the owner of HITL judgment.
- `tdd`, code, and specs consume canonical terms; they do not redefine them
  silently.
- Canonical `github-work` remains the public repository-work command.

## Final Evidence

Report the context, terms/relationships changed, concrete scenarios used,
code/doc contradictions and their dispositions, canonical files updated,
validation performed, and any ADR created or deliberately skipped.

## Attribution

Adapted from Matt Pocock's MIT-licensed `domain-modeling` skill, inspected at
commit `2ab958093e83e0ec752e6c1c5932da465bf23e0c`. This version discovers local
repository conventions, preserves GitHub Issue/PR ownership, and remains an internal
discipline instead of introducing another public command.
