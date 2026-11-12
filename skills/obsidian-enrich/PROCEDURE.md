---
name: obsidian-enrich
description: Gedeelde begrensde route voor obsidian-enrich.
---

# obsidian-enrich

Selecteer de doelvault expliciet en lees daar deze gedeelde contracten:

- `.viggobsidian/core/procedures/authority.md`
- `.viggobsidian/core/procedures/process.md`
- `.viggobsidian/core/procedures/daily.md`
- `.viggobsidian/core/procedures/maintenance-context.md`

Lees de lokale capability `refresh` in `.viggobsidian/providers.json` en de daar
gebonden adapter voordat je een effect uitvoert. Controleer met
`vault-layout --vault ROOT --skill-route obsidian-enrich`. De procedure is releasebeheerd;
lokale helpers behouden hun eigen authority en argumentcontract. Een aanwezige
procedure bewijst geen beschikbare provider. Alleen de gekozen werking uitvoeren;
legacy-intentie, bronopslag, kennisverwerking en taakuitvoering blijven gescheiden.
