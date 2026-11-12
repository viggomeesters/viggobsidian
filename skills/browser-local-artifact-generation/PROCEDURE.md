---
name: browser-local-artifact-generation
description: Build no-upload browser-local generators and portable offline artifacts with explicit preservation, determinism, provenance and downstream verification.
license: MIT
---

# Browser-local artifact generation

Use for local file processing or portable HTML, workbook, XML and ZIP generators that do not require hosted infrastructure. This is an implementation method, not a default requirement to turn every answer into HTML.

Resolve the actual target repository/workspace and its authorization, language and dependency rules. Read optional `.agents/skill-adapters/browser-local-artifact-generation.md` only from that explicitly selected target. It may bind an existing canonical renderer, serializer or domain validator; missing capability is not permission to fork it, reactivate a legacy writer or choose a private vault. Continue independent safe work and identify the affected proof gap.

If an authorized existing snapshot engine owns read-only projections, use its contract rather than building a competing renderer. Editable file generators and artifacts outside that engine's typed contract may still use this method. A provider locator is local configuration, not a universal source path.

## Privacy and effect boundaries

Source input stays in local/browser memory unless upload/storage is explicitly authorized. Network reference lookups must not include input bytes, sensitive filenames or customer rows. Treat a generated decision as draft/review-required unless an actual human decision establishes otherwise.

A local generator does not authorize hosting, external delivery or a production runtime. Existing session authorization persists within its real scope; do not reopen old choice gates after compaction, but resolve genuinely new consequential decisions. Static validation is not downstream execution proof.

## Establish the artifact contract

Before implementation identify accepted extensions/count/size, expected identity and ordering, filename/collision rules, exact output structure, preservation guarantees, malformed/ambiguous rejection, privacy behavior and downstream path/provenance requirements. Build focused behavior tests for the material requirements; avoid tests that merely mirror the implementation.

Keep parsing, identity resolution, generation, packaging and UI contracts separable so a failure identifies its layer. New code and dependencies follow the actual repository contract; a short script is not an exception to that policy.

## Patch or rebuild deliberately

Patch an authoritative workbook when existing user state must survive. Resolve real sheets/headers and stable identity; mutate only the authorized missing data or owned generated sheets. Preserve formulas, styles, hidden/custom sheets and human decisions. Reject duplicate/incomplete identities and append an update summary instead of rewriting history.

Build a new workbook when the requested output is a derived artifact. Make row/column order explicit, stabilize metadata only where byte determinism matters and preserve leading zeros/text types. Include useful human navigation and machine identity when required. Independently read serialized output back. Do not rebuild user-owned state because that is easier.

Do not claim `.xlsm` macro preservation through a library that does not preserve VBA. Reject that path, use a supported preserving method, or obtain explicit agreement for an honest non-preserving output.

## Identity and paths

Normalize at the canonical boundary only; retain template-native values when the downstream contract requires them. For a single-object contract, require exactly one object. Reject unmatched, mixed, ambiguous or conflicting identities and duplicate sequences; keep first-occurrence order unless an explicit sequence authority exists. An alias is valid only when its authoritative mapping identifies one unique target.

Treat filenames, sheet names, object names and parsed IDs as untrusted. Remove directory segments and use portable safe basenames. Reject traversal, absolute-drive, UNC and POSIX paths escaping a package. Escape or validate data entering formulas, XML, queries and downstream expressions. Preserve legitimate domain namespaces deliberately rather than broadening a regex without a proven contract.

## Deterministic packaging and runtime handoff

For deterministic ZIP output, sort all directory/file entries, create explicit directories with fixed timestamps, use fixed compression/platform metadata and avoid current-time manifest fields unless time is part of the contract. Resolve safe collision-free names before writing. Generate twice and compare raw bytes, then independently inspect entries/content.

Keep executable artifacts and relative runtime references within the package root. Include required-input placeholders only when useful, and a manifest with required files, outputs, source identity, generator version and external proof still missing. Retain license and file-bound provenance for bundled upstream assets: repository, commit/revision, source path, hash and bundled location.

A sync/check facility should compare the actual canonical bytes and provenance; write mode is an explicit refresh. A generated runtime in a package is not proof that a downstream application loaded or executed it.

## Static-first standalone HTML

When HTML is the selected product, the local canonical source remains authoritative and the HTML is a derived, versioned projection. Do not make it an undocumented import format or a second source of truth.

- Inline necessary CSS/data/runtime assets; require zero essential runtime fetches, remote fonts/styles/scripts or images for an offline claim.
- Embed JSON as non-executing data and escape `<`, `>`, `&`, U+2028 and U+2029 so values cannot terminate their script container. Render untrusted content as text rather than markup.
- Pre-render the essential report/catalog content and safe links as ordinary HTML. Script-blocked clients may still consider scripting enabled, so `<noscript>` alone is insufficient.
- Permit only validated outbound HTTP(S) links and isolate new browsing contexts. Read-only artifacts contain no create/edit/archive controls.
- State meaningful source/export/version dates without confusing the user's local calendar date with UTC or the latest observed event date. For timeline windows, treat missing dates as unknown rather than silently future/outside; do not truncate to the last recorded event unless explicitly requested.
- The visible static state and enhanced initial state must agree on selected window, denominators, unknown counts, warnings, data values and visible rows. “Some content exists” is not semantic parity.
- Keep one semantic state per visual mark. Check representative boundary records through source → class/label → computed style and legend; one color cannot mean incompatible states.
- For domain-indexed projections, pin the exact represented source hashes before and after the read. Do not falsely claim global freshness when only a domain slice is verified, and do not let unrelated global churn manufacture a pass or starve a legitimate bounded export.
- Treat possession of the output as read access. Use restrictive local file permissions and atomic publication appropriate to the platform; obtain an actual encryption/delivery decision when sensitivity requires it.

A recipient's proven capabilities determine the correction. If controls do not work, produce the necessary static selected view with essential detail visible; remove dead controls and script behavior rather than leaving inert affordances. Do not make a recipient-specific failure into a universal ban on useful links or native details. Domain requirements and actual source data still govern. Never disable the recipient's security software to make an artifact render.

Use a new versioned filename after a corrected attachment; clients can cache old bytes under the previous name. Retain the actual recipient report and the corrected artifact's provenance, not a claim of universal compatibility.

## Verification in layers

1. Run focused parser/generator behavior checks and malformed/conflict/security cases.
2. Independently read the serialized workbook/ZIP/XML/HTML. Validate actual cells/types, entries, identities and paths rather than construction objects.
3. Run applicable repository gates and domain validators.
4. Obtain real downstream runtime/result evidence when acceptance requires it; otherwise state the exact handoff and unverified boundary.
5. Open the final saved HTML offline at desktop and narrow mobile sizes. Exercise actual selected functions and largest selectable dataset/window, compare counts with embedded data, inspect console/network dependencies and check pixels.
6. Repeat with scripts genuinely blocked. Check exact semantic content, representative links/records, row counts, summary values and overflow. Keep QA probes out of the delivered screenshot.
7. Inspect the final input constraints, ambiguity/error states, preservation guarantees, remaining runtime requirements and clear download action. Long paths must wrap; mobile controls need actual room to shrink.

Common readback traps include hyperlink objects versus visible text, sparse blank rows, normalized autofilter ranges, shared mutable styles and an untested route that displays a placeholder. Verify the actual output behavior and avoid exact-shape assertions when the serializer legitimately normalizes a representation.

## Hosting retirement is separate authorization

If the user explicitly requests replacing a hosted dashboard with private snapshots, first preserve canonical data and prove the replacement. Keep data ownership/pipelines separate from presentation. Replace deploy automation in its configuration source so restoration cannot resurrect hosting. Preserve IDs, aliases and rollback evidence; commit/push or external mutations remain subject to their real authorization. Verify the new generation/delivery path before deleting hosting, then verify actual removed resources/aliases and former URLs. Do not infer a hosting teardown from an offline export request.

## Completion evidence

Report the input/privacy boundary, preservation or rebuild strategy, actual determinism/security/readback results, source/provenance drift checks, downstream runtime proof or precise limitation, tested desktop/mobile/static states and final artifact/package paths. Do not equate a screenshot, dry-run, copied expected data or package inventory with execution/result correctness.
