---
name: check-in
description: Capture a comparable daily self-report snapshot of feeling, energy/sleep, body, situation and current topics; finish it independently and offer optional coaching.
---

# Daily check-in

Resolve the explicitly selected personal context and read `AGENTS.md` plus an optional `.agents/skill-adapters/check-in.md` there. The adapter supplies the actual session owner, index, storage contract and question-format preferences. The skill installation root is not a context source. Without a required storage provider, state the persistence gap; never use another vault or a legacy writer as fallback.

Read existing current answers and, when resuming, the complete selected session with corrections and phase choices. Do not ask questions already answered. Keep the exact question set bound to its responses: option letters mean what the question actually offered, not a generic global scale. Free text, skipping and optional scores remain valid.

## Build one snapshot

Cover feeling, energy/sleep, bodily state, situation and current or pleasant topics with a small set of concrete questions. Ask only missing information and follow the local maximum/question preference. Record the actual local date/time, source identity, self-report and unknown fields. Distinguish self-reported observations from interpretations.

A second measurement is a new timestamped snapshot, not an overwrite. A resumed old session is not automatically a new day. Do not infer missing values from a previous day or from a vague emotional statement.

Use the real local storage contract where authorized, preserving exact input and actually shown output, provenance and readback. Do not claim a cache or queued write is final persistence. Conversation storage and accepted durable personal facts remain distinct.

## Finish without silently starting coaching

Summarize the snapshot factually. A meaningful comparison may identify a difference from a comparable earlier self-report, with its evidence and uncertainty. Do not add psychological analysis, a pattern explanation, debate round, treatment question, homework or a mandatory deeper menu in the snapshot phase.

Mark the snapshot complete or partial with explicit unknowns. This does not decide coaching. Offer a brief optional choice to continue with [coach](../coach/SKILL.md) or stop here. Emotional input, topic lists, scores, silence and an unanswered old offer are not consent. If coaching was already explicitly requested, retain that choice instead of asking again.

A declined offer closes the snapshot without another invitation in the same ending. An added snapshot detail stays a check-in until coaching is chosen. Record any selected phase transition in the same session according to its owner; the check-in remains complete even if coaching occurs later.

This skill creates no reminder, scheduler, daily-page mutation or health/task record as a side effect. Those domains retain their own authority and explicit request boundaries.
