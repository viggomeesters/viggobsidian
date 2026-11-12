---
name: release
description: Prepare or execute a version, tag, release or deployment using the selected repository's shipping authority and actual publication evidence.
---

# Release and rollout

Use for a release/version/tag/deployment request, release notes, or an explicit no-release decision. First resolve the repository, artifact and requested effect. Read the repository's release contract, version metadata and current source state. Preserve unrelated dirty work; use an isolated checkout where needed rather than resetting it or treating any unrelated edit as a permanent release blocker.

## Authority and scope

Publishing and deploying are separate effects. Authorization can come from the current instruction or a durable applicable project contract. For owner-specific defaults read `.agents/skill-adapters/release.md` from the explicitly selected target root when present. Never infer the target from the installed skill. A missing required shipping adapter blocks its external effect, not preparation or a bounded dry-run.

Verify the actual publish service, account, repository, branch, version/tag and artifact before mutation. A previous URL, successful build or accepted code suggestion is not automatically a publication instruction. Respect explicit local-only, preview-only or no-deploy scope. Do not create or dispatch a CI system merely to release; use the repository's approved local checks and deployment mechanism.

## Prepare

Inspect current version, latest relevant tags/releases, commit range, branch sync and package metadata. Choose the bump according to actual compatibility changes and repository policy, or record a precise no-release reason. Do not invent a version just to satisfy ceremony.

Draft release notes before changing external tags or systems. Describe the user-visible behavior, compatibility/migration requirements and material limits. For a release artifact run its required build, tests and packaging checks on the exact candidate. Dependency changes include the lockfile and applicable maintenance/license/install-script review.

Identify a proportionate rollback path: revert the change, reactivate a previous verified artifact/tag or use the repository's supported deployment rollback. Do not promise recoverability from an untested deletion or irreversible migration.

## Execute and verify

Create the intended commit/tag/publication/deployment only within authority. Inspect the external result and read back the actual version, attached artifacts, release URL or deployment identity. For a live app verify the production alias serves the intended build and main user-visible flow. Distinguish a preview, uploaded artifact, tag, published release and live deployment.

An immutable runtime needs evidence that the active source/version and payload are the candidate actually tested. A later quiet job on an old deployment does not prove a new rollout. If only metadata changed since the deployed commit, prove byte-equivalence for all runtime-owned paths rather than assuming it.

For an integrated deployment, follow its actual approved trigger and verify the resulting deployment instead of creating a duplicate manual one. If the environment has no approved automatic integration, use the authorized manual mechanism. Do not enable GitHub Actions as a fallback.

## Finish

Report the version decision, actual checks, commit/tag/release/deploy evidence and any remaining rollout gap. A no-release result gives its concrete reason. For production, retain the immediate main-flow, error/log and data-integrity checks appropriate to the product. If publication was not authorized or a required check is unavailable, leave a concrete reviewable candidate and name that exact boundary without claiming shipped.
