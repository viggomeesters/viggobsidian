---
name: ops
description: Inspect and maintain a computer or runtime using current measurements, bounded reversible actions, owner-aware process handling and verified recovery.
license: MIT
---

# Operations

A status question authorizes inspection, not a restart, update or configuration write. For an execution request, work through the smallest effective authorized actions and preserve ongoing work. Do not create cleanup routines, timers or permanent permissions as a side effect.

With an explicit target root, optionally read `.agents/skill-adapters/ops.md` there for host tools, known retention decisions, current incident history and supported runtime providers. Do not resolve it from this installed skill's directory. Missing host/provider knowledge blocks the affected destructive action; ordinary read-only measurements may continue.

## Measure before selecting an action

Use existing supported tools and a small current sample of CPU, memory pressure, swap activity, disk capacity and the relevant process tree. Compare repeated measurements where a transient could mislead. Occupied swap, a sleeping process or high historical RSS alone does not prove a performance fault.

For disk work, prefer the existing host inventory and retention history over another whole-home scan. Inspect only the best current candidates. Distinguish logical file size, shared filesystem blocks and measured physical capacity. A move on the same volume is not freed space.

Rank a few concrete opportunities by likely user-visible effect, data-loss/interrupt risk, verification cost and restoration cost. Separate memory/CPU benefit from disk capacity. Do not add those quantities or manufacture precision from unmeasured scores.

## Preserve active owners

Group a process family by executable, parent/children, start time, current session and supported app/runtime state. Revalidate identity immediately before a stop; PIDs can be reused. An orphaned parent, age or low CPU does not prove a task is released. Check available pipes/files/session evidence without dumping prompts or secrets.

Stop only an authorized, proven released process family through its supported shutdown or bounded graceful termination. Confirm the result and how to restart. Do not escalate automatically to a force kill, disable a service, reset an account or interrupt transcription/build/sync because a general speed request was made. When current ownership cannot be proven, choose another opportunity or identify the specific interruption decision.

Before changing a retention or runtime setting, inspect the installed version's actual behavior and preserve an exact preimage. A configuration on disk does not prove a running backend loaded it. Test activation with the relevant supported route; preserve required tool/browser runtimes during cache cleanup.

## Verify and retain evidence

After each meaningful action compare under reasonably similar load. Measure actual disk capacity separately from process RSS. Verify the affected app/task/tool route and recovery. Report the concrete outcome and any remaining boundary, without promising speedup from an instruction or configuration edit alone.

Store concise owner-scoped evidence when authorized: selected item/process identity, before/after measurement, action, result and restoration route. Do not log full arguments, prompts, credentials or unrelated inventories. Keep existing supported cleanup history rather than inventing a second register. Stop when the requested result is met or the best remaining action needs a real human choice.
