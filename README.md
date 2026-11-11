# ViggObsidian

Versiebeheer voor het beleid, de structuur, opmaak en agentvaardigheden van een Obsidian-vault.

Eén profiel: `20-projects` voor losse projecten, `21-programma` tot en met `29-programma` voor programma’s, `YYYY-MM-projectnaam` voor projecten en `code-NN-onderdeel` met Wiki en Raw. Deelnemers en rollen staan in het project-00. Daily’s tonen automatisch de die dag aangemaakte notes, nieuwste bovenaan.

- [Beleid en Vault Navigation](docs/policy.md)
- [Profiel](profiles/numbered-v1.yaml), [vaultschema](schemas/vault-schema.yaml) en [integratieschema](schemas/vault.schema.json)
- [Vaultskills en lokale adapters](docs/skills.md)
- [Installeren, updates en rollback](docs/updates.md)
- [Afbakening en migratie uit bestaande systemen](docs/system-boundary.md)

## Rust CLI

```sh
cargo test --locked
cargo build --release --locked
./target/release/viggobsidian --help
```

`check` controleert pakketintegriteit en de project-/onderdeelstructuur. Bestaande diepere layout-, bronregistratie- en taakcontroles blijven expliciete lokale adapters. Deze eerste release claimt geen volledige vervanging van bestaande vaultcheckers.

`plan` toont een versiegebonden update; `apply` weigert lokale drift en bewaart herstelbytes; `rollback` herstelt alleen een passende, ongewijzigde postimage. Inhoud, secrets, runtimeproviders en persoonlijke configuratie staan buiten het pakketoppervlak. Installatie start geen achtergrondservice en publiceert geen vaultinhoud.

## Bijdragen

Leg een wijziging vast in een Issue, maak een PR met lokale controles en onafhankelijke review, merge de beoordeelde delta en lever een release. Gebruik geen GitHub Actions. Meld bij een fout de pakketversie, checkeruitkomst en een geanonimiseerde reproductie; upload geen authentieke Raw-bronnen of herstelreceipts. De repository beheert uitvoering via haar lokale `.go/`-contract.

MIT. Bestaande Bases en het Go-bootstrapmechanisme zijn overgenomen uit eigen beheer; zie [herkomst](docs/provenance.md).
