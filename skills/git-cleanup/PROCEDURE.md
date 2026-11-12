---
name: git-cleanup
description: Make a repository ready for a reliable handoff by classifying dirty state and applying only authorized, evidence-backed cleanup while preserving user work.
---

# Git cleanup

The outcome is handoff readiness: another authorized machine can continue from durable state, or every remaining local-only item is explicitly preserved and explained. A clean working tree alone does not prove this. Read the target repository's instructions and snapshot ownership before changing anything.

When present, read `.agents/skill-adapters/git-cleanup.md` from the explicitly selected target root for its handoff checker, branch retention and publication policy. The absence of an optional checker permits equivalent manual evidence, never invented readiness or wider permission. Never infer a repository or rights from this procedure's source path.

## Snapshot and classify

Collect the relevant read-only evidence:

```text
git status --porcelain=v2 --branch
git stash list
git worktree list --porcelain
git branch -vv
git log --branches --not --remotes --oneline
git rev-list --left-right --count HEAD...@{u}
```

Run the final command only when an upstream exists. Classify staged, unstaged and untracked content; stashes; local-only commits; ahead/behind/diverged branches; unmerged work; unresolved worktrees; release drift; and relevant generated/ignored artifacts. Inspect contents and provenance rather than assuming that ignored or untracked means disposable.

State a scoped action, evidence and stop condition per group. Apply one safe group at a time and resnapshot afterward. Finish only when clean, clean-or-explained, or blocked with the exact next action. Keep all ambiguous items visible with their actual owner/uncertainty.

## Safe action boundaries

Stage explicit paths only when ownership and scope are clear. Commit reviewed intentional changes, preserving unrelated staged state. Inspect stashes rather than dropping them. Delete a branch/worktree only when it is proven clean, merged/disposable and authorized; a stale name is not proof. Never rewrite history, reset/clean away user work or drop a stash as a diagnostic shortcut.

Pushing requires the actual authorization and correct remote/upstream; cleanliness alone does not grant it. If sharing was not authorized, preserve commits locally and report that boundary. In a conflicted or interrupted merge/rebase, recover both primary intents with the available Git conflict procedure; do not abort automatically or choose one side wholesale merely to remove markers.

Stop the dependent cleanup when history intent is unclear, ownership is ambiguous, local-only work cannot be preserved or a destructive action would exceed scope. Independent read-only classification may continue. Do not create a new global cleanup engine or hosted CI gate.

## Prove the handoff

Run the target's available read-only handoff checker or equivalent evidence. Verify final status, branch/upstream, ahead/behind, stashes and relevant worktrees. Name preserved local-only state and the exact steps needed on another computer. A no-readiness verdict is valid when supported by concrete blockers; do not label it ready to shorten the report.

Report the practical result first, then relevant cleanup applied, preserved or blocked items, and the final handoff verdict with evidence. Do not dump an unrelated repository inventory when a small scoped cleanup answers the request.
