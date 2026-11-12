# Activity provenance footer — issue #47

New and explicitly converted Event bubbles keep the accessible author title and
show provenance once in a final separate italic source paragraph. The existing
shared footer CSS renders it smaller, muted, right aligned and nonitalic, outside
folded Details. No role color, icon, avatar or ordinary italic source is removed.
The #32 visible header CSS remains a bounded historical compatibility rule until
explicit source conversion; new authoring emits no `activity-meta` span.

The provenance-v1 checker accepts final direct footers as well as historical
top stamps and #32 headers. It rejects mixed/duplicate forms, invalid fields,
footer placement inside Details or fences, missing paragraph separation and
visible content after provenance. Mail version/delivery checks remain active.
The processing checker also accepts the new three-field footer while keeping
its older author-only/two-field forms. Missing-provenance adoption stays finite:
invalid or duplicate supplied footers cannot become historical formatting debt.

Authoring guidance, shared vault skill, mail template, Work intake and fictional
delivery fixtures use the same footer syntax. Work intake preserves unknown time
as `—`. The procedure fixture snapshot is regenerated from the actual skill.
The source-only `activity-footer-preview` Rust binary requires the current SHA256
and explicit original heading lines, prints a candidate, and never writes to a
vault. It preserves unselected bubbles and refuses ambiguous source. The writer
still owns its target, preimage gate, human content and recovery authority.

Implementation language: Rust for checker/tool/tests; existing CSS and Markdown
assets are maintained in their native formats. Existing SHA256 dependency is
reused; no dependency or Cargo.lock change. Baseline origin/main: `7712f54`;
local toolchain: rustc/cargo 1.92.0. No GitHub Actions or authentic source content.

Local checks: workspace locked tests, rustfmt, Clippy with warnings denied,
and complete diff review. Native Reading view/Live Preview, wide/narrow visual
acceptance, independent review and pinned installations are separate release
gates; this implementation note alone does not claim them complete.
