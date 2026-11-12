## Rollout / Rollback Readiness Gate

When the current work is user-visible, deployment-facing, data-affecting, or changes automation/runtime behavior, include this compact launch gate before calling it done:

- **Release shape** — no-release, internal-only, feature-flagged, canary, full rollout, or manual handoff. Record the chosen shape and why.
- **Rollback trigger** — name the concrete condition that reverts or pauses the change: failed smoke, error-rate jump, p95 latency regression, data mismatch, user-visible broken flow, or security/privacy issue.
- **Rollback path** — exact revert/disable command, feature flag, previous artifact/tag, or documented manual rollback. If rollback is not practical, mark the task higher risk.
- **First-hour checks** — after deploy/release, verify the main user path, logs/console, errors, performance/cost signal, and any business/data counter the change can affect.
- **Dependency/supply-chain check** — for new or changed dependencies, inspect fit with the existing stack, license, lockfile, install scripts/postinstall risk, bundle/runtime size, maintenance, and known high/critical vulnerabilities. Do not run noisy blanket audit commands as a hard gate unless the project already uses them.
- **No-release reason** — if no version/tag/public release is needed, state the reason in the task/release evidence instead of leaving it implicit.

Suggested evidence shape:

```text
Release shape: <none/internal/canary/full>
Rollback trigger: <threshold or condition>
Rollback path: <command/flag/revert path>
First-hour checks: <what to watch and where>
No-release reason: <if applicable>
```
