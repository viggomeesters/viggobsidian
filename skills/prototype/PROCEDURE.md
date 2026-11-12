---
name: prototype
description: Build disposable runnable evidence for one unresolved design question using a small logic/state harness or comparable UI variants.
license: MIT
source: https://github.com/mattpocock/skills/tree/main/skills/engineering/prototype
source_commit: 2ab958093e83e0ec752e6c1c5932da465bf23e0c
---

# Disposable prototype evidence

Use only when a specific uncertainty cannot cheaply be resolved by reading, discussion or an existing test. Debug an actual defect through the debug route; implement an already-decided authorized production change through the repository's normal workflow. A prototype is not an early production commitment.

Read the target's instructions, tooling and dirty state. Preserve unrelated changes. State one falsifiable question and the evidence that would settle it. Choose the cheapest useful question rather than open a prototype for every possible concern.

Optionally read `.agents/skill-adapters/prototype.md` from the explicit target root for language/tooling policy and local decision-owner conventions. Never use the installed source root as that target. Missing local tooling blocks the dependent execution; it does not justify unapproved languages, services or runtime changes.

## Logic or state uncertainty

Put the behavior behind a small portable interface: pure functions, reducer, state machine or equivalent existing-language module. Use the thinnest terminal/browser harness to drive representative cases. Show the relevant state, last action and available choices after each action. Exercise cases likely to overturn the design: empty state, repeats, invalid order, cancellation, conflicts, exhaustion and reset where relevant.

Use fixtures or memory by default. Do not add databases, production writes, migrations, service dependencies, a broad CLI framework or comprehensive hardening merely for the harness. Provide one real run command with the repository's supported tooling.

## UI structure uncertainty

Keep representative content and data constant across variants. Compare genuinely different hierarchy, navigation, density, disclosure or interactions rather than cosmetic color changes. Use a small number of variants proportionate to the decision. Prefer an existing route/context when safe; isolate only when necessary.

Make each choice linkable and survive refresh when practical. Provide a small clear switcher and keyboard support if the framework makes that cheap. Do not let comparison controls mutate production data. Hide disposable controls from production and avoid unrelated backend/schema changes. Apply useful frontend design judgment without turning a disposable comparison into a production accessibility/browser certification exercise.

## Decide and dispose

Make the complete consequential state visible. When acceptance needs human judgment, present evidence and wait for the actual judgment; the agent must not supply it on the user's behalf. Record the chosen direction, rejected alternatives and remaining uncertainty with the existing owner.

Promotion means reimplementing to production standards or lifting a deliberately small verified logic unit. Remove variant switchers, fake mutations, losing variants and disposable fixtures from the production change. Preserve or discard the prototype according to its owner policy. Do not switch/reset/clean an existing dirty checkout to isolate it.

Report the question, branch, exact run command, demonstrated cases/variants, observed evidence and human verdict if available, remaining uncertainty and disposition: promote, iterate or discard. If judgment is pending, the decision remains incomplete.

Adapted from the MIT-licensed upstream prototype pattern at the pinned source revision. Preserve attribution and the relevant upstream license when redistributing source-derived materials.
