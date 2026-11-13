---
name: audit
description: "Investigate or recheck code, products, documents or processes using evidence; report findings and repair only within explicit authorization."
---

# Investigate and verify

When a target owner or repository is selected, read its optional `.agents/skill-adapters/audit.md` for relevant local criteria and sources. The skill installation does not select that owner; load only context needed for this request and never fall back to another vault.

Resolve the chosen subject and scope from the current request. A recheck targets the original request and just-delivered result; an audit does not automatically select the last thing the agent did. Read the actual sources and operating instructions. Default investigation is read-only: recommendations are not repair, task creation or publication authority.

Define the behavior or acceptance being tested, inspect current evidence and try meaningful counterexamples. Separate confirmed failures, plausible risks and missing evidence. A reviewer should be able to reproduce an actionable finding from its trigger, location, consequence and expected behavior. Prioritise material correctness, data integrity and user impact over preference. A clean scanner or passing build does not establish semantic correctness or a usable interface.

Recheck with the original constraints and final artifacts. Repair safe gaps only where the current request authorizes repair. Re-run the relevant checks after a correction, preserve unrelated changes, and state complete or incomplete from actual evidence. Independent review is useful for risky changes; it is not another authority to widen scope.

For feedback about agent behavior, preserve the exact observed event and distinguish failure to apply an existing rule from a missing or contradictory rule. Fix application first. Do not infer a permanent personal preference from one incident or write another policy for every mistake. An authorized process correction goes to its actual maintained source, through the available skill authoring tool when appropriate, with behavioral verification. Diagnosis alone changes no skills or configuration.

Return the material findings with direct evidence and practical implications. State a real limitation when a required source or runtime is unavailable; do not substitute invented proof. For requested interface review use [visual and copy verification](../output/references/interface-check.md).
