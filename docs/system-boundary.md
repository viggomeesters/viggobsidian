# Wat hoort waar?

| Onderdeel | ViggObsidian | Lokale vault |
| --- | --- | --- |
| Navigatie, programma/project/onderdeel, Wiki/Raw | Canoniek beleid en profiel | Inhoudelijke projecten, namen en migraties |
| Schema | Generieke regels en integratievorm | Exacte scopes, historische uitzonderingen en writerdelegaties |
| CSS | Generieke layout/tokens | Thema, actieve snippets, routekleuren en iconen |
| Bases | Canonieke gedeelde assets | Versiegebonden installatie op bestaande embedpaden |
| Vaultskills | Zeven generieke publieke ingangen | Provideradapters, interne procedures en hostprofielkoppelingen |
| Rust-checker | Gedeelde package-/structuurcontrole en updater | Bestaande dieptechecks zolang gedrag nog niet gedeeld is |
| Agentregels | Generiek bron-/autorisatie-/updatecontract | Root-AGENTS, domeinregels, taal-/hostvoorkeuren |
| Ingestion en registratie | Capabilitycontract, geen database | Claims, writers, queues, credentials, logs en bronopslag |
| Beheer en historie | Releasegeschiedenis van het product | Computerbeheer, bewijs, persoonlijke besluiten en archieven |

Een lokale versiegebonden installatie is geen tweede onderhoudbare bron. Bewaar haar hash en wijzig upstream. Historische documenten en actieve providers zijn evenmin duplicaten die je zomaar verwijdert.

Bij adoptie: inventariseer ieder bestaand `99-system`-pad; onderscheid inhoudelijk gelezen bronnen van naamtreffers. Controleer inkomende links en bestaande schema-/manifestcallers vóór verwijdering. Behoud bestaande notenamen als lokale ingang waar dat nodig is. Een korte adapter met een verwijzing is beter dan dezelfde generieke regels opnieuw uitschrijven. Verplaats geen persoonlijke systeemdocumenten of bronarchieven naar de publieke repository.

Deze release laat bestaande diepe checkers bewust lokaal. Hun private en werkomgevingsvarianten kunnen verschillen. Samenvoeging vereist geanonimiseerde gedragstests die beide varianten dekken; alleen een identieke bestandsnaam bewijst geen gelijk gedrag.
