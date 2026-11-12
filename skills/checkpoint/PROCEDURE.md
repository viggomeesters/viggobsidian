---
name: checkpoint
description: "Create a safe cross-session handoff for a fresh agent: capture authoritative repository state, current objective, evidence pointers, open work, and exact resume commands in a redacted temporary artifact by default."
version: 2.0.0
author: Generic Skill Library (adapted from Matt Pocock's handoff)
license: MIT
user-invocable: true
triggers:
  - "/checkpoint"
  - "checkpoint"
  - "maak checkpoint"
metadata:
  source: https://github.com/mattpocock/skills/tree/main/skills/productivity/handoff
  source_commit: 2ab958093e83e0ec752e6c1c5932da465bf23e0c
  hermes:
    tags: [handoff, continuity, git, context, safety]
    related_skills: [git, git-cleanup, github-work, wayfinder]
created_by: agent
---

# Checkpoint

## Purpose

Create a compact, evidence-linked handoff that lets a fresh agent continue
without reconstructing the session from chat. The checkpoint describes current
state; it does not become a second source of truth for plans, tasks, decisions,
or implementation.

Creating or finding a checkpoint does not create a durable pause. In a later
turn or session, follow the newest user instruction: a newer explicit `github-work`,
`ga door`, or equivalent execution request supersedes the older checkpoint
request. Use the checkpoint only as continuation evidence unless the newest
message itself still asks to stop after saving it.

Read the explicitly selected target's optional `.agents/skill-adapters/checkpoint.md`
for its scratch-path convention. Use that configured scratch directory, outside
tracked source unless its owner explicitly chooses otherwise. Do not use the
skill installation root to select a vault or a temporary destination. Only write
`CHECKPOINT.md` inside the repository when the user explicitly requests a durable
repo artifact; report that new dirty path and do not commit it unless asked.

## Workflow

### 1. Resolve the continuation target

Resolve the exact repository and read its `AGENTS.md`. Treat an argument as the
next session's intended focus, not merely as a project path. State:

- the observable objective for the next session;
- what is already complete;
- what remains open; and
- the first safe action the next agent should take.

Completion criterion: a fresh agent can distinguish done, remaining, and
blocked work without consulting the current conversation.

### 2. Capture authoritative state

Collect only read-only evidence relevant to continuation:

```bash
git status --porcelain=v2 --branch
git log --oneline --decorate -10
git branch -vv
git stash list
git worktree list --porcelain
```

When an upstream exists, include exact ahead/behind counts. Read the existing
GitHub Issue and PR for technical status and the next authorized step. Never
infer task state from chat or load a retired workflow runtime.

Inspect the smallest relevant subset of `README.md`, plans/specs, ADRs,
architecture docs, and changed files. Prefer explicit paths, commit ids, task
ids, test commands, and URLs over paraphrased copies.

Completion criterion: every state claim names its authoritative command or
artifact, and concurrent/dirty work is visible rather than silently absorbed.

### 3. Reference instead of duplicating

Do not paste content already owned by:

- GitHub Issues, PR reviews and merge evidence;
- specs, plans, ADRs, issues, or pull requests;
- commits and diffs; or
- generated reports and test logs.

Link or name the canonical path/id and add only the one-line implication needed
for continuation. Copying large source passages creates stale competing truth.

Completion criterion: each durable fact has one canonical owner and the
checkpoint contains pointers, not shadow copies.

### 4. Redact before writing

Never read secret files merely to make the handoff comprehensive. Exclude
`.env*`, credential stores, private keys, tokens, cookies, authorization
headers, and unrelated personal data. Redact sensitive values already visible
in command output or conversation as `[REDACTED]`; preserve only the fact that a
credential or user decision is required.

Before delivery, scan the artifact for common secret markers such as private
key headers, bearer tokens, API-key assignments, credential URLs, and obvious
personal data that the next session does not require.

Completion criterion: the handoff contains no reusable credential or unrelated
personal information.

### 5. Write the handoff

Use a unique absolute path under the explicitly selected target's configured scratch directory by default, for example
`checkpoint-<repo>-<timestamp>.md`. Do not overwrite another checkpoint.

```markdown
# Checkpoint — <repository>

Created: <ISO timestamp>
Repository: <absolute path>
Branch/upstream: <branch, upstream, ahead/behind>

## Next-session objective
<observable outcome and optional user-supplied focus>

## Proven complete
- <claim> — evidence: <commit/task/path/command>

## Current state
- Git: <clean/dirty/diverged plus exact paths>
- GitHub: <Issue/PR, review and merge state, when applicable>
- Runtime/deployment: <only when relevant>

## Remaining work
1. <bounded next action> — owner/source: <task/spec/path>

## Decisions and constraints
- <one-line implication> — source: <ADR/path/url>

## Verification evidence
- `<command>` → <observed result>

## Blockers and risks
- <blocker or "None proven">

## Suggested skills
- `<skill>` — <why the next agent should load it>

## Resume exactly here
1. `cd <absolute repository path>`
2. <read Issue/PR and run the relevant local validation command>
3. <first bounded action>
```

Completion criterion: all headings are present or explicitly marked not
applicable, and the first resume command is safe to execute against current
state.

### 6. Verify the artifact

Read the completed file back. Confirm:

- its repository, branch, task ids, commit ids, and paths match current state;
- no completion claim lacks evidence;
- no secret marker or sensitive value remains;
- resume commands are syntactically concrete; and
- default temporary output did not change `git status`.

Print the absolute path, line count, redaction verdict, and whether the
repository remained unchanged.

## Attribution

Adapted from Matt Pocock's MIT-licensed `handoff` skill, inspected at commit
`2ab958093e83e0ec752e6c1c5932da465bf23e0c`. This version retains the upstream
reference-over-duplication and redaction principles while adding GitHub Issue/PR
references, authoritative git evidence, exact resume commands, and a clean-worktree
default.
