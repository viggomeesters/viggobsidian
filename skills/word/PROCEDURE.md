---
name: word
description: Routeer een expliciet verzoek over een Word-bestand naar een beschikbare documentprovider met behoud van de gevraagde scope. Bevat geen gebundelde vendorimplementatie.
---

# Word-provider kiezen

Bepaal het werkelijke invoerbestand of de verbonden documentsessie en het gevraagde resultaat. Selecteer de doelroot uit de opdracht; lees daar eventueel `.agents/skill-adapters/word.md` voor de aangewezen provider en lokale bestandsregels. Beschikbaarheid van een tool verleent geen extra edit-, opslag- of publicatieautorisatie.

Ontdek en lees de beschikbare officiële document-skill of sessiecapabilities die het gevraagde formaat ondersteunen. Laat die provider de Wordbewerking en haar formaat-/rendercontrole uitvoeren. Gebruik `output` wanneer een opleverbestand is gevraagd. Een ontbrekende provider wordt concreet gemeld; installeer of kopieer geen onbekend pakket als stille fallback.

Draag bronbestand, gevraagde wijziging, bestemming en relevante gebruikerskeuzes over. Laat de gebruiker bepalen wat inhoudelijk wordt gewijzigd; bewaar ongevraagde delen en de oorspronkelijke bron volgens de opdracht. Maak onderscheid tussen analyse, voorgestelde tekst en daadwerkelijk opgeslagen output. Claim geen geslaagde Wordoplevering zonder het resultaat en de door de provider bewezen controle.

Deze algemene router bevat geen code, prompts, templates of referenties uit een niet-vrij herdistribueerbaar documentpakket. Vendorbestanden behouden hun eigen oorspronkelijke eigenaar en voorwaarden; hun aanwezigheid is geen distributierecht voor ViggObsidian. Nieuwe code of een eigen formaatprovider vraagt een afzonderlijke implementatiekeuze.
