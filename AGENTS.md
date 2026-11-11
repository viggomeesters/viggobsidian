<!-- go-workflow:agents-gateway:v1:start -->
## Repository-local Go workflow gateway

This repository uses `.go/` as its project workflow source of truth. Keep repository-specific instructions outside this managed block; they remain binding.

When the user invokes `Go`, `Go plan`, `Go <task-id>`, `Go loop`, or asks to continue autonomously:

1. Check the existing immutable project pin before routing or product edits. Use `stack check` and the pinned launcher's `doctor` for compatible starts; older pins without `stack check` use their exact-ref doctor. Run `stack update` only for an explicitly selected upgrade or a required compatibility repair; inspect its isolated plan before apply.
2. Read `.go/vision.json`, `.go/architecture-principles.json`, `.go/hierarchy.json`, and the selected task.
3. Run the repository-local `validate`, `status`, and `router` commands, then announce `Route: <selected_route>`.
4. Create or repair a concrete `.go` task before changing product files. Execute one task at a time and stay within its `scope.modify` boundary.
5. Record content-bound verification evidence, run the required critic/recheck, repair blocking findings, and satisfy finish plus any required release evidence.
6. Continue through remaining in-scope work until the goal is met, a declared budget is exhausted, or a real repository gate blocks progress. Never call an empty queue `done` without auditing the original outcomes.
7. “Go tot alle taken klaar” freezes all unfinished in-scope task IDs, including blocked tasks, into the existing campaign controller. Show scope and dependency order; do not invent a five-task or two-hour ceiling. Explicit user budgets still apply.
8. Finish each task through claim, implementation, verification, independent review, commit, authorized push, configured deployment/readback and synchronized closure before announcing done and starting another. New authorized taskwise runs default to push; user/repository restrictions and frozen older-run authority prevail. Deployment without a requirement is not applicable; required unconfigured deployment is blocked.
9. Use the versioned progress outbox and an independently capable transport for task start, phase, repair, amendment, done and final messages. Preflight transport before unattended execution. A separate watcher offers a heartbeat every 300 seconds; no connected transport means no promised chat heartbeat. Retain undelivered events and stop before the next task until delivery recovers.
10. Resume from current canonical task, workspace, proof and publication records. Keep necessary repair tasks linked to original outcomes, show amended future tasks, and continue independent work around concrete blockers. Never count status/log churn as proven progress or expand an old run’s authority implicitly.

Do not redirect repository workflow state to a hidden central queue or retired vault. Nested `AGENTS.md` files may add directory-specific obligations, but they do not replace the root gateway or `.go` source of truth.
<!-- go-workflow:agents-gateway:v1:end -->

## Product boundaries

Own code is Rust. Validate locally with locked tests, fmt and Clippy. Do not add
GitHub Actions. Never import authentic vault content, history, credentials or
local provider configuration. Publish only generic assets and fictional fixtures.
One profile: numbered-v1, standalone projects under 20-projects, dynamic program
roots 21-* through 29-*. Local adoption debt is not a second profile.

Package updates must preserve user content and task/source identity. Explicit
preimages, drift rejection, symlink/path guards and recoverable rollback are
required. Source location never selects a target vault. Keep writers and deeper
local checks behind explicit adapters until behavior parity is demonstrated.
