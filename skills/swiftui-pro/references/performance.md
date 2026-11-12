# Routine performance

- Treat view initializers and body as frequent evaluations. Move costly sorting, parsing, file reads and processing to their owner, with explicit invalidation if caching. A cache that misses an update path creates stale UI.
- Preserve structural identity when only a modifier value changes. Avoid a generic conditional View.if helper that replaces the view hierarchy. Deliberate different hierarchies can be valid; assess state loss before refactoring.
- Prefer concrete views or generic composition over unnecessary AnyView. Extract a subview when it narrows dependencies or improves a complex body, not because every helper property is forbidden.
- In lazy lists, give each element a consistent top-level row container. Filter excluded elements before rendering rather than changing the number of top-level rows within a ForEach. Keep element IDs stable across filtering and edits.
- Pass a row the data it needs rather than a store/index requiring a full-collection search. Retain any row models rather than recreate them every body evaluation.
- Keep timer ticks, geometry and scroll progress out of a broad shared environment. Limit frequent updates to views that display them. Derived threshold states should change only when the threshold changes.
- Use a view-scoped `.task` for cancellable view work; use an app/domain-owned lifetime when processing must continue after a window closes. Neither window dismissal nor async cancellation may silently abandon required persistence.
- Check long histories and active timers as well as empty fixtures. Measure a concrete slow path before adding complicated observation structure; deeper changes are in [performance-plus.md](performance-plus.md).
