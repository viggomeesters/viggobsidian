# Local terminal fixture repair — Issue #74

`test_vault_terminal.patch` is a bounded compatibility change to the locally owned
`99-system/contracts/test_vault_terminal.py`. It contains only fictional fixture
data and test code; no vault notes or host configuration. It is not a distributed
asset, runtime, skill or source copy. The local tests and their provider authority
remain with their owner.

The isolated doctor fixture now includes the shared source-asset helper used by
its validators and the actual `hermes/skills/viggo` profile layout. A healthy
fixture must pass every doctor check; missing or redirected links still fail
without repair. The task-state test verifies both parallel active identities,
no arbitrary selected task, a visible ambiguity message and unchanged source
bytes. This matches the existing reader and lifecycle tests; runtime behavior is
unchanged.

The reviewed patch maps these exact SHA-256 test-file revisions:

- Before: `3209183ac3dedfa377e5871653216b153c09803d3a1bef4cc2df83419cf3d10b`
- After: `455a98dd8dd3cf16498281ecf747c17b0a85a23af9d763d3585d2fe730dbded0`

The initial terminal run reproduced two failures in eleven tests. The repaired
terminal suite passes all eleven. The related profile (7), discovery (2) and
lifecycle (65) regressions also pass: 85 tests in total. Exact patch application
reproduces the local after-file byte for byte. Checks use the
owner's actual helpers; the public repository does not contain those private
providers or claim to run their tests independently. Local preimages and full
test output stay with that owner. No new package release or GitHub Actions.
