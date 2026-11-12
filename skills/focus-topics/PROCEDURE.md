---
name: focus-topics
description: Review a human-curated set of attention topics, preserve snapshot-bound conversational handles and queue provenance-bound updates through the selected local provider. This is not a new task manager.
---

# Focus topics

Focus topics are a small, explicitly human-curated attention lens over existing projects, relationships or events. They do not replace daily notes, task ownership or the underlying canonical record system. The user alone adds, removes, reorders, renames or changes lifecycle membership. Frequency is evidence for an optional suggestion, never permission to activate a topic or restore a removed one.

Read `.agents/skill-adapters/focus-topics.md` from the explicitly selected target root for the actual reader, mutation compiler, queue client, canonical readback, lifecycle vocabulary and private snapshot storage. The adapter is required for live listing or mutation; without it, discuss the supplied content but do not invent a backend, write canonical records directly or use another vault as fallback. Resolve read and mutation authority separately if the provider requires that distinction.

## Review one frozen list

Obtain one status snapshot from the supported reader. Preserve canonical IDs, ordering, source version and the exact displayed address map. Show short temporary handles such as T1..Tn only for top-level topics. Address subtopics by name under the frozen parent, not invented multi-level numbers. A temporary handle is valid only for that shown snapshot; it is never durable identity.

Use the snapshot for both the briefing and the reply. Do not regenerate handles after a user answers. Keep the normal exchange compact and ask what changed; do not attach unsolicited coaching, accountability, tasks or deadlines. Include waiting, parked or completed topics only under the provider's normal selected view or the user's request.

## Interpret a reply without flattening it

Split a reply into one structured update per affected top-level topic while preserving the original message verbatim as source evidence. Bind each temporary handle to its stored canonical ID. Combine multiple statements about the same parent. Link only clearly identified existing subtopics, and extract only the factual summary and explicit lifecycle/date changes stated by the user.

A semantic contradiction between a handle and the described topic needs a short clarification; do not silently remap it or write blindly. Unaffected independent clauses can proceed. Do not infer membership from repeated mentions, turn every sentence into a task or invent sentiment, blockers, deadlines or relationships.

Before admission, build one provenance-bound envelope with actual source platform/chat/message locators where available, occurrence time and raw input. Where the source has no durable locator, use the supported provider's truthful unique source reference rather than asking the user for an internal ID. Preserve the frozen address map while separately verifying current mutation authority.

Use the supported compiler without canonical writes. Read back the compiled envelope and require one source record, one accepted observation per updated parent, only requested relations and an idempotency key. Represent lifecycle/date changes as supported deltas, not a stale full-topic replacement.

## Submit and prove

Submit the exact inspected envelope through the selected queue writer. Require its receipt and readback-verified pending bytes/hash. A durable queue admission is not a canonical commit. Say queued when that is all that has been proven; only a matching accepted canonical observation/readback supports saying saved or updated. A moved pending file or a later green process is not sufficient evidence.

Preserve private temporary input and compiled envelope on failure, under the adapter's permission and retention policy. After proven admission, remove only those temporary copies, never the pending item. Do not create a competing log, daily note, task collection or direct JSONL edit as a recovery shortcut.

## Explicit list management

Administer membership only when explicitly requested. Resolve current all-state IDs for name-based management; reuse a temporary handle only from its frozen snapshot. Keep status observations and explicit management changes in separate atomic envelopes. If an update depends on a new topic, queue the add first and wait for canonical readback before the dependent update; do not imply cross-envelope atomicity.

Retain source history when removing membership. Unknown/ambiguous identity, missing backend, duplicate parent updates, rejected envelopes, receipt/hash mismatch and contradictory canonical readback are fail-closed conditions. Report the exact unresolved dependency without mutating the underlying system.

Deeper questions are optional and evidence-backed: for example an elapsed date, explicit waiting dependency or contradiction. They are not a compulsory second phase. If requested, cite the latest accepted source and acknowledge gaps in real-world state. Finish with a human-scale acknowledgment; expose technical IDs, queued/committed distinction and readback evidence only when they matter to the result or an operational limitation.
