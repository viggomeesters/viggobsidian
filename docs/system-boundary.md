# Wat hoort waar?

Vanaf v0.4 is `docs/navigation.md` de ene actieve navigatiestandaard. De oude
folder-note-/workspace-/checknotenames worden als dunne releasebeheerde ingangen
geïnstalleerd met behoud van kopankers. Het generieke navigation-contract wordt
door de Rust-engine gebruikt; lokale exacte scopes en overige domeinregels
blijven in het bestaande vaultschema. Bevroren legacy-/roadmapinterfaces zijn
expliciet lokaal beschreven en krijgen geen nieuwe caller als navigatiedefault.
Hun historische herstelpins worden niet vernieuwd als bijeffect van deze release.

| Onderdeel | ViggObsidian | Lokale vault |
| --- | --- | --- |
| Navigatie, programma/project/onderdeel, Wiki/Raw | Canoniek beleid en profiel | Inhoudelijke projecten, namen en migraties |
| Schema | Generieke regels en integratievorm | Exacte scopes, historische uitzonderingen en writerdelegaties |
| CSS | Volledige gedeelde layout, tokens en stylesheetcompiler | Thema, routekleuren, iconen, identiteit en externe dossierafbeeldingen in appearance.json |
| Bases | Canonieke gedeelde assets | Versiegebonden installatie op bestaande embedpaden |
| Vaultskills | Zeven generieke publieke ingangen | Provideradapters, interne procedures en hostprofielkoppelingen |
| Rust-checker | Eén volledige layoutengine, packagecontrole, updater en regressies | Exacte scopes/delegaties, Work-pluginmanifest en bestaande writerproviders |
| Agentregels | Generiek bron-/autorisatie-/updatecontract | Root-AGENTS, domeinregels, taal-/hostvoorkeuren |
| Ingestion en registratie | Capabilitycontract, geen database | Claims, writers, queues, credentials, logs en bronopslag |
| Beheer en historie | Releasegeschiedenis van het product | Computerbeheer, bewijs, persoonlijke besluiten en archieven |

Een lokale versiegebonden installatie is geen tweede onderhoudbare bron. Bewaar haar hash en wijzig upstream. Historische documenten en actieve providers zijn evenmin duplicaten die je zomaar verwijdert.

Bij adoptie: inventariseer ieder bestaand `99-system`-pad; onderscheid inhoudelijk gelezen bronnen van naamtreffers. Controleer inkomende links en bestaande schema-/manifestcallers vóór verwijdering. Behoud bestaande notenamen als lokale ingang waar dat nodig is. Een korte adapter met een verwijzing is beter dan dezelfde generieke regels opnieuw uitschrijven. Verplaats geen persoonlijke systeemdocumenten of bronarchieven naar de publieke repository.

Vanaf v0.2 zijn beide diepe checkers samengevoegd. Installeer dezelfde gehashte bronnen op het bestaande cratepad; writerbindings controleren code, scope, binary en stijlconfiguratie. Kopieën in vaults zijn release-installaties, geen onderhoudsbronnen. Het zelfstandige backupgereedschap in een bestaande Work-crate blijft een lokale tool en valt buiten deze codeoverdracht. Er worden geen backupdiensten gestart of omgerouteerd.
