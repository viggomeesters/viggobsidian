# Proceduredekking

`schemas/procedures.json` is de onderhoudbare machinekaart. De release installeert
43 bestaande routes (7 publieke ingangen, 36 interne/compatibilityprocedures),
36 actieve referentielocators, de native-vaultbridge en acht gedeelde contracten.
De publieke native lijst groeit niet. Alle callerpaden blijven stabiel.

| Onderzocht oppervlak | Gedeelde opvolger | Lokale grens |
| --- | --- | --- |
| vault, read, log, capture en documentcapture | authority; read-capture | exacte reader/extractor, byteprovider, bronregister, privacy |
| process/inbox; mail; sociale berichten | process; batches-imports; daily | bronadapters, domeincompletiongates, echte claims en pakketmoves |
| task; action-notes; lifecycle; task-changes | task | bestaande IDs, datumbeleid, status-/actie-/finish-/Issue-provider en eindige repairs |
| review/tackle; hub assessment/layout/reuse/scoring/updates/publication | selection-hub | canonieke voorraad, lokale beschikbaarheid, audit/cache/feedback-/claimimplementatie |
| project-refresh; compile; briefing-link; contact/context | maintenance-context | gerichte ownerwriter, generated-receipts, dailylock, persoons-/legacyauthority |
| bookmark-/prepared-/parallelreferenties; iMessage/Reading List | batches-imports | bestaande importers en bronmanifests; geen nieuwe collector |
| enrich/inbox/briefing/obsidian-compatibilityaliases | dezelfde effectcontracten | dagelijkse toestemming, concrete provider, geen bronrootfallback |
| audio/foto/berichtbewijs/web/video; mailaliases; contacts | read-capture/process/maintenance-context | werkelijke extractor en live-accounttoegang; geen automatische send of archive |
| native-vaultbridge voor persoonlijke domeinskills | authority plus gekozen effectcontract | persoonlijke domeinregels blijven onder hun eigen lokale eigenaar |

De primaire inventaris omvatte alle 179 assets onder vault/vault-*:
64 Markdown, 107 Python, 5 JSON en 3 YAML. Alle 64 Markdown zijn inhoudelijk
beoordeeld. Van 42 referentiedocumenten zijn 35 generieke/gemengde active docs
gesplitst naar gedeelde contracten plus providerbinding. Zes historische JSONL-
recepten en één eindige verhuisreferentie blijven lokale, inerte geschiedenis.
Er zijn geen writerimplementaties naar een nieuw runtimeoppervlak verplaatst.

De actieve moduleclosure is bovendien beoordeeld: 24 procedures buiten de prefix.
21 generieke aliases/capture-/workspaceprocedures gaan mee, evenals de actieve
periodeselectiereferentie; nutrition en twee persoonlijke meeting-/voorbereidings-
procedures blijven domein-eigen en bereiken de algemene regels via native-vault.
Die domeinprocedures zijn geen niet-onderzochte generieke restcategorie. De overige
persoonlijke SKILL.md-domeinen zijn geïdentificeerde afhankelijkheden, geen claim
dat hun volledige inhoud in deze migratie is gereviewd.

Private exacte paden, oorspronkelijke hashes/versies, persoonlijke uitzonderingen
en onderliggende source-/registry-/writerimplementaties blijven in de lokale
bewijsinventaris. Python is hier bestaande providerimplementatie, geen nieuwe
gedeelde runtime. Geselecteerde CLI-interfaces en bestaande gedragstests bewaken
de aansluiting; er is geen volledige audit van alle 107 Pythonbestanden geclaimd.

Nieuwe gedeelde code en controles zijn Rust, met bestaande gelockte dependencies.
De vier routeproeven controleren expliciete rootbinding, source-adapter als extra
afhankelijkheid, volledige capabilitydekking, missing/drift/symlink-/manifest-
weigering en het ontbreken van write-/provideruitvoering. Pakkettests bewaken de
gecompileerde allowlist, adoptiepreimage en terugweg.
