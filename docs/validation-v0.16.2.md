# System-guide history planning — issue #42

An approved canonical guide update planned a sibling history file which strict
layout validation could not accept. Apply refused and restored the owner.
The existing contract already supports exact operator-reviewed writer-file
bindings; no checker exemption, directory delegation or adoption reset is needed.

The maintained procedure now requires checking that structural support during
planning. The existing private canonical adapter receives a bounded maintenance
guard: a reviewed system-owner update that generates sibling history needs an
exact nonempty writer-file binding, or planning refuses before writes. The schema
is already sealed in the plan and apply refuses configuration drift. Source,
claim, owner, history, preimage and transaction guards remain in force.

The canonical adapter is existing Python. This bounded repair preserves its
exclusive writer and recovery behavior; it is not a new writer or runtime.
Actual adapter bytes, configuration, source evidence and receipts remain local.
Only the generic contract and its generated public test snapshot are distributed.

Fictional local tests cover missing/wildcard/wrong-sibling bindings, exact binding,
schema drift, preserved source content and owner/history rollback after a forced
postwrite failure. Paragraph and storage suites also pass. One mixed-source test
still refuses an unrelated owner outside its existing scope; the same failure
was reproduced with the unmodified adapter and original test, before this change.
That failure is not hidden, skipped or fixed by loosening permissions.

The reported real update now refuses during planning when unbound. After review
of one exact operator binding, the canonical transaction commits the owner and
its generated history together, preserving the previous paragraph and source
quote. Targeted strict validation accepts the pair. No authentic guide names,
capture data, source locators or private provider code are published here.
