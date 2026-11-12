# Event bubble headers — v0.14.0

Issue #32 adds a canonical provenance header to the existing provenance-v1
contract. The callout title names the author once; an exact activity-meta span
holds time and channel. CSS renders it smaller, muted and wrapping at narrow
widths. Existing stamp-only notes retain their valid representation and style.
New mail templates, vault instructions and Work task intake use the header.

The checker accepts one representation, rejects invalid clocks, empty channels,
extra markup and a contradictory immediate legacy stamp. A normal inline-code
first source line remains content. Malformed supplied header provenance cannot
be classified as historical formatting debt. Original input, goal binding and
mail-version checks remain independent and enforced.

Local verification: 65 engine tests, 30 integration tests and 25 updater tests;
fmt and Clippy with warnings denied. No new dependency. Cargo audit used the
available local advisory database (1471 advisories, 39 locked dependencies).
The Rust policy scan uses the actual integration base; JavaScript is confined
to the mandatory native Obsidian test API, not a new product runtime.

Native Obsidian checks used the existing test vault and detached, hidden render
surfaces. Reading view rendered all seven fictional headers at widths 1100 and
390. Live Preview checked human/unknown-time/long-name cases and each complete
mail version v1/v2/v3 at both widths, with three copy controls per mail. Metadata
was visible, smaller than the author, outside code elements and without header
overflow. An italic closing sentence remained visible. The active leaf stayed
unchanged. The initial virtualized full-note probe did not materialize later
mail bubbles; separate short version fixtures supplied that missing evidence.

No generic italic-content hiding rule was retained. No bulk note migration is
part of the package. Explicit source conversion must preserve the original bytes
in local recovery evidence and change only adjacent canonical heading/stamp
pairs, retaining unknown time, author, channel and every content byte.
