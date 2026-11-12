---
name: vault-task
description: "Voer een concrete vaultopdracht uit vanuit de bestaande taaknote of tekstintake, met stabiele identiteit en acceptatiebewijs."
---

# Taak uitvoeren

Lees voor nieuwe of gewijzigde mailconcepten `.viggobsidian/core/mail-format.md`
en gebruik `.viggobsidian/core/mail-draft.md`: To, Onderwerp en Body apart
kopieerbaar in dezelfde auteursbubble, adresbewijs en conceptstatus erbuiten.
Verzend niets en wijzig geen taakstatus als gevolg van conceptopmaak.

Lees het doelcontract uit [vault](../vault/SKILL.md) en de lokale `task`-adapter. Een expliciet taakpad moet bestaan, binnen de gekozen vault liggen en volledig gelezen zijn. Een fout pad is geen nieuwe tekstintake. Zoek bij tekst eerst een bestaande eigenaar en dezelfde onafgeronde taak; maak geen duplicaat.

Bewaar oorspronkelijke input via de ondersteunde intakewriter. Gebruik bestaande taak-/actie-/projectidentiteiten en het lokale ID-uitgiftecontract. Lees eigenaar, actuele status, bronnen, eerdere besluiten en eventuele gebonden werkdossier. Respecteer alleen-plannen en advies. Leg concrete acceptatie en afhankelijke stappen vast in dezelfde taak; maak geen tweede globaal register.

Voer geautoriseerd werk tot het afgesproken eindpunt uit. Repositorywerk volgt GitHub Issue → branch → PR → lokale controles en onafhankelijke review → merge binnen autorisatie. Gebruik het doelrepo en verwijs vanuit de bestaande taak naar Issue/PR/bewijs; maak geen tweede technisch taakregister en vereis geen `.go`-state of Go-runtime. Een expliciete plan- of PR-only-opdracht behoudt haar eindpunt. Updates mogen geen writerauthority, credentials of andere dossiers veranderen. Lees writes terug, toets echte acceptatie en draai de lokale finish-/layoutcontroles. Rapporteer resultaat, bewijs en resterende beperking; een bestand of groene checker alleen bewijst geen afgeronde taak.
