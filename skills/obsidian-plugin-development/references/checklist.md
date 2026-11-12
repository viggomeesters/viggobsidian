# Architecture and release checklist

Select only relevant product features. This checklist preserves useful older plugin-development guidance while replacing stale registry/workflow requirements. It does not make a viewer's no-write rule apply to an explicitly authorized importer/editor.

## Before building

- Identify the user flow, actual file types, read/write/network/process scope, intended platforms and install channel. Check existing own/public plugins and choose one functional owner; do not publish an obsolete duplicate merely to fill a catalog slot.
- Separate the Obsidian view/lifecycle adapter, pure parsing/validation logic, bounded file access and presentation. Use public APIs where available. Document necessary private APIs and test ownership/interoperability before maintaining a native patch.
- Binary/archive/database formats use appropriate binary APIs and guarded parsers; text APIs are for text. Reject traversal, escapes and unsafe URL/active-content handling. Preserve original bytes and report unsupported/empty/malformed/large input explicitly.
- Dependencies: review licenses, bundled bytes, transitive/runtime behavior and necessary external tools. Community normally installs main.js, manifest.json and optional styles.css; an extra executable/ZIP is not automatically installed. Do not self-install/update plugin dependencies. Prove a separately required helper has a supported, documented route before calling the Community install complete.
- Use the selected isolated test vault, preserve existing fixtures/settings/plugins/active note and prefer terminal/background controls. Never silently install/test in a main user vault. Authentic sources and local paths stay out of public source/history/screenshots.

## Local candidate

- Root manifest: stable unique lowercase-letter/hyphen ID without obsidian/plugin suffix, valid short name and action description, x.y.z version, actual API minimum, author/license and truthful desktop/mobile flag. Node/Electron runtime use requires desktop-only. Read current official requirements; this list is not a frozen policy substitute.
- Package and both lockfile version roots agree with manifest; current versions.json mapping is explicit in this package's release contract. Standard source/output/documentation files are present. styles.css is optional when unused.
- Run ordinary **npm ci** with the committed lockfile and no exception flags, official Obsidian lint, real TypeScript checks, meaningful tests, production build and production dependency audit. Confirm effective library/import types; implicit Node globals can hide missing browser library declarations such as String.matchAll/ES2020. Use portable build commands. Reproduce without accidental local tools.
- Inspect source and the actual bundle for unexpected network, telemetry, child processes, writes, eval/new Function, polyfills or unbounded content. Distinguish legitimate declared behavior, unsupported features, style recommendations and actual security/build failures. Do not suppress rules globally to turn a failure green.
- Tests exercise real failure behavior, not just helpers shaped like the implementation. Cover normal/empty/malformed/large/unsupported sources, path/link safety, unload, timers/listeners/URLs/child cleanup, retries/cancellation, duplicate/concurrent events and interactions with related plugins where relevant.
- Actual-host evidence names app/installer/platform, candidate version/payload digest, native versus simulated input, source-byte preservation, settings persistence and exact restoration, reload/unload, errors and the promised main flow. Build, SVG mockup, synthetic event and compiled cross-platform binary are different claims. Keep experimental Live Preview opt-in until actual acceptance.
- Render the real UI/settings and check controls, labels and errors. On current Obsidian test declarative settings as well as the older display fallback. Search metadata alone does not prove working switches. Use scoped theme-respecting CSS and the applicable copygate. A hero is optional useful communication, never runtime evidence; actual screenshots must not leak private tabs/content.
- Independent review resolves material findings on the exact final candidate. Preserve a verified rollback path and record changes after the tested/reviewed source.

## Before first Submit or update publication

- Follow the separate first-registration/update sequences in the main procedure. For a first submission: public exact GitHub release first, then registration. For an existing listing: use an exact-source branch preview where available, then release and full review.
- Resolve the **intended** repository explicitly; never invent OWNER/obsidian-ID from a naming convention. Validate actual default-branch manifest, exact tag and dereferenced commit. A release title or target_commitish=main is not identity.
- Release is public, non-draft, appropriate stable channel and includes main.js, manifest.json and used styles.css. Download anonymously, compare bytes/digests and parse the downloaded manifest. Do not replace immutable published assets to hide a mismatch.
- Read the official scanner's actual findings. A build-verification Warning is still a real build defect; a recommendation is not necessarily a blocker. Resolve source/library/dependency/build problems and repeat affected checks. Full release reproduction is separate from preview.
- Verify public listing/version and a fresh actual catalog install/upgrade with the installed bytes. Respect the exact known app transformation only; do not accept arbitrary differences. Preserve existing settings, fixtures and cooperating plugins.
- Keep pending propagation and inaccessible services visible, with a concrete next check. Do not classify 403/network errors as a missing release or successful empty scan. Do not create unrequested timers/services/CI to hide a pending gate.

## Repository outline (adapt to actual architecture)

```
manifest.json, versions.json, package.json, package-lock.json
src/main.ts                 # lifecycle/platform adapter
src/<format-or-domain>.ts   # independently testable core
fixtures/, tests/           # synthetic edge cases and regressions
scripts/                    # existing host/build interfaces where necessary
README.md, LICENSE, CHANGELOG.md, SECURITY.md, CONTRIBUTING.md
docs/                       # supported behavior, publication and evidence
main.js, styles.css         # generated release bytes; CSS only when used
```

Follow the selected repository's language policy. Obsidian's TypeScript/JavaScript host adapter is a concrete technical interface exception; do not force a native executable into a simple plugin or rewrite existing code merely to satisfy a preferred language. New reusable tooling can use the target's normal language/architecture. No extra task registry or mandatory copied plan note.

## Current primary sources (re-read for each release)

- https://docs.obsidian.md/community-directory/developer-policies
- https://docs.obsidian.md/community-directory/submission-requirements-for-plugins
- https://docs.obsidian.md/Reference/Manifest
- https://docs.obsidian.md/Reference/Versions
- https://docs.obsidian.md/Plugins/Releasing/Submit%20your%20plugin
- https://github.com/obsidianmd/eslint-plugin
- https://obsidian.md/blog/future-of-plugins/
