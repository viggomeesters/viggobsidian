# Plugin plan and evidence outline

Use in the existing Issue/PR/repository evidence. Replace placeholders with observed values; do not create a second task registry.

- User flow, scope/non-goals, file types and input origins; viewer/importer/editor responsibilities.
- Repository, plugin ID/name, first submission or existing listing; overlap/successor decision.
- Public-source/privacy decision; local test-vault binding from target instructions.
- Architecture: lifecycle adapter, core logic, access boundary, UI, necessary dependencies/helpers and install channel.
- Candidate commit, manifest/minimum/platforms, main.js digest, exact tag/assets and rollback.
- Local commands/results: clean install, official lint, typecheck/effective types, regressions, production build/audit.
- Actual host/version/platform, fixture edge cases, physical/simulated inputs, settings/source preservation, unload and independent restoration; untested boundaries.
- Independent review and resolved findings on this candidate.
- Phase/next action: source-preview ready, public GitHub release verified, official full review/build verified, public listing/version, actual catalog installation/payload.
- Pending condition and concrete recheck date; explicitly say whether anything is actually scheduled.

The JSON [evidence outline](evidence-template.json) is input from reviewed observations, not a way to manufacture pass results. Keep false/pending until the actual operation succeeds; put report/log/review locators alongside statuses in existing evidence. The checker binds candidate and payload but does not independently execute or authenticate these supplied observations.
