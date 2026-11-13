# Obsidian plugin preflight

`plugin-check` is read-only: it collects candidate and optional installation
bytes, checks metadata and evaluates supplied local/host/review evidence. It
does not build, install, change visibility, publish or perform a Community
submission. Use an isolated candidate checkout and preserve the JSON result.

```sh
viggobsidian plugin-check --repo-dir DIR --github OWNER/REPO \
  --candidate COMMIT --phase source-preview \
  --preview-intent local-private --evidence EVIDENCE.json
```

`--phase` is required. `--candidate` defaults to `HEAD`. `--preview-intent`
defaults to `public-review`, preserving existing public-source requirements.
`local-private` is accepted only with `source-preview`; it is an explicit scope
choice, not a factual claim that a repository is private. Unknown intents and
private intent on publication phases are errors before collection.
Both `--preview-intent VALUE` and `--preview-intent=VALUE` are supported.
Missing or unknown values and repeated intent options (including mixed forms
or identical values) are errors before filesystem or remote collection.

| Phase and intent | Required proof | Maximum claim |
| --- | --- | --- |
| `source-preview`, `local-private` | Exact clean candidate, local assets/configuration and supplied local, host and independent-review evidence | `private-local-preview-only` |
| `source-preview`, `public-review` (default) | Same local proof plus accessible public source metadata | `ready-for-source-preview-only` |
| `public-release`, `public-review` | Same local proof, public source, default manifest, exact stable public tag bound to candidate, anonymously downloaded matching assets | `public-GitHub-release-only` |
| `catalog-install`, `public-review` | Public-release proof plus release review, actual catalog repository/version, matching installed files and supplied fresh Community install evidence | `catalog-installation-acceptance` |

Local-private preview performs no remote metadata, release, asset or catalog
requests. Its `public-repository` and `public-release` rows have status
`expected`: those facts are outside this explicit local scope, not verified
passes. It retains candidate, bundle SHA-256, manifest/version agreement,
lint/build configuration, effective runtime libraries, local-command, host and
independent-review requirements. `--github` still names the intended source
repository; no authenticated metadata is needed to accept local proof.

Public-review source preview still fails for a private repository and remains
pending for unavailable repository metadata. It permits a missing/draft release
before publication; an actual remote collection error remains visible as
pending. Before a first submission, use `public-release --initial`: a matching
public release is mandatory even though a pre-registration source-preview
dashboard is unavailable. For later updates, the public-release phase also
requires supplied `sourcePreview` evidence for the exact candidate. A private
local result is not official review evidence. An explicit non-public
`sourcePreview.previewIntent` or private `claimCeiling` is rejected as pending;
legacy supplied review records without these additive fields retain their
existing evidence contract.

The JSON schema remains `viggobsidian.obsidian-plugin-preflight.v1`; the new
`previewIntent` and `publicAvailabilityVerified` fields are additive.
`phasePassed` means every check is `pass` or explicitly `expected` within the
selected scope. `claimCeiling` bounds the possible claim even when the phase
fails; it is not itself a success flag. `publicAvailabilityVerified` is true
only for a passing `public-release` or `catalog-install` phase.
`deliveryComplete` is true only for a passing `catalog-install` phase.
Both flags are always false for source previews, including public ones.

Supplied `localValidation`, `hostValidation` and `independentReview` records
must each contain `status: "pass"` and the exact candidate `commit`.
`candidate.bundleSHA256` must equal the local `main.js` hash. Local checks name
actual `npmCi`, `officialLint`, `typecheck`, `tests`, `build`, `productionAudit`
and `typesResolved` results. Host evidence includes `pluginVersion`, matching
`payloadSHA256`, `appVersion`, `platform`, `settingsRestored`,
`sourcePreserved` and `unloadPassed`. These supplied records are not newly
executed or independently certified by the checker. Preserve their provenance
separately; public artifacts use only fictional fixtures.

Catalog acceptance additionally requires current-commit `releaseReview` and
`catalogInstall` records, the latter with `channel: "community"` and
`fresh: true`. Use `--installed-dir DIR` to read actual installed bytes.
Matching manual installation files cannot substitute for that channel proof.
No intent can bypass exact release hashes, public-source checks in publication
phases or actual catalog installation requirements. CLI exit status is zero
only when `phasePassed` is true; pending checks remain an unsuccessful phase.
