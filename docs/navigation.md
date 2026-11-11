---
type: note
category: knowledge
obsidianUIMode: preview
cssclasses: [vault-navigation, vault-dashboard, vault-wiki]
---

# Vault Navigation

> [!navigation|shortcuts]
> - [[99-system/vault/vault-navigation/vault-navigation|Back]]

```wiki-toc
```

Dit is de gedeelde navigatiestandaard van ViggObsidian. Wijzig de bron via
Issue → PR → review → release. `.viggobsidian/installed.json` bezit de lokale
versiepin. Providers, exacte uitzonderingen en uitrolstand blijven bij
`.viggobsidian/local-adapter.md` en `.viggobsidian/navigation-local.md`.
Oudere ingangen behouden kopankers en verwijzen naar deze standaard.

## Hiërarchie

Eén profiel: `numbered-v1`. Losse projecten staan onder `20-projects`;
programma's onder `21-naam` tot en met `29-naam`. Een project heeft één fysieke
eigenaar. Een topic is een blijvend onderwerp zonder begrensd projectresultaat.
Een onderzoek of traject met een resultaat hoort bij een project.

```text
21-programma/
  21-programma.md
  YYYY-MM-projectnaam/
    YYYY-MM-projectnaam.md
    abc-00-project-wiki/
      abc-00-project-wiki.md
      wiki/wiki.md
      raw/raw.md
    abc-01-onderdeel/
      abc-01-onderdeel.md
      wiki/wiki.md
      raw/
        raw.md
        YYYYMMDD-bron/YYYYMMDD-bron.md
```

De projectmaand is de startmaand. Onderdeelmappen gebruiken de unieke projectcode
in kleine letters, een tweecijferig nummer en een inhoudelijke naam. Zichtbare
codes mogen hoofdletters gebruiken. Onderdeel `00` bevat projectbrede besluiten,
deelnemers en rollen. Andere onderdelen volgen de inhoudelijke volgorde. Behoud
bestaande project-, onderdeel-, taak- en bron-ID's bij hernoemen of verplaatsen.
Een werkelijk nieuw project/onderdeel krijgt eenmaal een UUIDv4 via de bevoegde
intake; een gewijzigde naam krijgt geen nieuw ID. Controleer de unieke code met
de lokale `--resolve-code`-route; los botsingen op zonder bestaande IDs te vervangen.

### Hoofdmappen en notetypes

`00-inbox` ontvangt bronpakketten. `10-daily` bevat dagnotities en expliciete
writercollecties. Projectroots bevatten projecten en onderdelen. `30-topics`
leidt via gevulde navigatielagen naar dossiers. `90-templates` is een bibliotheek.
`99-system` ontsluit lokale adapters en beheer. Extra bestaande roots zijn lokale
adoptie/compatibiliteit, geen tweede profiel. Nieuwe centrale `01-tasks`,
`02-meetings`, `03-notes`, `04-sources` of `10-projectinfo` zijn geen standaard:
bronnen en taken horen in onderdeel-Raw. Bestaande providers met zulke paden
behouden uitsluitend hun begrensde contract.

### Vault als systeemcollectie

Een systeemcollectie bevat haar ingang en directe inhoudelijke kindmappen.
Technische vaste paden en writeropslag blijven op de exact gedelegeerde plaats.
De ouder houdt hun navigatie zichtbaar. Een uitzondering autoriseert geen
algemene bronnenmap als alternatief voor dossier-Raw.

## Principes en regels

Lees eerst de actuele eigenaar en de oorspronkelijke bron. Onderscheid feit,
voorstel, besluit en uitgevoerde actie. Wiki onderhoudt huidige kennis; Raw
bewaart oorspronkelijke input, bijlagen, herkomst en verwerking. Eén claim heeft
één actuele eigenaar. Verwijs elders; dupliceer geen veranderlijke kennis.
Maak alleen gevulde navigatielagen en bruikbare notes. Bronopslag autoriseert
geen uitvoering. Een groene structuurcheck bewijst geen bronwaarheid.

## Design per pagina

| Pagina | Opzet |
| --- | --- |
| Collectie/project | Titel, snelkoppelingen, één tegel per directe inhoudelijke kindmap |
| Onderdeel/dossier | Wiki en Raw, taakstatuskaarten en Open tasks |
| Wiki | Back, Raw, Finder, actuele kennis met precieze bronlinks |
| Raw | Back, Wiki, Finder, brontellers en Latest sources |
| Event Note | Titel, Back/Finder, dossierkaart, detailblok, Context, Activity |

### Foldernotes en navigatie

Iedere inhoudelijke tegel opent de gelijknamige foldernote van precies één
directe fysieke kindmap; iedere directe inhoudelijke kindmap heeft één tegel.
Back en Finder staan in `[!navigation|shortcuts]`, buiten de kindtegels in
`[!navigation]`. Gebruik interne wikilinks zonder `.md`; Finder wijst naar de
werkelijke lokale map. Routekleuren en iconen komen uit de lokale appearance-
configuratie en gedeelde CSS. Tegels op één pagina hebben herkenbare accenten.
Collecties en Raw hebben `layer: navigation`; dossiers en Wiki `layer: wiki`.
Een bron gebruikt haar inhoudelijke type/category. Chronologische kindcollecties
staan oud naar nieuw met alleen bewezen startperiodes. Een bestaand README met
eigen caller blijft de enige ingang tot een gecontroleerde omzetting.

### Wiki en Raw

Op een Wiki/Raw-dossier zijn `wiki/` en `raw/` de directe inhoudelijke lagen.
Technische writerdelegaties staan exact in de lokale scope. Zet geen bron-, log-
of analysemappen daarnaast als alternatief. Een zelfstandige verdieping in Wiki
krijgt een submap en gelijknamige note.
Na de Wiki/Raw-tegels volgt een leeg `vault-task-summary`-blok en dan
`![[vault-dossier.base#Open tasks]]`. Op Raw volgt na de snelkoppelingen een leeg
`vault-raw-summary`-blok en dan `![[vault-raw.base#Latest sources]]`.
Raw toont Status, Created, Note; taken tonen Status, Action date, Note.
Wiki-knoppen staan vóór eventuele onderbouwde feitkaarten. Gebruik `vault-wiki`
en voor feitkaarten `vault-wiki-overview`. Vanaf vier H2-koppen staat een leeg
`wiki-toc`-blok na de kaarten. Bronlinks staan bij de bewering en wijzen naar
de echte note/kop/passage. Gebruik waar passend een kort bronlabel met `↗`.

### Event folders en kop van de eventnote

Een Raw-pakket heet `YYYYMMDD-naam` met een gelijknamige hoofdnote en `type`,
`category`, `wiki_status`. De bewezen eventdatum bepaalt de pakketdatum;
`created` beschrijft vastlegging. Onbekende historische tijden blijven onbekend.
Bij `processed` horen `wiki_processed_on` en een exact `wiki_target` met kop,
plus zichtbare verwerking of een onderbouwde reden voor geen Wiki-update.
Event Note v1.1 gebruikt `vault-navigation`, `vault-dashboard`, `vault-event`,
`vault-event-v1-1`: H1 → Back/Finder → `[!wiki-facts|dossier]` met echt eigenaarpad
→ `vault-event-detail` → Context → Activity. Het bestaande `vault-task-detail`
hoort bij de expliciete oudere Event Note-variant, niet bij v1.1.
Na Activity staat alle inhoud in gedateerde, benoemde auteursbubbels, inclusief
provenance en archief. Lange inhoud staat onder de gevouwen `Meer`-callout binnen
dezelfde bubble. Quote iedere regel, ook fences en lege regels. Gebruik de
werkelijke auteur; behoud oorspronkelijke datum en taakidentiteit.
`Verwerking in Wiki` staat in de verwerkingsbubble met exacte koplink en
inhoudelijke samenvatting. Nieuwe/gewijzigde mailconcepten volgen
`.viggobsidian/core/mail-format.md`: To, Onderwerp en Body apart kopieerbaar;
bronbinding en conceptstatus daarbuiten binnen dezelfde bubble. Opmaak verstuurt
niets en verandert geen taakstatus.

Een bubble begint bijvoorbeeld met `> [!activity-message|assistant] Assistant`
onder een `> [!activity-day|YYYY-MM-DD]`-scheiding. Het tijd-/auteursstempel volgt
het bewezen kanaal; schrijf geen historische minuten bij benadering. `Context`
blijft vóór Activity. Status en actiedatum komen uit de taakmetadata; een lege
actiedatum blijft leeg. Een simpele taak krijgt geen verplicht tweede werkdossier;
een bestaand `work_dossier` blijft gebonden aan zijn echte statuseigenaar.

## Daily

Daily's staan in `10-daily/notes/YYYY-MM-DD.md`. Een embedded Base selecteert
`file.ctime.date() == date(this.file.name)`, sluit de daily zelf uit en sorteert
`file.ctime` aflopend. Dit vervangt handmatige agentlinklijstjes waar automatisch
overzicht volstaat; menselijke links en daginhoud blijven behouden. Een kopie
kan een nieuwe filesystem-aanmaakdatum krijgen: dit overzicht is geen historie.

## Bronvolledigheid en inhoudelijke acceptatie

Inventariseer bronpaden en bijlagen vóór omzetting. Bewaar originele bytes,
registratie en IDs; lees iedere inhoudelijke bron en verwerk haar bij de eigenaar.
Controleer fysieke dekking, citaten en Wiki-readback. Herbind bij moves echte
inkomende kop-/blok-/bijlagelinks en callers. Bewaar historische versies en
exacte bronlocators; herschrijf geen bron om checks groen te maken.
Een dataverhuizing is een afzonderlijk geautoriseerde stap.

## Contracten en lokale compatibiliteit

De machinebron is `schemas/navigation-contract.yaml`, geïnstalleerd als
`.viggobsidian/core/navigation-contract.yaml`. De Rust-engine gebruikt daaruit
de minimale metadata-, layer- en Raw-reviewregels. Het lokale
`99-system/contracts/vault-schema.yaml` behoudt domeinvelden, exacte uitzonderingen
en writerdelegaties. Generieke veldkopieën daarin zijn compatibiliteit voor
bestaande readers, geen zelfstandig nieuw beleid.
`folder-note-contract.yaml` en `project-content.yaml` blijven op bestaande lokale
paden als bevroren interfaces voor oudere folder-/topic-/workspace- en roadmap-
callers. Hun centrale collectie-indeling is uitsluitend legacy, geen default
voor nieuwe dossiers. Persoonsdefaults en exacte scopes worden niet gepubliceerd.
Wijzig die interfaces pas met caller-/gedragstests. Een YAML-include verandert
bestaande readers niet automatisch. Behoud historische herstelhashes.
Nieuwe tooling gebruikt het gedeelde navigatiecontract.

## Lokale controles

Vanaf de gekozen vaultroot:

```sh
./.agents/bin/vault-layout --strict pad/naar/dossier
./.agents/bin/vault-layout --indexes --paths pad/naar/note.md
./.agents/bin/vault-layout --check-all
```

`--json` geeft machineleesbare uitvoer. Exit 0: geen bevindingen; 1: afwijkingen;
2: uitvoering onmogelijk. `--check-all` controleert numerieke collecties plus
index-/tegeldekking en blijft na een bevinding latere roots controleren. Work
gebruikt daarnaast zijn bestaande `--check-work`-adapter. Beoordeel gedelegeerde
grenzen afzonderlijk: hun inhoud is niet door deze scan geaccepteerd.
In een geadopteerde vault is een ontbrekend/ongeldig gedeeld contract een
uitvoerfout. Standalone enginefixtures kunnen hun expliciete lokale schema houden.
Updates verlopen via preview, preimages, driftcontrole, apply, package-doctor en
lokale dieptechecks. Verander geen hashes of uitzonderingen om fouten te verbergen.

### Event Note v1.1-bubbels

Zie [[99-system/vault/vault-navigation/vault-navigation-handleiding#Principes en regels]] en de lokale adapter.

## Voorbeeldnotes

Zie [[99-system/vault/vault-navigation/vault-navigation-handleiding#Principes en regels]] en de lokale adapter.

## Een batch omzetten

Zie [[99-system/vault/vault-navigation/vault-navigation-handleiding#Principes en regels]] en de lokale adapter.

### Bronvolledigheid en inhoudelijke acceptatie

Zie [[99-system/vault/vault-navigation/vault-navigation-handleiding#Principes en regels]] en de lokale adapter.

### Uitvoering per dossier

Zie [[99-system/vault/vault-navigation/vault-navigation-handleiding#Principes en regels]] en de lokale adapter.

## FAQ · Implementatie en Rust-checker

Zie [[99-system/vault/vault-navigation/vault-navigation-handleiding#Principes en regels]] en de lokale adapter.

### Hoe zien de drie dossierpagina’s eruit?

Zie [[99-system/vault/vault-navigation/vault-navigation-handleiding#Principes en regels]] en de lokale adapter.

### Wat controleert de Rust-checker?

Zie [[99-system/vault/vault-navigation/vault-navigation-handleiding#Principes en regels]] en de lokale adapter.

### Waar volgen we de uitrol?

Zie [[99-system/vault/vault-navigation/vault-navigation-handleiding#Principes en regels]] en de lokale adapter.

### Welke opdracht gebruik ik per dossier?

Zie [[99-system/vault/vault-navigation/vault-navigation-handleiding#Principes en regels]] en de lokale adapter.

## Projecten en onderdelen

Zie [[99-system/vault/vault-navigation/vault-navigation-handleiding#Principes en regels]] en de lokale adapter.

