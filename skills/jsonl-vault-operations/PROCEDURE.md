---
name: jsonl-vault-operations
description: Route explicitly requested existing JSONL-runtime operations through their declared authority and supported writer or queue, never as a fallback for native work.
license: MIT
---

# Explicit legacy JSONL operation

Select this route only when the request explicitly concerns an existing JSONL runtime, bounded legacy import, recovery or historical operation. Directory presence, an old example or an unavailable native provider does not select legacy execution.

Read `.agents/skill-adapters/jsonl-vault-operations.md` from the explicitly selected target root for its current root resolver, exclusive writer/queue and exact bounded recipe. Never resolve the target or runtime from this installed skill's source directory. Without that required adapter, stop the dependent runtime effect and preserve the user's requested scope; do not invent an alternate writer.

Resolve the active code/runtime/data locations through the declared locator and verify actual host/writer authority. Revalidate historical commands against current runtime contracts. Use only its supported canonical writer or producer queue; never directly edit record files, dual-write another system as fallback or activate retired producers.

Before retries/corrections inspect exact source/payload identity and the current processing state. Respect idempotency and lock/provider rules. Verify returned queue/record/receipt/blob identities and exact stored hashes as required. Queued is not committed, a regenerated view is not a new canonical fact, and a restored evidence archive is not a live runtime replacement.

Report the bounded operation, resolved roots, actual proof/state and material remaining limits. Runtime administration, backup scheduling and unrelated migrations are separate effects and are not authorized by this route.
