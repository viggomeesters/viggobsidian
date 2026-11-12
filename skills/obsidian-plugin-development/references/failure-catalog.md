# Common failure diagnosis

| Symptom | Actual check and correction |
|---|---|
| No release matches your manifest version | Read default-branch manifest, exact release tag, draft/prerelease state, intended repository and public assets. A tag or release title is insufficient. Before first Submit, publish and verify the exact GitHub release. During an existing-plugin branch preview a missing candidate release is expected; do not call that delivery. |
| v1.2.3 versus 1.2.3 | Require the literal manifest version as tag. Preserve an existing published version; prepare a correctly versioned new release instead of replacing assets/history. |
| Correct local metadata, wrong Community version | Compare the intended default branch and downloaded release manifest; distinguish website refresh, app registry propagation, installed plugin metadata and an actually loaded instance. |
| Local build/lint passes, scanner reports unsafe types | Inspect effective tsconfig libraries and imports/peers first. Explicit ES2018 plus String.matchAll requires appropriate ES2020.String/later libraries; accidental local Node types are not a fix. Do not broadly suppress unsafe rules. |
| Clean install fails unless --legacy-peer-deps | Fix the actual lockfile/peer dependency contract and run normal npm ci. An exception-assisted local install does not establish scanner compatibility. |
| Official release build differs/fails after clean preview | Reproduce the exact tag in a clean environment without undeclared system tools. Keep standard plugin builds independent of external language helpers unless explicitly supported. Recheck the full release, not only the source preview. |
| Extra native helper present in a ZIP but missing after app install | Community installs the standard plugin assets, not arbitrary helper assets. Prove a separately supported explicit installation path or redesign packaging. Do not self-download/install dependencies. |
| HTML/data files execute or reach the network | Trace actual rendering/navigation and metadata-controlled MIME. Use inert text or a suitable sandbox; do not trust a no-fetch source scan as a complete no-network guarantee. |
| Render toggle seems off but rendering continues | Check every observer/postprocessor/editor extension, reload semantics and already scheduled work, not only the settings boolean. |
| Old links changed, slow/duplicate drops missed | Test native completion timing, matching filenames, existing token protection, concurrent edits, multi-file events and managed-importer ownership. A synthetic immediate drop does not establish physical-provider behavior. |
| Plugin enabled but receiver/view unavailable | Inspect the actually loaded instance and its live status. An enabled-ID set may survive unload. Restore each state independently and surface cleanup failure. |
| Native explorer patch breaks another plugin/on unload | Track patch ownership and active state; do not overwrite later patches or allow delayed callbacks to reapply after unload. Prefer a public-API view when it fulfills the intended behavior. |
| CLI host smoke times out only in background | Account for renderer background timing without taking user focus. If timer policy changes for the test, restore the exact prior policy independently in finally; retain failure evidence. |
| Settings are indexed but controls disappeared | Inspect actual declarative rendering and the legacy fallback. Names without renderers are not a settings UI. |
| New public listing, ID absent from app registry | Record a genuine failed app-catalog attempt, the current official propagation window and recheck date. Restore verified files if needed and label them manual. No new release for cache delay alone. |
| Private repo cannot be submitted safely | Review source/history/fixtures and obtain the concrete visibility/export decision. A clean public source export may preserve the private original; never bulk-publish authentic data/history. |
| Screenshot/hero looks good, runtime unverified | Separate mock/asset render evidence from real host behavior and source/payload binding. Check actual windows/pop-outs, settings and supported platforms. |
| Recommendation asks for unavailable/forbidden CI | Assess the recommendation's actual meaning. Keep local validation and preserve the target's infrastructure policy; do not enable Actions to turn an optional recommendation green. |

These are diagnostic patterns, not permission to apply a viewer's no-write rules to an authorized importer, to restore retired registries/workflows, or to claim a particular platform was tested. Keep lessons generic here; authentic incidents, source hashes, host paths and decisions remain with their private/repository owner.
