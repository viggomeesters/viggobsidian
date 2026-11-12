---
name: swiftui-pro
description: Build, debug, or review native SwiftUI Apple apps with project-compatible APIs, reliable state flow, accessibility, and performance. Use for SwiftUI code, macOS menu-bar apps, and native windows.
license: MIT
metadata:
  author: Paul Hudson; adapted for project-compatible agent use
  version: "1.0.0"
---

# SwiftUI Pro

Read `.agents/skill-adapters/swiftui-pro.md` from the explicitly selected target when present for host-specific live-test, focus and storage rules. Do not derive a target from this skill's source directory. An unavailable required host adapter limits that live effect, not independent source/build review.

Use for native SwiftUI implementation and review. The existing [frontend skill](../frontend/PROCEDURE.md) owns visual direction and consumer copy; load it when designing or changing visible UI. SwiftUI Pro owns native technical guidance. Ordinary React/web work stays with its existing skills.

## Establish the project boundary

Read the target repository's AGENTS.md, product decisions, workflow contract, build configuration, and affected views/models before editing. Record actual platform, minimum OS, Swift language mode, compiler/SDK, and existing UI/state architecture. Inspect `Package.swift`, Xcode settings or build scripts as applicable; `swift --version` and `xcode-select -p` establish available tools. Full Xcode is not required when the project builds with Command Line Tools.

Keep deployment targets and toolchain requirements unchanged unless the request includes changing them. Verify unfamiliar or version-specific APIs against the installed SDK or official Apple documentation. Runtime `#available` guards cannot make an older compiler/SDK recognize a declaration. Preserve working AppKit integration, ObservableObject/Combine and serialized queues where the project's architecture requires them; modernization needs a concrete benefit within scope.

Native controls, Raycast commands, and terminal clients must use the app's existing action and writer boundary. Read the actual owner contract before storage work; a vault path does not authorize a second writer, direct JSONL writes, a new database, or sync changes. Use isolated fixture data for tests involving timers, recordings, or health logs.

## Select only relevant references

Resolve these links from this managed skill directory, after resolving profile symlinks.

| When | Read |
|---|---|
| State ownership, bindings, lists, actor isolation | [Data flow](references/data.md) |
| API replacement, Swift, localization | [API and Swift](references/api-swift.md) |
| Visible controls, appearance, keyboard or accessibility | [Accessible native UI](references/interface.md) |
| Menubalk/popover, navigation, sheets or resizable windows | [Windows and navigation](references/windows.md) |
| Routine performance review or a demonstrated slow update | [Performance](references/performance.md) |
| Explicit deep performance review or diagnosed broad invalidation | [Deeper performance](references/performance-plus.md) |

Read all six for a comprehensive native review; partial changes load only their relevant branches. For a review, report concrete defects with file/line, affected behavior, target-compatible remedy, and priority. Unrelated modernization stays a suggestion; do not report stylistic preferences as defects.

## Complete and verify

Implement only the authorized behavior through the repository's own task and checks. Recheck every affected state/action, including cancellation and errors. Run the available native build and focused tests; keep unavailable checks explicit. User-visible UI changes require native visual/interaction evidence where available, including keyboard access, light/dark appearance, narrow windows and relevant accessibility settings. Respect repository desktop-test permission rules; do independent silent checks while awaiting a required test slot. Honor the selected host adapter before taking focus, mouse or keyboard for live application tests.

Report result, changed paths, actual checks, and unverified behavior. A build does not prove installed-app behavior, profile links do not prove fresh session loading, and a receipt alone does not prove successful vault delivery. Follow the target owner's install/finish contract when deployment is in scope.

## Attribution

Adapted from Paul Hudson's [SwiftUI Agent Skill](https://github.com/twostraws/SwiftUI-Agent-Skill/tree/f9800713b24580bc444931949aad4519128605e8), upstream SwiftUI Pro 2.0.0, under [MIT](LICENSE.txt). [Provenance](provenance.json) records upstream hashes and conversion decisions. Unverified platform/hardware predictions, forced SDK upgrades, plugin installers and promotional cross-installs were excluded.
