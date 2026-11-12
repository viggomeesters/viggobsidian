# Deep modules

A module earns its interface when it hides substantial behavior and concentrates
change. Judge depth by what callers get for what they must know, rather than
implementation line count. The interface includes invariants, ordering, error
modes and required configuration as well as signatures. A seam is a location
where behavior can vary through an adapter.

For a candidate, inspect real callers and recent change hotspots. Apply the
**deletion test**: removing a pass-through should remove complexity; removing
an earning module redistributes its hidden complexity among callers. Preserve
project vocabulary and ADRs rather than imposing universal names.

Compare interface alternatives only when the shape is uncertain. At least two
substantively different shapes should state caller usage, hidden complexity,
error/invariant contract and trade-offs in locality and testability. Sequential
comparison is sufficient; delegation follows the active task's authorization.

Use dependency type to choose verification:

- In-process computation/state: exercise the public interface directly.
- Local-substitutable dependency: use a realistic local stand-in when suitable.
- Remote owned service: use a port/adapter when real variation justifies it.
- External provider: isolate its boundary; mock or use a safe fixture there.

Internal test seams do not automatically belong in the public interface.
A hypothetical future adapter is insufficient reason to add indirection.

When interface-level tests replace shallow-module tests, map every existing
behavior first. Remove obsolete implementation-coupled tests only after their
meaningful assertions are covered or explicitly dispositioned and the relevant
suite passes. Keep distinct coverage; fewer tests alone is not success.

Adapted from Matt Pocock’s MIT-licensed skills at `f3fc5632f401156837ee3872f14fe33ccf1024ea`. Source: https://github.com/mattpocock/skills/tree/f3fc5632f401156837ee3872f14fe33ccf1024ea/skills. License: [Matt Pocock](../../../licenses/matt-pocock-skills.txt).
