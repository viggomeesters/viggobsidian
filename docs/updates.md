# Installeren en bijwerken

Gebruik een gecontroleerde releasecheckout, niet een automatische pull van `main`. De manifesthash legt alle pakketbytes vast; `revision` wijst naar de exacte broncommit waarmee die assets zijn gebouwd. Verifieer de tag en manifestdiff voordat je een nieuwe release vertrouwt. Een hash is integriteitsbewijs, geen cryptografische uitgevershandtekening.

```sh
cargo build --release --locked
./target/release/viggobsidian plan --vault /path/to/vault --package . > /path/to/preview.json
./target/release/viggobsidian apply --vault /path/to/vault --plan /path/to/preview.json
./target/release/viggobsidian check --vault /path/to/vault
```

Preview schrijft niets naar de vault. Apply controleert de hele preview opnieuw, weigert lokale drift en maakt een receipt met oude bytes vóór de eerste wijziging. De manifestallowlist beperkt de updater tot gedeelde instructies, skills, de actieve CSS, Bases en de expliciete Rust-cratebestanden met regressiefixtures. Hij verandert geen notities, projecten, registraties, runtimeproviders, Gitremotes, pluginconfig of root-AGENTS. Algemene verwijdering van eerder beheerde assets blijft geweigerd.

Vanaf v0.19 is één eindige migratie ingebouwd: de voormalige `go`-ingang en zeven `go-*`-aliases worden vervangen door `github-work`. Alleen de negen exact bekende, beheerde en ongewijzigde bestanden mogen verdwijnen, en alleen als beide nieuwe `github-work`-bestanden in het gevalideerde pakket staan. Preview toont de verwijderingen als `after: null`; apply bewaart de oude bytes in het herstelreceipt. Rollback herstelt ook de oude installatiestatus en weigert later aangemaakte bestanden op de oude paden. Onbeheerde bestanden en profielsymlinks worden niet door de pakketupdater verwijderd. De lokale profielbeheerder werkt de publieke naam en metadata afzonderlijk met eigen preimages bij.

## Stijlen en diepe layoutcontrole vanaf v0.2

De lokale `.viggobsidian/appearance.json` bevat geordende bestemming-/avatarregels en een optionele `human_alias`. De compiler combineert deze gegevens met de gedeelde CSS tot het bestaande actieve `.obsidian/snippets/folder-note.css`. Het manifest bewaart de bronhash; de installatie bewaart daarnaast de configuratiehash en afgeleide outputhash. Preview/apply berekent dit geheel opnieuw en weigert een verouderde preview. Een lokale configuratiewijziging vraagt een nieuwe preview/apply; wijzig de gegenereerde CSS nooit rechtstreeks. Bewaar originele stylesheets en beoordeel de gerenderde cascade bij eerste adoptie.

De engine staat regulier geïnstalleerd onder `.agents/tools/vault-layout`; de bestaande launcher blijft werken. `--strict PAD` toetst een scope, `--check-all` alle genummerde collecties, `--check-work` daarnaast het Work-package-/linkstijlcontract. De checker bevat de strengere controles uit beide oorspronkelijke varianten en kan dus meer bestaande inhoudsproblemen melden. Het updatecommando migreert die inhoud niet.

Een opname van `css-split --input FILE` is uitsluitend lokaal voorwerk; inspecteer iedere gedeelde regel vóór publicatie. Automatisch splitsen op selectors is geen volledige privacybeoordeling. Geïnstalleerde fixtures zijn reguliere releasebestanden; de repository verwijst voor Base-fixtures naar haar eigen canonieke assets. Voer dezelfde update niet gelijktijdig op meerdere Syncthing-hosts uit: de updatelock is lokaal.

## Eerste adoptie

Vanaf v0.10 bezit de release ook de generieke PROCEDURE.md-callerlocaties,
actieve referentielocators, native-vaultbridge en de acht gedeelde
procedurecontracten. `core/procedures.json` is de expliciete dekkingsmatrix;
de updaterallowlist is daarvan gecompileerd, niet door een aangeboden pakket
uit te breiden. Bestaande onbeheerde procedurebytes vragen beoordeelde
adoptiehashes en blijven in de herstelreceipt behouden. De authoringhelper
`cargo run --bin procedure-assets -- --adoption-map TARGET` maakt uitsluitend
een read-only hashinventaris voor inspectie; zij geeft geen adoptietoestemming.

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
