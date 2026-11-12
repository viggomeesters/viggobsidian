---
name: obsidian-vault
description: Gedeelde begrensde route voor obsidian-vault.
---

# obsidian-vault

Selecteer de doelvault expliciet en lees daar deze gedeelde contracten:

- `.viggobsidian/core/procedures/authority.md`
- `.viggobsidian/core/procedures/read-capture.md`

Lees de lokale capability `read` in `.viggobsidian/providers.json` en de daar
gebonden adapter voordat je een effect uitvoert. Controleer met
`vault-layout --vault ROOT --skill-route obsidian-vault`. De procedure is releasebeheerd;
lokale helpers behouden hun eigen authority en argumentcontract. Een aanwezige
procedure bewijst geen beschikbare provider. Alleen de gekozen werking uitvoeren;
legacy-intentie, bronopslag, kennisverwerking en taakuitvoering blijven gescheiden.
