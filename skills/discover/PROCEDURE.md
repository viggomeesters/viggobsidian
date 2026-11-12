---
name: discover
description: Inventory local Git repositories under an explicitly selected root without registering projects or changing repository configuration.
---

# Discover repositories

Resolve the requested inventory directory. A target-root adapter at `.agents/skill-adapters/discover.md` may supply a known local default; never infer a private repository root from the installation location of this skill. Ask for a missing root only when it cannot be established from the actual request/context.

List directories in that bounded scope containing a `.git` directory or valid worktree pointer file. Avoid following arbitrary symlinks outside the selected root. Read each repository's own identity, origin and local instructions; a remote URL or directory name alone is not proof of the repository's current work status.

Report actual repository paths and missing/relevant contracts. Preserve unreadable/unavailable paths as explicit gaps instead of claiming complete coverage. This inventory is read-only: do not add repositories to a retired/global registry, create tasks, fetch remotes, change Git config or mutate project files merely to discover them.
