---
name: repo-design-vision
description: Create or revise a repository's requested product/design contract with concrete principles, boundaries and acceptance criteria.
---

# Repository design vision

For a target-specific convention, read `.agents/skill-adapters/repo-design-vision.md` from the explicitly selected repository when present. This source directory never selects a data root or authorizes a missing writer. An unavailable required adapter blocks that dependent effect, not a read-only design review.

Use when a user asks for a repository vision, structural product principles or a repeatable operating contract. Read existing README, design/architecture decisions, schemas, domain vocabulary, tests and AGENTS instructions before proposing another artifact. Preserve the repository's chosen format and source of truth. A tiny utility or repo-readiness check does not automatically require a new machine-readable contract.

## Establish purpose

Identify actual users/callers, product surface, observable promise, core flows and consequential constraints. Agent-first is a legitimate product choice when supported by the repository, not a universal requirement. Inspect real entry points and implementation before writing intended-state claims. Separate observed current behavior, accepted goals and proposals.

Make every principle actionable: its stable identifier, rule, good/bad examples where useful and a way a reviewer can judge it. Prefer a small set that constrains real choices over a large generic manifesto. Include non-goals, public/data safety boundaries and the acceptance criteria that materially define the requested product.

Useful questions include: what does a fresh clone need to run, what must remain fast, what errors fail closed, where are sources/ownership preserved, which side effects require authorization, and which unknowns block the next step? Choose only those relevant to the actual repository.

## Artifact and validation

If a JSON contract is requested or already standard, preserve its existing schema and validate against it. A compact example shape is:

```json
{
  "schema_version": "1.0.0",
  "artifact_kind": "repo_design_vision_contract",
  "repo": {"name": "example", "primary_users": ["developers"]},
  "vision": {"one_line": "Observable product promise"},
  "principles": [{"id": "source-integrity", "rule": "Preserve source identity during migration"}],
  "non_goals": [],
  "public_safety": {"forbidden": ["secrets", "authentic confidential data"]},
  "acceptance_scorecard": [{"id": "source-integrity", "pass_condition": "Changed sources retain verified identity and provenance"}],
  "review_checklist": ["Are current claims backed by the implementation?"]
}
```

This example is illustrative, not a new mandatory schema or output file. Use Markdown or the existing native format when that is requested. Link from the repository's appropriate entry only when the artifact is meant to become maintained guidance. Do not duplicate the README and a new document word for word.

Use existing schema/contract checks where they exist. Add meaningful semantic tests only when the contract participates in runnable behavior; do not create tests that merely assert copied prose or keys without value. Repository changes follow its normal local checks and review, with publication/push authority preserved.

## Review

Can a future maintainer use the contract to reject a concrete wrong change? Are domain terms, public safety and side-effect boundaries explicit? Do principles reflect the actual product surface rather than assumptions about agent-oriented software? Are gaps marked instead of described as completed capabilities? Report the resulting contract and any real unresolved product decision; no runtime workflow, task ledger or new governance engine follows from this document.
