---
name: vault-hub
description: "Maak compacte afgeleide taaknavigatie of verwerk menselijke hubfeedback bij de oorspronkelijke taaknotes."
---

# Takenhub

Lees vóór handelen alle gedeelde contracten voor route `vault-hub` uit
`.viggobsidian/core/procedures.json` in de gekozen doelvault. Controleer de lokale
capability met `vault-layout --vault ROOT --skill-route vault-hub` en lees de
gebonden providerinstructies. Een groene check vervangt deze inhoudelijke lezing niet.

Lees het doelcontract uit [vault](../vault/SKILL.md) en de lokale `hub`-adapter. Lees de canonieke taakselectie binnen de gekozen scope en de volledige relevante taaknotes. Maak een gedateerde momentopname met één duidelijke verwijzing per bestaande taak, context en nuttige volgorde. De hub bezit geen afvinkstatus en creëert geen tweede taak-ID.

Bij menselijke feedback: lees de actuele brontaak en eerdere hubcontext, onderscheid een vraag van een uitvoerbare opdracht, en werk alleen geautoriseerde status-/inhoudsdelta’s via de lokale writer terug. Bewaar de feedback en bronbinding. Een oude hub mag een nieuwere taakstatus niet overschrijven. Controleer actuele hashes en echte links; meld conflicten en de uiteindelijke bronstatus.
