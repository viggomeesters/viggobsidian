---
name: vault-project-refresh
description: Gedeelde begrensde route voor vault-project-refresh.
---

# vault-project-refresh

Selecteer de doelvault expliciet en lees daar deze gedeelde contracten:

- `.viggobsidian/core/procedures/authority.md`
- `.viggobsidian/core/procedures/maintenance-context.md`
- `.viggobsidian/core/procedures/process.md`

Lees de lokale capability `refresh` in `.viggobsidian/providers.json` en de daar
gebonden adapter voordat je een effect uitvoert. Controleer met
`vault-layout --vault ROOT --skill-route vault-project-refresh`. De procedure is releasebeheerd;
lokale helpers behouden hun eigen authority en argumentcontract. Een aanwezige
procedure bewijst geen beschikbare provider. Alleen de gekozen werking uitvoeren;
legacy-intentie, bronopslag, kennisverwerking en taakuitvoering blijven gescheiden.
