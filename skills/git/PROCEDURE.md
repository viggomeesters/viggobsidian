---
name: git
description: Safe Git compatibility route for scoped Issue, branch, PR, local validation, review and merge, including evidence-based merge/rebase conflict resolution.
---

# Git

Use the current `github-work` repository procedure for authorized Issue → branch → PR → local checks and independent review → merge work. Preserve explicit plan-only or read-only requests. Retired workflow engines, claims and contracts are not prerequisites or fallback routes.

Read `.agents/skill-adapters/git.md` from the explicitly selected target root when present. It supplies local publication, branch-retention and handoff-check conventions; otherwise follow the actual target repository's rules. Source location does not select a repository. Before commit, push, branch deletion, stash removal or rebase, snapshot state and ownership. Preserve unrelated work and stage explicit reviewed paths.

## Merge and rebase conflicts

For unmerged paths or an interrupted merge, rebase, cherry-pick or revert, follow [the conflict-resolution reference](references/resolving-merge-conflicts.md). Recover common base, both side versions, originating commits and primary requirements. Resolve one hunk at a time from intent; never choose ours, theirs or a larger side wholesale simply to make the file parse.

Finishing the operation is in scope when the user requested resolution/continuation and every hunk's intended behavior is evidenced. If intent is materially ambiguous or user work could be lost, preserve the conflict state and identify the exact hunk, sources inspected and required decision. Do not reset, clean, discard or automatically abort unrelated work.

## Handoff evidence

Before claiming another-computer readiness, use the target's available read-only handoff checker or equivalent manual evidence. A plain clean `git status` does not establish that local commits, stashes or worktree state are durably available. Report branch/upstream, ahead/behind and concrete blockers at a scale appropriate to the request. A source edit or successful push is not proof that a running product loaded the resulting code.
