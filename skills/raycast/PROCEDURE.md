---
name: raycast
description: Build or maintain Raycast extensions using the repository's supported TypeScript/React tooling and verified asynchronous command states.
license: MIT
---

# Raycast extension work

Inspect the target extension's manifest, supported SDK version, permissions, preferences, commands and actual build/test scripts. Use its existing TypeScript/React conventions and typed interfaces. Check installed types or official documentation before copying version-specific SDK examples. Preserve shared package boundaries and existing command names.

Optionally read `.agents/skill-adapters/raycast.md` from the explicitly selected target root for local live-testing, import and session rules. Never infer that target from the installed skill source. Where live use is prohibited or delegated to the user, build/test in the background and give a concrete manual check rather than starting the app. Absence of live permission does not prevent local type/build/test checks.

## Implementation

Use the supported Raycast components and actions. Preserve useful primary keyboard/Enter behavior, actual input and clear result states. Handle a real error with a bounded meaningful message, not an untrusted raw stack or a generic parse failure that hides a controlled partial result.

For an asynchronous script/model/network command, verify:

1. loading/progress after at least one visible update interval;
2. successful result and its primary action;
3. empty, partial or abstained results;
4. process failure with bounded diagnostics.

Keep the user's input visible during waits. Show elapsed time where a wait makes it useful and clean up intervals/resources on completion and unmount. Add focused tests for important state transitions and formatted output instead of checking only the final success screen.

For Script Commands, inspect the actual result viewport and output mode. A long report can put diagnostics first and a compact outcome/affected item/state/next action at the end where the app shows newest output. Bound wrapping untrusted fields and test that the final visible block remains useful. Do not hide an unverified downstream state behind a successful local command.

## Session and import boundaries

Do not kill another development session, start a live dev server, invoke a command/deeplink or clean a directory merely as a routine delivery step. Registration/preference changes may need a user-owned restart; identify the specific requirement. Prepare imports through the available supported payload route and verify their intended target, without executing them unless authorized.

Run the repository's available build, lint, type checks and meaningful automated tests. Inspect manifest changes and new entrypoints. For visible UI apply the relevant frontend/copygate. Report what was proven locally, any actual live readback, and the remaining manual check; do not label a user-owned live test as performed by the agent.
