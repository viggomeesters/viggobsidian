# ViggObsidian

Versiebeheer voor het beleid, de structuur, opmaak en agentvaardigheden van een Obsidian-vault.

ViggObsidian blijft de canonieke Rust-engine, pakketbron en native CLI. [ViggVault](https://github.com/viggomeesters/obsidian-vigg-vault) is het gebruikersproduct en de Obsidian-interface. Beide blijven bestaan met hun eigen versies; zie het [product-/enginecontract](docs/product-engine-boundary.md). ViggVault 0.1.1 biedt alleen opt-in Work-linkstijl; de gedeelde Wiki-tekstcore levert nog geen extra plugincommando.

Eén profiel: `20-projects` voor losse projecten, `21-programma` tot en met `29-programma` voor programma’s, `YYYY-MM-projectnaam` voor projecten en `code-NN-onderdeel` met Wiki en Raw. Deelnemers en rollen staan in het project-00. Daily’s tonen automatisch de die dag aangemaakte notes, nieuwste bovenaan.

- [Beleid en Vault Navigation](docs/policy.md)
- [Profiel](profiles/numbered-v1.yaml), [vaultschema](schemas/vault-schema.yaml) en [integratieschema](schemas/vault.schema.json)
- [Vaultskills en lokale adapters](docs/skills.md)
- [Algemene skillbibliotheek en werk-/privéadapters](docs/skill-library.md)
- [Installeren, updates en rollback](docs/updates.md)
- [Afbakening en migratie uit bestaande systemen](docs/system-boundary.md)

## Rust CLI

```sh
cargo test --workspace --locked
cargo build --workspace --release --locked
./target/release/viggobsidian --help
./target/release/vault-layout --vault /path/to/vault --strict 20-projects
./target/release/vault-layout --vault /path/to/vault --check-all
```

`doctor` controleert pakketintegriteit, de integratieconfiguratie en de afgeleide CSS. `check` toetst daarnaast de project-/onderdeelstructuur aan het profiel. De gedeelde `vault-layout`-engine verzorgt de volledige layout-, navigatie-, broninventaris-, Event Note- en projectcodecontrole. `--check-all` bezoekt alle genummerde collecties; `--check-work` voegt het expliciet aangesloten Work-package- en linkstijlcontract toe. Registratie- en domeinwriters behouden hun lokale authority.

De volledige actieve CSS heeft één bron. `folder-note.css` wordt samengesteld uit `assets/folder-note.css` en lokale `.viggobsidian/appearance.json`; `agents-callout.css` en `active-note-contrast.css` komen rechtstreeks uit de release. Exacte bestemmingen, avataridentiteit en externe dossierafbeeldingen staan uitsluitend lokaal. Geïnstalleerde checkers en CSS zijn reguliere, gehashte distributiebestanden op bestaande paden. Wijzig de bron upstream en update beide installaties; onderhoud geen lokale codefork.

`plan` toont een versiegebonden update; `apply` weigert lokale drift en bewaart herstelbytes; `rollback` herstelt alleen een passende, ongewijzigde postimage. Inhoud, secrets, runtimeproviders en persoonlijke configuratie staan buiten het pakketoppervlak. Installatie start geen achtergrondservice en publiceert geen vaultinhoud.

## Bijdragen

Leg een wijziging vast in een GitHub Issue, werk op een branch, maak een PR met lokale controles en onafhankelijke review, merge de beoordeelde delta en lever waar nodig een release. Gebruik geen GitHub Actions. Meld bij een fout de pakketversie, checkeruitkomst en een geanonimiseerde reproductie; upload geen authentieke Raw-bronnen of herstelreceipts. Gebruik geen `.go`-contract, stackbootstrap of lokale workflowqueue.

MIT. Bestaande Bases zijn overgenomen uit eigen beheer; zie [herkomst](docs/provenance.md).
