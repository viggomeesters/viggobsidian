# Transaction validation

Use `vault-layout --vault ROOT --write-set` with a JSON request on stdin for
bounded writes. Keep `--strict PATH`, `--check-all` and `--check-work` for full
audits. An unrelated sibling's existing errors must not block a transaction.

The request schema is `viggobsidian.write-set.v1`, with `phase: baseline` and
`changes: [{"path":"30-topics/example/example.md","expect":"present"}]`.
Use `absent` for deleted files. Include every created, updated and deleted file,
including both ends of moves and changed parent navigation. Paths must be
visible relative normalized file paths without symlinks. Hidden provider state
is validated separately by its owning writer.

Capture the baseline before mutation and bind the entire JSON report into the
writer's sealed plan alongside exact preimages, checker binary/hash and input
configuration pins. After mutation, submit the same changes with `phase:
validate` and `baseline` containing that report. Exit 0 means accepted, 1 means
findings, 2 means invalid input or execution failure. Any failure requires the
writer's existing rollback; never rewrite plan pins to bypass drift checks.

Each changed file owns its direct folder. Raw/Wiki changes own their complete
dossier, including source inventories and processed-source references. Ancestor
navigation is checked without recursively visiting unrelated siblings. Changed
owners receive all strict checks; they never inherit baseline exceptions. Only
exact unchanged ancestor findings may remain. New boundary findings fail.

This interface supplies validation, not write authority. Local adapters retain
their existing routing, locking, source registry, preimage and rollback duties.
The caller must seal the baseline: it is evidence, not a trusted user exemption.
