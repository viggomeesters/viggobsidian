# ViggVault en ViggObsidian

Besluit van 13 november 2026, uitgewerkt in [ViggVault-plan #3](https://github.com/viggomeesters/obsidian-vigg-vault/issues/3) en [engine-uitvoering #78](https://github.com/viggomeesters/viggobsidian/issues/78).

ViggVault is het gebruikersproduct en de Obsidian-interface, met Community-ID `vigg-vault`. [Het productcontract](https://github.com/viggomeesters/obsidian-vigg-vault/blob/main/docs/product-engine-boundary.md) bezit de beschikbare commando’s, snapshotcapture, Worker/WASM en presentatie. ViggObsidian blijft de canonieke Rust-engine, pakketbron en native CLI. Beide blijven bestaan; ViggObsidian wordt niet deprecated. Bestaande CLI-, pakket-, provider-, writer- en installed-ID-contracten worden niet hernoemd of verwijderd.

Regels worden eenmaal upstream in Rust geïmplementeerd. De native filesystemadapter houdt selectie/classificatie, lokale configuratie, bronbestaan, pad-/symlinkgrenzen en schrijfrechten. Lokale writers blijven eigenaar van registraties, claims en domeintransacties. Een pure tekstcore kan die feiten niet bewijzen en verleent geen schrijfrecht.

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
