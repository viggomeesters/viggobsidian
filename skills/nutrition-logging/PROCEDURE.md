---
name: nutrition-logging
description: Record explicitly requested food and nutrition data with source-backed quantities, calculations, uncertainty and a supported domain writer.
license: MIT
---

# Source-backed nutrition logging

Resolve the current nutrition owner and supported writer from `.agents/skill-adapters/nutrition-logging.md` in the explicitly selected target root. Never infer that target from this installed skill's source root. This procedure installs no food database, writer or retired consumption automation. If the writer is missing, an explicitly authorized source capture may remain a capture; do not label it logged nutrition or a verified day total.

Use the user's exact quantity and supplied nutrition value first. Reuse established product facts/aliases where identity matches; then use package evidence or appropriate trusted sources. If an estimate is necessary, mark its basis and uncertainty. Do not claim precision unsupported by the portion or product identity.

Preserve food/product, actual portion/unit, energy and requested macronutrients/fibre/diversity fields, date/time when known, source identity and uncertainty. Deduplicate by the actual source message/entry before adding anything. A similarly named meal is not necessarily the same record.

Calculate item values and any requested full logged-day subtotal using available arithmetic tools and actual domain records. Name missing/unresolved entries and report partial totals as partial. Do not count a human daily note as a nutrition ledger or edit it as a side effect. Recheck unit conversion and whether a nutrition value is per serving, per unit or per mass.

Use only the selected supported writer and verify exact stored entry/readback. Historical JSONL recipes are loaded only for explicitly requested legacy operations, never because their directory exists. Do not activate old consumption scripts or change provider authority to make a log succeed.
