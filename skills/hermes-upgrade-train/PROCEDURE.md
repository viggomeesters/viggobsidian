---
name: hermes-upgrade-train
description: Audit, prepare, rehearse, stage, apply or verify a Hermes upgrade with separately classified local carries and overlays, exact candidate provenance, rollback and loaded-runtime proof.
---

# Hermes upgrade train

Keep upstream engine, minimal generic carries/extension points, local overlay and live runtime state separate. A successful upgrade should reduce or preserve the necessary carry set rather than grow a fork casually. A source checkout change is not a loaded gateway and a successful message is not a soak test.

Read `.agents/skill-adapters/hermes-upgrade-train.md` from the explicitly selected target root. This adapter is required for runtime/candidate effects: it identifies current live/core/canary/overlay repositories, supported automation interfaces, local policy, backup scope, permitted test environment and idle reload/readback owner. Inspect current runbooks and CLI help; never recreate missing host scripts from remembered paths. Without a required interface, report the dependent limit and continue only independent authorized read-only analysis.

## Choose the requested mode

- Audit: fetch official references and inspect carries/runtime; no candidate or live mutation.
- Candidate: create an isolated stable-based candidate and port reviewed carries; live remains untouched.
- Rehearse: prove candidate, real overlay registration, config preview and rollback without live mutation.
- Stage: only when explicitly requested, switch live source to the exact rehearsed candidate without gateway restart.
- Apply: execute prior gates, stage, idle-gated reload and runtime proof within actual authorization.
- Verify: read-only comparison of source, loaded process, representative behavior and soak evidence.

A request to audit or prepare is not rollout permission. Read every existing ownership and active-process boundary before mutation. Explicit apply may proceed through green gates without repeated confirmations, but stops at real ownership, backup, credential, drift or active-agent integrity failures.

## Source truth and carry review

Inventory exact package version, live/overlay/canary branch/SHA and dirtiness, official stable tag and fetched upstream HEAD, merge base, actual ahead/behind and active gateway/interpreter generation. Use Git evidence; a version string's behind count may describe another ancestry. Stable is the default target; unreleased main is a separate explicit choice.

Assess every carry using patch-id and observable behavior, not subjects alone:

- Upstream-equivalent: drop when current behavior replaces it.
- Obsolete: drop retired paths and revert/history noise.
- Overlay: keep local policy, presentation and bridges outside generic core.
- Generic carry: port the smallest behavior still missing upstream.
- Extension point: retain a narrow public hook and keep policy local.
- Workflow-only: do not port task ledgers/evidence as engine changes.

Record original SHA, patch-id, touched files, classification reason, upstream replacement evidence, focused test/invariant and overlay extraction status. An audit snapshot mismatch needs a fresh review, never bypassing the decision gate. No unaudited carry enters a candidate.

## Candidate and complete rehearsal

Use the maintained local provider to create a fresh stable worktree/branch. Preserve a dirty or interrupted canary as evidence rather than force-reusing a provenance-pinned directory. Port only reviewed generic carries and hooks, resolving against current upstream architecture. Retain exact stable base, candidate SHA and carry count.

Prove all affected boundaries in isolated test state, without live auth, state databases, cron or gateway processes:

1. Candidate is the exact intended generation and its ownership is understood.
2. Config migration preview uses copies or fixtures, not the live home.
3. Focused behavioral tests cover every retained carry.
4. Relevant gateway/session/queue/cron/database tests exercise affected risks.
5. The real local overlay registers against this exact candidate.
6. Overlay policy/document/queue validators pass where applicable.
7. Import/compile and CLI/version/status smoke use the candidate binary.
8. Backup contents and checksums are recorded, and rollback can recover the original live generation.

Apply only tests relevant to changed behavior, while retaining actual overlay integration as a separate proof from unrelated green repository suites. Do not copy authentic user payloads into public fixtures. A rehearsal report distinguishes clean proven readiness from understood report-only warnings and blocking failures.

## Stage, reload and prove

Before stage, recheck candidate SHA against the rehearsal, live/overlay/candidate cleanliness and ownership, absence of interrupted Git operations, understood config migration and verified rollback. Switching source and restarting a gateway are separate effects with different rollback evidence.

Reload only through the supported single-owner idle gate. Do not kill active agents, restart from an active delivery turn blindly, stack restart watchers or treat scheduled reload as loaded. An unreadable idle/profile state is a real boundary. Verify each affected profile separately rather than assuming one process proves the fleet.

After reload, record old/new PID, executable/interpreter/source generation, loaded version/SHA where provable, representative affected routes and fresh logs. Queue, database concurrency, scheduler and long-running worker changes need a bounded soak condition separate from immediate proof. Do not create a new scheduler as an upgrade side effect or wait silently for hours; use the actual existing monitoring path when authorized.

Report patched/source-persisted, loaded, runtime-proven and soak-proven as distinct states. A one-shot delivery cannot prove scheduler recovery; stage alone cannot be called an upgraded runtime. Failed stage or runtime verification preserves logs and generation, stops further rollout, restores the rehearsed backup via the supported provider, reloads through the same idle gate and proves rollback. Never mutate a failed pinned candidate silently and retry it as the same artifact.

Finish with requested mode, practical result, exact live/target generations, carry classifications and remaining unaudited count, candidate/overlay/backup gates, loaded-process evidence, affected-route proof and any concrete pending soak condition. Capability availability does not authorize new capabilities during the upgrade.
