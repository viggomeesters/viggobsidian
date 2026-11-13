# Native source preparation

`source-preparation-core` owns the reusable `source-capture-v1` postimage for an
explicitly selected plain `type: note`, `category: source` capture. It applies the
numbered-v1 Raw/Event Note convention. It is not a general Markdown formatter,
mail/task adapter, source classifier or writer. Existing producers and legacy
recovery formats keep their own contracts.

The native `source-note-transform` executable reads bounded JSON on stdin and
returns one JSON response on stdout. It performs no filesystem, process, registry,
network, clock or vault writes. It does not discover or select a vault. The host
must independently establish the actual root, owner, source inventory, original
Git object, claims, authority and strict native checks. No plugin command, native
bridge, background service or new write authority follows from this component.

## Request and result

Select `schema: viggobsidian.source-preparation.v1`, `format: source-capture-v1`
and `operation: prepare` with these string fields:

- `source_note`: complete UTF-8 source including its original frontmatter.
- `date`: real preparation date (`YYYY-MM-DD`), supplied by the host.
- `source_path`: primary `00-inbox/YYYYMMDD-slug/YYYYMMDD-slug.md` locator.
- `destination_owner`: existing matching dossier foldernote, confirmed by the host.
- `vault_root`: explicit absolute URI context for Finder links, never root selection
  or authorization. It is not opened or probed by the core.

The postimage preserves original metadata text, adds only the typed reading-layer
fields and quotes the original body inside a folded source bubble. The Agent
stamp describes preparation, not the original human author or an invented source
time. Original heading anchors remain usable. Original complete source bytes
must also remain available through the host's existing pinned Git preimage.

`operation: mark-wiki-processed` takes the prepared `source_note`, its existing Raw
`source_path`, and additionally `original_note`, `preparation_date`, `wiki_target`
and `summary`. The preparation date comes from the sealed preparation intent.
The core reconstructs the exact prepared postimage before producing the narrow
processed metadata and attributed Wiki-processing bubble. The host must prove
that the target is a unique existing Wiki heading and the summary is true.

`viggobsidian.source-preparation-result.v1` reports `status: complete`, component
`engine_version`, format/operation, source/prepared locators, `before_sha256`,
`after_sha256`, `original_sha256` and `postimage`. Complete means only that this
typed text transformation succeeded, not that the note was saved or processed.
Failures return `status: incomplete`, a reason and no postimage; the CLI exits 2.
Unknown fields/schema/format, duplicate JSON or YAML keys, YAML merge keys, unsafe
paths, invalid calendar dates, source/owner mismatches and excessive input fail
closed. Limits: 8 MiB JSON request, 1 MiB per source/original note, 2 MiB postimage.
Merge refusal uses the parser's merge-key policy at every nesting level, including
flow mappings, whitespace and explicitly merge-tagged keys. Quoted `<<` keys,
explicitly string-tagged `<<` keys and quoted text containing `<<` remain ordinary
source data; they are not merges and remain byte-preserved in the header.

## Local adapter and recovery

Pin an exact reviewed executable hash, API and format in the existing local
provider binding. Bind the evaluated binding bytes and executable hash into every
new source-format intent, including ordinary owners. Check them before and after
the read-only call, before installation and before finalizing. Pin local policy
and the exact selected owner/Wiki/Raw separately. Never replace a mutable global
CLI, infer an owner from source prose or treat a clean response as write authority.

Keep the existing local canonical claim, complete-package verification, source
identity, CAS, staging, fsync, journal and recovery engine. The local adapter calls
this core for both new source-format operations; it must not retain a second copy
of their rendering logic. An old intent without a format must reconstruct its
exact old postimage using the old path. Do not upgrade, repin or beautify pending
or ready legacy intents. A format change requires a separately reviewed adapter
transition, not a new registry or source identity.

The public fixtures are fictional. Core/CLI tests freeze exact postimages, hashes,
body preservation and rejection behavior. Native vault-layout tests independently
apply the actual strict Raw/Event Note gates to both golden postimages and reject
malformed variants. Local provider tests remain responsible for real authority,
Git, claims, executable/config drift and interrupted recovery. This separation
does not weaken existing validation.

## Heading references after preparation

The preserved original source may contain headings inside nested blockquotes.
These are live Markdown heading targets; quoted fenced examples and HTML comments
are excluded. Check actual cross-note references through the shared native
filesystem adapter with `vault-layout --vault ROOT --indexes --paths NOTE ...`
(anchors are enabled by default). Missing and duplicate targets remain errors.
Do not edit original source bytes or flatten its quotes to satisfy a stale local
checker. Existing local checker adapters need their own reviewed compatibility
repair and regression tests; this text-core behavior grants no writer authority.

A mixed-owner processing fixture must use an ordinary writable owner for its
general topic. One reviewed system owner does not authorize another system
owner. Keep a negative transaction test proving that the second system write is
refused without changing source, task or owner bytes. Provider-specific fixtures
and bindings remain local; public tests use fictional source material only.
