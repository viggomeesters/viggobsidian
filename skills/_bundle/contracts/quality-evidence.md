# Quality evidence

Shared by audit, quality-audit, devil-review and hardening-pass. Select the requested mode; this contract adds no review round or mutation authority.

Pin the actual artifact and authorized scope against the originating request and acceptance. Inspect relevant sources, diffs, tests, logs and runtime evidence. Distinguish requested behavior from implementation conventions; missing mandatory acceptance is an in-scope gap, not future backlog. A passed check proves only its actual scope, not user acceptance, runtime loading or another host.

Report concrete findings in severity order, with source/location, observed evidence or a plausible failure path and the smallest relevant correction. Separate blockers, non-blocking concerns and unverified boundaries. If no findings exist, say so and name the strongest evidence checked. Do not manufacture findings to fill a checklist.

Read-only inspection and adversarial challenge do not authorize edits. Explicit hardening/recheck may repair the already authorized current scope; preserve external-action, writer and destructive-action gates. Re-run affected checks after repairs and assess the original requested outcome again. Pending required reviews or missing mandatory proof prevent a complete verdict. Use the selected mode's own output/verdict format; no duplicate report required.
