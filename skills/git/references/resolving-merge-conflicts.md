# Resolving Merge and Rebase Conflicts

Adapted from Matt Pocock's MIT-licensed `resolving-merge-conflicts` skill,
inspected at commit `2ab958093e83e0ec752e6c1c5932da465bf23e0c`:
https://github.com/mattpocock/skills/tree/main/skills/engineering/resolving-merge-conflicts

## 1. Freeze and classify the operation

Before editing, capture:

```bash
git status --porcelain=v2 --branch
git status
git log --graph --oneline --decorate -20
git diff --name-only --diff-filter=U
```

Identify whether Git is merging, rebasing, cherry-picking, or reverting and
which commit is being applied. Snapshot unrelated staged, unstaged, untracked,
stash, and worktree state. Do not stash, reset, abort, or switch branches as a
diagnostic shortcut.

Completion criterion: the operation, incoming commit(s), conflicted paths, and
unrelated user work are explicitly known.

## 2. Recover both primary intents

For every conflicted file, inspect:

- the common-base version and both side versions;
- the commits that introduced each side (`git log`, `git show`, and blame where
  useful);
- the originating GitHub Issue, PR, spec, ADR or test when referenced; and
- nearby callers/tests that define the current contract.

Commit messages are clues, not proof. Prefer the artifact that originally owns
the requirement. Do not invent new product behavior during conflict resolution.

Completion criterion: each side's intended behavior and invariant can be stated
in one or two evidence-backed sentences.

## 3. Resolve hunk by hunk

Classify each hunk:

- **compatible** — combine both intents with the smallest coherent edit;
- **superseded** — retain the newer intent only when primary evidence proves it
  deliberately replaces the other;
- **incompatible** — choose the side matching the stated merge/rebase goal and
  record the displaced trade-off; or
- **unclear** — leave the hunk unresolved and request the missing decision.

Edit only classified hunks. Search the whole file for conflict markers and
inspect the resulting diff before staging that path. Stage explicit paths; never
use blanket staging while unrelated work is present.

Completion criterion: every staged resolution preserves both compatible intents
or names evidence for the deliberate choice, and no conflict marker remains in
that path.

## 4. Prove the integrated result

Discover and run the repository's relevant checks, normally in this order:

1. syntax/format or generated-file consistency;
2. type/static checks;
3. focused tests for conflicted behavior; and
4. the full relevant suite.

Also inspect `git diff --check`, the staged diff, and `git status`. A passing
test suite does not prove that a lost requirement was preserved; reread the
primary sources after the diff is assembled.

Completion criterion: checks pass and each resolved intent is still observable
in code, tests, or documented decision evidence.

## 5. Continue safely

Continue the merge/rebase/cherry-pick/revert only after all conflicted paths are
resolved and validation is green. If a later rebase commit conflicts, repeat
the full intent recovery for that commit; do not reuse the previous answer by
analogy.

Stop without aborting when primary intent is missing, a secret-looking path is
involved, unrelated work would be overwritten, or resolution requires a product
decision beyond the stated operation. Report the exact continuation or decision
needed. Abort only when the user explicitly chooses it after seeing the
consequences.

Final evidence must include the operation completed, commits integrated,
conflicted paths, resolution rationale, checks run, final branch/upstream state,
and any preserved unrelated work.
