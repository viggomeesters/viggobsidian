---
name: repo-complete
description: Assess or improve repository readiness with scoped hygiene, privacy, runnable entry points, local validation and actual release/deployment evidence.
---

# Repository completeness

Resolve the repository and intended readiness level: local/internal, externally shared source, or a published product/package. Inspect the actual product kind and existing instructions before selecting checks. A check/plan remains read-only; explicit apply authorizes scoped safe repairs, not overwrite, remote metadata changes or publication beyond the request.

## Modes and source state

- **Check:** report actual gaps, evidence and consequence.
- **Plan:** make the proposed repairs concrete and reviewable.
- **Apply:** implement the authorized missing pieces while preserving existing human content and unrelated dirty changes.
- **Validate:** run the applicable concrete checks on the candidate and report nonzero failures honestly.

If a target-owned checker or bootstrap provider is required, resolve it through that target's verified configuration. Read `.agents/skill-adapters/repo-complete.md` from the explicitly selected target root when present for additional owner criteria and runner bindings. Never derive a target or executable path from this skill's installation. An unavailable runner limits the checks it owns; it does not justify claiming its result from a handwritten checklist or installing an old helper.

## Inspect the product

Determine kind from real manifests and source, not one filename: plugin, app, CLI, library or reference/data repository. Identify how a new caller installs, invokes and verifies it. Check current version information and inspect the actual build/test commands, dependency lock and supported platform. Preserve established language/toolchain choices and data/writer contracts.

For an unfamiliar or complex repo establish a short onboarding map: entry points → domain flows → storage owners → side-effect boundaries → tests/evals → known risks. Use a graph only when it aids a real question; an inspectable explanation of the route can be enough. For changed code connect impact to the checks actually needed and remaining risk.

## Hygiene and usability

Evaluate README purpose, setup/usage examples, applicable license, ignore rules, and version-control state. Additional contributing/security/changelog/release guidance depends on the audience and product. Do not add empty boilerplate simply to make a checklist green. Never choose a license owner or legal license without authority.

Check kind-specific files against the real build system rather than forcing a universal scaffold. For plugins validate manifest/package/release asset consistency. For a CLI verify an actual invocable example. For a library verify an API/install example. For apps verify the requested user flow, assets and deployed target if deployment is in scope.

A visual, demo or screenshot is useful public product evidence where it explains the UI; a universal branded hero is not a technical readiness prerequisite unless the owner explicitly chose that criterion. Preserve any meaningful selected visual system. Render created/changed visual assets and inspect actual content, clipping and readability.

## Validation strategy

Use the repository's approved local validation commands and appropriate independent review. Do not create, enable, dispatch or require GitHub Actions. A missing CI service does not block successful local validation. If an existing workflow is out of scope, leave it intact; do not silently remove it.

A bootstrap route is create-only by default: identify exact files and preimages, create only authorized missing content and read it back. Changing an existing file follows the actual requested repair, preserving unrelated content. Do not assume package managers, runtime installs or architecture modernization are necessary for readiness.

## Public-safety review

Before any authorized public sharing inspect current tracked content and relevant history for secrets, environment files, private keys, credentials, personal/financial defaults, real customer data, proprietary files, screenshots and large binary assets. Ignore patterns alone do not prove absence from history. Distinguish a current-tree cleanup from a historical disclosure; do not rewrite shared history without a deliberate separate decision.

Verify provenance and license for copied code/templates and public-safe fixtures. A realistic-looking private example is not safe merely because it is in a test file. Unknown ownership or license blocks the affected publication, not unrelated local improvements.

## Public-facing documentation

The first screen should deliver what the page promises. A product landing shows purpose and real use; an image gallery displays the images; architecture/API/CLI documentation shows the actual model or examples; release/status pages show actual released artifacts or dated state. Runbooks may start with procedure because that is their explicit purpose.

Inspect relevant README, docs, contribution, security and release pages. Look for stale installation/current-state claims, placeholders presented as finished product, missing/nontracked assets and mismatched URLs. Preserve honest proposals where they are useful; do not make the word future alone a universal error. Repair exact misleading claims instead of enforcing a fixed document template.

## Remote and release evidence

Where requested, verify actual remote repository/visibility, description/topics and product URL. Metadata edits require their own applicable authorization and exact target. For an app compare README and chosen homepage with the actual reachable deployment. For a release verify the real tag and appropriate attached binary/package/plugin assets. A README claim does not prove a first release, deployed URL or published package.

Whether a release, hosted demo or public visibility is required comes from intended audience and project policy. A private utility or static reference repo need not publish just to complete a local repair. Use [release](../release/PROCEDURE.md) when shipping is actually part of the task.

## Finish

Lead with the practical readiness result, then concrete blockers and material unknowns. Keep privacy/publication blockers explicit. Report the actual commands/results and remaining manual or access-dependent checks. A count of created files or a green syntax check is not full product acceptance. If the task authorized repair, complete safe in-scope fixes before reporting; leave a precise pending external action rather than claiming an unobserved rollout.

The onboarding insight that a useful graph teaches routes rather than impressing with density is attributed to the MIT-licensed Understand-Anything project, inspected source `https://github.com/Egonex-AI/Understand-Anything` at `54754a6f9705`. No project code is reproduced here.
