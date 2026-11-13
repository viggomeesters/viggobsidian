# ViggVault en ViggObsidian

Besluit van 13 november 2026, uitgewerkt in [ViggVault-plan #3](https://github.com/viggomeesters/obsidian-vigg-vault/issues/3) en [engine-uitvoering #78](https://github.com/viggomeesters/viggobsidian/issues/78).

ViggVault is het gebruikersproduct en de Obsidian-interface, met Community-ID `vigg-vault`. [Het productcontract](https://github.com/viggomeesters/obsidian-vigg-vault/blob/main/docs/product-engine-boundary.md) bezit de beschikbare commando’s, snapshotcapture, Worker/WASM en presentatie. ViggObsidian blijft de canonieke Rust-engine, pakketbron en native CLI. Beide blijven bestaan; ViggObsidian wordt niet deprecated. Bestaande CLI-, pakket-, provider-, writer- en installed-ID-contracten worden niet hernoemd of verwijderd.

Regels worden eenmaal upstream in Rust geïmplementeerd. De native filesystemadapter houdt selectie/classificatie, lokale configuratie, bronbestaan, pad-/symlinkgrenzen en schrijfrechten. Lokale writers blijven eigenaar van registraties, claims en domeintransacties. Een pure tekstcore kan die feiten niet bewijzen en verleent geen schrijfrecht.

De [getypeerde bronvoorbereiding](source-preparation.md) levert uitsluitend een
deterministische Raw-bronpostimage via een read-only native CLI. Lokale adapters
blijven eigenaar van bestemming, claims, bronidentiteit, installatie en herstel.
De plugin krijgt hierdoor geen schrijf- of bronplaatsingsfunctie.

Het pakketprofiel `numbered-v1` is expliciet gekozen beleid. De Wiki-opbouwregels horen bij die conventie; willekeurige Markdown of een bestandsnaam maakt een note geen Wiki. Heading-/ankerhelpers zijn tekstmechaniek, geen automatische profielkeuze. De bestaande Work-linkstijl blijft een afzonderlijk gekozen regelset/contract.

Product- en engineversies blijven onafhankelijk. Een pluginrelease pint de exacte gereviewde enginebron, schema/API, WASM-hash en licentie; zij haalt geen nieuwe engine of executable zelfstandig op. ViggVault 0.1.1 biedt alleen opt-in Work-linkstijl. Deze upstream-extractie voegt nog geen Wiki-commando aan de plugin toe en is geen volledige vaultcheck. De native CLI blijft onafhankelijk bruikbaar.

## Wiki-tekstgrens v1

`wiki-layout-core` bevat de bestaande `wiki_layout::findings` en heading-/ankerhelpers. De originele regelnamen, boodschappen, volgorde en `has_facts` blijven behouden. De bestaande native CLI blijft haar documentselectie en metadata/body-splitsing uitvoeren; haar filesystem-/writergedrag verandert niet.

De aparte JSON-input heeft `schema: viggobsidian.wiki-layout-snapshot.v1`, `ruleset: wiki-layout.v1`, `complete: true`, `profile: numbered-v1`, `classification: wiki`, `path` (zichtbaar relatief Markdownpad) en `body` (tekst na metadata). De adapter bevestigt classificatie en volledigheid; het core controleert geen filesystem en leidt Wiki-identiteit niet uit het pad af. De capability is uitsluitend Wiki-tekst: preamble, optionele facts, TOC/H2/ankers en technische broninventaristabellen.

Het antwoord gebruikt `viggobsidian.wiki-layout-result.v1`. `complete` bevat `report: {path, has_facts, findings: [{path, code, message}]}`; een complete uitvoering kan bevindingen hebben. `not_applicable` zonder rapport betekent een niet-gekozen profiel of gewone note. Ontbrekende vereiste input, onbekend schema/regelset of limietoverschrijding geeft `incomplete` zonder rapport. Geen uitkomst bewijst volledige vaultgeldigheid. De regels leveren geen locaties of ernstlabels: adapters verzinnen die niet. Note-navigatie kan op het meegegeven pad; regelnavigatie vraagt een later betrouwbaar locatiecontract.

Limieten: 1 MiB body en 8 MiB JSON-request. De WASM-build wordt op 256 MiB lineair geheugen begrensd; hosts moeten traps, time-outs en annulering opvangen. De importvrije ABI gebruikt dezelfde single-worker request-/responsebuffers als de bestaande linkstijlgrens. Een volgende pluginintegratie moet capture/drift, Workercleanup en echte Obsidian-gebruiksproeven afzonderlijk bewijzen. Bronbestaan, shortcutdoelen, dossierstructuur en `--check-all` vallen buiten deze slice.

## Gedragsbewijs en taalkeuze

De fictieve corpusresultaten zijn vóór de extractie vastgelegd tegen bron `42ce85ebf7d1494cbe186190294ec24a2eed753c` (commit `fd68f83`). Ze dekken correcte opbouw, facts afwezig/verplaatst/beschadigd/dubbel, de vier-H2-grens, Unicode/gecodeerde/dubbele ankers, CRLF, comments, fences en broninventaris. Tests vergelijken exacte codes, boodschappen en facts; native filesystemselectie en echte WASM-uitvoering worden apart vergeleken. De filesystemvergelijking selecteert alleen de zes gedeelde tekstcodes; eigen adapterbevindingen zoals `wiki-direct` en `broken-shortcut-note` blijven bestaan. Bestaande parsergrenzen blijven dus ook behouden; dit is geen nieuwe Markdownstandaard.

Nieuwe engine-, CLI- en testlogica is Rust. Er zijn geen nieuwe registrydependencies; `serde_json` en de bestaande lockkeuzes worden hergebruikt. `experiments/wiki-layout-parity.test.mjs` is de noodzakelijke JavaScript WebAssembly-platforminterface voor het echte import-/buffer-/runtimebewijs. Het implementeert geen regels; een nieuwe Rust-WASM-runtime toevoegen zou onnodig dependencyoppervlak veroorzaken. De productieplugin blijft ongewijzigd.

Lokaal reproduceren (zet `CARGO_TARGET_DIR` en `TMPDIR` op een eigen private scratchmap):

```sh
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --locked -p wiki-layout-core --bin wiki-snapshot-check
RUSTFLAGS='-C link-arg=--max-memory=268435456' cargo build --locked -p wiki-layout-core --lib --release --target wasm32-unknown-unknown
WIKI_LAYOUT_WASM="$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/wiki_layout_core.wasm" WIKI_LAYOUT_NATIVE="$CARGO_TARGET_DIR/debug/wiki-snapshot-check" node --test experiments/wiki-layout-parity.test.mjs
```

Geen release, pakketrollout, notewrites of nieuwe pluginfunctie volgt alleen uit deze extractie. Integratie en distributie vereisen hun eigen beoordeelde bronrevision en controles.

Wiki capture protocol v2 adds portable canonical target selection: `vault-wiki-direct` in a string-array cssclasses, or a `wiki.md` note beneath any relative `wiki` folder component. The native adapter calls the same pure Rust helper. Protocol v1 remains unchanged. The host supplies body text after frontmatter and parsed cssclasses; v2 validates required fields and normalizes body line endings like the native adapter. Obsidian YAML parsing is a platform boundary, not a claim of native YAML or filesystem eligibility parity. No link target or folder fact is checked by this text slice.

Portable Wiki v2 requires typed `frontmatter_present`; selected Wiki targets without frontmatter return Incomplete, matching native Scan::note eligibility. Ordinary non-target notes remain not applicable. Missing or malformed presence input cannot pass as clean.

## Activity text v1

The separate [structure capability experiment](structure-capabilities.md)
documents the next bounded slice: existing matching-entrance diagnostics shared
with explicit visible-inventory input. It does not transfer native regular-file,
symlink, profile discovery or package guarantees to Obsidian, and adds no plugin
command. Engine #92 and ViggVault #7 own its proof and capability design.

Engine [#86](https://github.com/viggomeesters/viggobsidian/issues/86) owns `activity-core`; product [#6](https://github.com/viggomeesters/obsidian-vigg-vault/issues/6) owns its Obsidian interface. The existing Activity layout, provenance/result-goal, delivery chronology and mail-draft text checks now live once in this Rust crate. Native modules reexport them; native strict/metadata selection, messages, writer paths and historical exceptions remain unchanged. `link-style-core` supplies the existing visible-body/quote helpers without changing its bytes.

The portable input is `schema: viggobsidian.activity-snapshot.v1`, `ruleset: activity-text.v1`, `complete: true`, `profile: numbered-v1`, visible relative Markdown `path`, typed `frontmatter_present`, mapping `metadata` containing only `cssclasses`, `activity_format`, `activity_contract`, and `body` after metadata. Missing capture fields, invalid metadata shapes and absent frontmatter under the selected profile return Incomplete. Host YAML parsing is an explicit platform boundary; this mapping protocol does not claim byte-level native YAML-parser parity. Line endings normalize like the native frontmatter adapter. The fixed leading knowledge-owner notice contains no Activity tokens; its presence cannot change these text results, covered by fixtures.

Existing selection is retained: `activity_format: chat-v1` checks legacy chat; any present `activity_contract` checks provenance-v1 or reports the existing unknown-contract finding; `vault-event-v1-1` selects the existing Activity-tail validator. Marked mail drafts, deliveries and human-input markers use existing visible/fenced selection. Marker-free historical notes are not upgraded to a provenance contract and are visibly not applicable. The plugin profile is opt-in. This text slice includes mail checks that native strict mode already applies independently of the provenance opt-in; it is not the complete native strict check.

Results use `viggobsidian.activity-result.v1`: `complete` has capabilities `activity-text.v1`, `provenance-v1`, `chat-v1`, `event-activity-v1.1`, `mail-delivery-v1` and `{path,findings:[{path,code,message}]}`. Ordinary historical notes and unselected profiles return `not_applicable`; unsupported schema/ruleset, missing capture, resource or host failures return `incomplete`, without a clean report. Existing codes/messages are preserved. `event-activity-layout` identifies only the extracted Event v1.1 Activity-tail check; Event title/cards, dossier breadcrumbs, shortcut targets, Raw task classification, sources and filesystem facts remain native-only. No identity or completed-action claim follows from a formally valid author/provenance marker. No new reliable line locations exist, so consumers offer note navigation only.

Limits are 1 MiB body, 8 MiB request, 100 returned findings and 256 MiB WASM memory. A host must terminate its Worker on deadline/cancellation; the legacy provenance scan can be quadratic for many short messages. A finding limit is Incomplete, never truncated success. The Activity ABI uses `activity_request_buffer`, `activity_validate`, `activity_response_buffer`; this permits linking the unchanged link-style helper crate, whose separate legacy ABI symbols may also be present. Consumers call only the Activity ABI.

Language/dependency gate: new rule, CLI, fixtures-generation and native tests are Rust. No new registry dependency or version was introduced; existing serde_json and link-style-core/its locked dependencies are reused. The only new JavaScript is `experiments/activity-parity.test.mjs`, required to exercise the actual WebAssembly runtime and buffers. Frozen fictional corpus outputs come from the pre-extraction source `10d288f8bc7ec055d6e14fa8988507327b3a717d`, whose three original modules are byte-identical at the current integration base `93f6c62cbc8d4650736532fbfd3f9823f0481939`. The native filesystem test covers strict delivery/mail, opt-in provenance, chat, unknown contract and preserved legacy selection while checking source bytes remain unchanged. No runtime/datawriter or vault package rollout is authorized by this extraction.

Reproduce with a private scratch `CARGO_TARGET_DIR` outside this checkout (installed-crate tests require no enclosing source workspace), then:

```sh
cargo test --workspace --locked -- --test-threads=1
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --locked -p activity-core --bin activity-snapshot-check
RUSTFLAGS='-C link-arg=--max-memory=268435456' cargo build --locked -p activity-core --lib --release --target wasm32-unknown-unknown
ACTIVITY_WASM="$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/activity_core.wasm" ACTIVITY_NATIVE="$CARGO_TARGET_DIR/debug/activity-snapshot-check" node --test experiments/activity-parity.test.mjs
```
