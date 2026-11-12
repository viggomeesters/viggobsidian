---
name: python
description: Maintain an existing Python project or implement an explicitly selected Python task using its actual supported runtime, tooling and distribution contract.
license: MIT
---

# Python maintenance and scoped implementation

A Python file or available interpreter does not authorize a language/toolchain migration. Read the actual repository instructions, supported Python versions, package manager, lockfile, build backend and checks. Preserve a working existing setup unless the request includes changing it.

Optionally read `.agents/skill-adapters/python.md` from an explicitly selected target root for user language policy and approved tooling. Never resolve the target from this installed source root. Apply that policy before selecting a new script, test, generator or inline implementation. If the requested language conflicts with a controlling rule, resolve that specific conflict before code creation; do not infer an exception from convenience.

## Select the smallest useful work

- Existing module or script: use the repository's established manager and runtime. Avoid a package scaffold for a small maintained script.
- New explicitly authorized script: prefer a simple supported execution shape. If external dependencies are necessary, use the selected environment or a supported script dependency declaration rather than global installs.
- New package: select a build backend, runtime range and metadata that fit actual consumers. Existing supported tools are valid; do not impose one package manager, all lint rules or arbitrary coverage percentage.
- Tooling migration: preserve metadata, package contents, commands and recoverable environment state until the replacement is verified. Treat lockfile, dependency groups and entrypoints as compatibility surfaces.

Inspect actual tool versions/help or primary documentation before using version-specific flags and APIs. Add dependencies through the project's manager and lock mechanism. A one-off tool environment is preferable to modifying the project when it is supported and sufficient. Do not install hooks, remote automation, CI providers or publication settings as incidental setup.

## Verify behavior

Run the changed behavior's meaningful tests, actual entrypoint and relevant lint/type checks. Respect existing unittest/pytest and mypy/pyright/other supported conventions rather than rewriting to match a preference. For a packaging change build and inspect wheel/source distribution contents, metadata and executable entrypoints; a generated configuration alone proves none of those.

Keep imports, serialization, exceptions and asynchronous behavior consistent with supported Python versions and actual callers. Avoid replacing an established API solely to shorten an example. Report changed paths, commands actually run, behavior evidence and material missing checks. New runtime/dependency/security claims must be checked against their actual current sources.
