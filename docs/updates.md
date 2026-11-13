# Installeren en bijwerken

Gebruik een gecontroleerde releasecheckout, niet een automatische pull van `main`. De manifesthash legt alle pakketbytes vast; `revision` wijst naar de exacte broncommit waarmee die assets zijn gebouwd. Verifieer de tag en manifestdiff voordat je een nieuwe release vertrouwt. Een hash is integriteitsbewijs, geen cryptografische uitgevershandtekening.

Bouw vóór `pack` de CLI uit dezelfde releasecheckout. Wanneer `Cargo.toml`
aanwezig is, leest de bestaande Cargo-tool de echte `package.version` met zijn
TOML-parser via `metadata --no-deps --offline --locked`. Deze versie en de
CLI-versie moeten exact overeenkomen. Er wordt geen broncode gebouwd of netwerk
gebruikt, en Cargo mag de lock niet wijzigen. Een oude CLI, onleesbaar manifest of onvolledige
bronrevision wordt geweigerd vóór `package.json` verandert. Gebruik voor de
revision de volledige veertigtekensbroncommit; behoud gepubliceerde tags en
herstel foutieve versiemetadata met een opvolgende patchrelease.

```sh
cargo build --release --locked
./target/release/viggobsidian plan --vault /path/to/vault --package . > /path/to/preview.json
./target/release/viggobsidian apply --vault /path/to/vault --plan /path/to/preview.json
./target/release/viggobsidian check --vault /path/to/vault
```

Preview schrijft niets naar de vault. Apply controleert de hele preview opnieuw, weigert lokale drift en maakt een receipt met oude bytes vóór de eerste wijziging. De manifestallowlist beperkt de updater tot gedeelde instructies, skills, de actieve CSS, Bases en de expliciete Rust-cratebestanden met regressiefixtures. Hij verandert geen notities, projecten, registraties, runtimeproviders, Gitremotes, pluginconfig of root-AGENTS. Verwijdering buiten de eindige beoordeelde migratieset blijft geweigerd.

De tien-kernenmigratie heeft een eindige, gecompileerde historische padinventaris.
Oude manifesttargets blijven leesbaar en herstelbaar, maar kunnen niet door een
nieuw pakket worden geschreven. Alleen ongewijzigde beheerde bestanden uit die
inventaris mogen verdwijnen, en alleen bij aanwezigheid van alle tien nieuwe
SKILL.md-kernen. Preview toont after: null; apply bewaart alle oude bytes vóór de
eerste wijziging in een lokaal receipt. Onbeheerde bestanden, providers en
profielsymlinks worden niet verwijderd. De lokale profielbeheerder gebruikt eigen
preimages. Een later aangemaakt bestand op een oud pad blokkeert rollback.

## Stijlen en diepe layoutcontrole vanaf v0.2

De lokale `.viggobsidian/appearance.json` bevat geordende bestemming-/avatarregels en een optionele `human_alias`. De compiler combineert deze gegevens met de gedeelde CSS tot het bestaande actieve `.obsidian/snippets/folder-note.css`. Het manifest bewaart de bronhash; de installatie bewaart daarnaast de configuratiehash en afgeleide outputhash. Preview/apply berekent dit geheel opnieuw en weigert een verouderde preview. Een lokale configuratiewijziging vraagt een nieuwe preview/apply; wijzig de gegenereerde CSS nooit rechtstreeks. Bewaar originele stylesheets en beoordeel de gerenderde cascade bij eerste adoptie.

De engine staat regulier geïnstalleerd onder `.agents/tools/vault-layout`; de bestaande launcher blijft werken. `--strict PAD` toetst een scope, `--check-all` alle genummerde collecties, `--check-work` daarnaast het Work-package-/linkstijlcontract. De checker bevat de strengere controles uit beide oorspronkelijke varianten en kan dus meer bestaande inhoudsproblemen melden. Het updatecommando migreert die inhoud niet.

Een opname van `css-split --input FILE` is uitsluitend lokaal voorwerk; inspecteer iedere gedeelde regel vóór publicatie. Automatisch splitsen op selectors is geen volledige privacybeoordeling. Geïnstalleerde fixtures zijn reguliere releasebestanden; de repository verwijst voor Base-fixtures naar haar eigen canonieke assets. Voer dezelfde update niet gelijktijdig op meerdere Syncthing-hosts uit: de updatelock is lokaal.

## Eerste adoptie

De release bezit tien kernen en beperkte geselecteerde naslag, met een expliciete
allowlist in skill-library.json. core/procedures.json v2 beschrijft effecten en
echte bronadapters. De verwijderde naamroutes en proceduregenerator komen niet
terug. `cargo run --bin procedure-assets -- --adoption-map TARGET` blijft uitsluitend
een read-only hashinventaris; zonder dat argument genereert de tool niets.

Maak vóór gebruik de lokale `.viggobsidian/providers.json` met exact gebonden
doelroot en alle capabilities. Een beschikbare capability wijst naar een
bestaande lokale instructiesectie en haar benodigde providerbestanden. Externe
providerbestanden zijn alleen expliciete absolute locators; zij selecteren nooit
een andere doelvault. Een ontbrekende capability wordt `unavailable` met reden.
`vault-layout --vault ROOT --skill-routes` controleert de gedeelde release-assets
en volledige lokale binding; de controle voert geen provider uit. Zie docs/skills.md.

Verander lokale writer-/registry-/importercode niet als bijeffect van deze
bronoverdracht. Vergelijk hun oorspronkelijke hashes en draai bestaande
gedragstests. Gedeelde tekstwijzigingen kunnen reviewcaches/plannen terecht laten
weigeren; herlees en herplan, vernieuw geen oude reviewguards of bewijswaarden.

Maak lokaal `.viggobsidian/integration.json` met `schema: viggobsidian.integration.v1`, `profile: numbered-v1`, `adoption: {legacy_project_roots: []}` en `local_adapter: .viggobsidian/local-adapter.md`. Schrijf die adapter voor de werkelijke doelvault en providers. Deze twee bestanden zijn lokale configuratie en staan buiten releasebeheer.

Bestaande skill- of Base-bestanden worden nooit automatisch als eigendom geclaimd. Een adoptiebestand is een JSON-object met per doelpad de beoordeelde SHA256 van de bestaande bytes. Geef dat expliciet mee via `plan --adopt /path/to/adoption.json`. Bekijk de delta vóór apply en bewaar een lokale Git-snapshot. Latere updates gebruiken de vorige geïnstalleerde hashes en vereisen geen hernieuwde adoptie.

De CLI schrijft JSON naar stdout. Redirect naar een nieuw lokaal planbestand buiten het pakketoppervlak. De tool controleert symlinkouders en padtraversal. Hij gaat uit van een vertrouwde lokale gebruiker en lokale schijf; hij is geen sandbox tegen een kwaadwillend proces dat tegelijkertijd bestandsnamen vervangt. Sluit andere beheertools tijdens een update. Een update-lock coördineert ViggObsidian-processen, geen inhoudelijke writers.

## Herstel

Vanaf v0.5 behouden `human` en de geconfigureerde `human_alias` allebei de menselijke
uitlijning en lokale avatar. De alias vervangt de canonical rol niet. Namen van
assistent-/systeemrollen zijn geen geldige menselijke alias. Een update wijzigt
geen auteursmetadata of historische notes om een stijlprobleem te repareren.

```sh
./target/release/viggobsidian rollback --vault /path/to/vault --receipt /path/to/vault/.viggobsidian/receipts/ID.json
```

Rollback vereist de actuele geïnstalleerde postimage en weigert lokale wijzigingen. Bij een onderbroken apply staat `.viggobsidian/pending.json`; geef dat bestand als receipt om de gedeeltelijke transactie terug te draaien. Een onderbroken rollback bewaart `.viggobsidian/rollback.json`; geef dat bestand als receipt om het herstel te hervatten. Verwijder zo’n marker of lock niet blind. Receipts bevatten oude instructiebytes en kunnen lokale informatie bevatten: bewaar ze privé en neem ze niet op in een publiek Issue. Ze zijn lokale herstelbestanden, geen onbetrouwbare downloadinput.

Na apply: draai ook de volledige bestaande lokale strikte layoutcheck en de skillbroncontrole. Een fout daarin is geen toestemming om een hash te vernieuwen of een validator te versoepelen. Herstel de gerichte delta of draai terug. Geïnstalleerde bestanden worden niet handmatig onderhouden: wijzig de upstreambron, lever een nieuwe release en herhaal preview/apply per vault.
