---
name: quality-audit
description: Perform a focused evidence-backed quality review of a selected artifact, change, plan or repository without implicit repairs.
license: MIT
---

# Focused quality review

Choose the smallest lens set matching the request and risk. Inspection is not permission to repair, harden, create backlog or publish review comments. Adversarial assumption testing and just-delivered acceptance checks may use their own procedures without starting a second generic audit.

## Lenses

- **Code:** correctness, edge cases, maintainability and actual conventions.
- **Tests:** meaningful assertions, missed failure modes, fixtures and coverage of important behavior.
- **Architecture:** ownership, coupling, compatibility, interfaces, migration safety and reversibility. Repetition or a pass-through layer is a possible smell, not automatically a defect.
- **API:** caller contracts, permissions, error semantics and integration behavior.
- **Performance:** measured latency, memory, query, render and scaling risks; no speculative speedup claims.
- **Accessibility:** keyboard, focus, semantic names, contrast and responsive behavior.
- **UX:** the actual task flow, content hierarchy, empty/loading/error states and understandable outcomes.
- **Security:** authentication/authorization, data exposure, injection, secrets, dependencies and supply chain.
- **Production/operations:** rollout, rollback, observability, retention, auditability, tenant isolation and proportional infrastructure.

Named packs may select several lenses, but their name does not grant effects. If a selected pack has an existing definition, load it rather than substitute a generic pass. Optionally load `.agents/skill-adapters/quality-audit.md` from an explicit target root for local packs and project standards. The installed source root never selects that target; unavailable local evidence limits only the affected conclusion.

## Pin what is reviewed

Identify the actual artifact/source version. For branch/PR/WIP work, record a valid comparison ref and merge-base where applicable. Include explicitly selected uncommitted and untracked changes. An empty committed diff does not prove a dirty worktree has no changes.

Run two distinct passes over the same scope:

1. **Specification:** locate the request or acceptance source. Account for requirements as implemented, partial, missing or unverified; identify unintended behavior. If no specification exists, say so.
2. **Standards:** load the relevant repository/owner rules. Cite the actual rule behind a violation and distinguish it from personal taste. Use actual tool-check output for enforced conventions.

Preserve a verdict and strongest evidence for each axis; passing one cannot mask failure of the other. Separate actual defects from plausible but unproven risk. Rank findings by user impact, name a tight location/source and explain the concrete failure or missing proof. Do not manufacture findings for a quota.

For external links, retain query and fragment, follow redirects and verify the landed object semantically. A status 200 or matching path is not proof of the intended document, address or listing. Authentication may explain an inaccessible private target; do not infer a broken link solely from unauthenticated access.

## Report

Lead with actionable findings ordered by severity. State selected lenses, inspected scope, source/commands actually checked, material assumptions and residual uncertainty. If no findings remain, say so and give the strongest evidence. Do not claim completion or production readiness for untested requested behavior. Broader recommendations stay separate from defects in the current scope.
