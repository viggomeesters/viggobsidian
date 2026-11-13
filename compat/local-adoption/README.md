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

## Source-audit fixture repair — Issue #76

`test_skill_sources.patch` repairs the local source-audit test that still used
the removed `obsidian` identity and a nonexistent installed target. Its isolated
package now declares the current `vault` core and contains an actual central
source. The same test rejects a standalone core copy, accepts its central link,
then reports a broken target without removing the link. This exercises the real
checker without relying on the host's installed core registry. Runtime detection
is unchanged, and no obsolete skill identity is reintroduced.

Exact SHA-256 revisions of `.agents/tests/test_skill_sources.py`:

- Before: `5a78378b2e354ec25edd57d79b75e0fd1e778cd89a64e243fbd933781e872fdf`
- After: `fa5fd908e586ab33f612f94a39a9b91bccdcf0e1e34611ad61c3478770dbfb51`

Only generic fixture/test context is published. The locally owned test and
private providers remain local; this patch is not a released asset or another
runtime. The previously observed source-audit failure is reproduced before
repair and all six source-audit cases pass afterwards.

The full local rerun passes all 353 relevant tests: 213 public workspace tests,
25 bundle, 11 terminal, 7 profile, 2 discovery, 65 lifecycle, 24 vault and 6
source-audit tests. Both installed-manifest checks, all installed asset hashes,
host-source checks and the canonical profile install preview also pass. These
results cover skill adoption and its regression set, not unrelated vault content.
