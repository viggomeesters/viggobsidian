---
name: vault-document-capture
description: Gedeelde begrensde route voor vault-document-capture.
---

# vault-document-capture

Selecteer de doelvault expliciet en lees daar deze gedeelde contracten:

- `.viggobsidian/core/procedures/authority.md`
- `.viggobsidian/core/procedures/read-capture.md`

Lees de lokale capability `document` in `.viggobsidian/providers.json` en de daar
gebonden adapter voordat je een effect uitvoert. Controleer met
`vault-layout --vault ROOT --skill-route vault-document-capture`. De procedure is releasebeheerd;
lokale helpers behouden hun eigen authority en argumentcontract. Een aanwezige
procedure bewijst geen beschikbare provider. Alleen de gekozen werking uitvoeren;
legacy-intentie, bronopslag, kennisverwerking en taakuitvoering blijven gescheiden.
