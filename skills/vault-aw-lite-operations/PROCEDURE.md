---
name: vault-aw-lite-operations
description: Gedeelde begrensde route voor vault-aw-lite-operations.
---

# vault-aw-lite-operations

Selecteer de doelvault expliciet en lees daar deze gedeelde contracten:

- `.viggobsidian/core/procedures/authority.md`
- `.viggobsidian/core/procedures/maintenance-context.md`

Lees de lokale capability `legacy` in `.viggobsidian/providers.json` en de daar
gebonden adapter voordat je een effect uitvoert. Controleer met
`vault-layout --vault ROOT --skill-route vault-aw-lite-operations`. De procedure is releasebeheerd;
lokale helpers behouden hun eigen authority en argumentcontract. Een aanwezige
procedure bewijst geen beschikbare provider. Alleen de gekozen werking uitvoeren;
legacy-intentie, bronopslag, kennisverwerking en taakuitvoering blijven gescheiden.
