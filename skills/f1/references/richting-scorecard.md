# Richting en scorecard

Kies alleen dimensies die de beslissing beïnvloeden. Dit zijn voorbeelden, geen verplichte 35-puntenaudit:

| Perspectief | Mogelijke dimensies |
|---|---|
| Developer | Correctheid, configuratie/state, foutafhandeling, data, codeorganisatie, documentatie, verificatie, AI-gebruik, delivery, automatisering, security, samenwerking |
| AI | Initiatief, context/geheugen, herstel, tools, coördinatie, leren, kwaliteitscontrole, aansturing, observatie, gedrag, vertrouwen, onderhoudbaarheid |
| Human | Gebruikswaarde, handelingen, aandacht, begrip, regie en houdbaarheid; bij expliciet persoonlijk werk diens eigen relevante levensdomeinen |

Een richtingskaart kan in enkele zinnen. Fictief voorbeeld: “Voorbeeldapp: minimale zoekvertraging en omvang, behoud zoekkwaliteit/configuratie/toegankelijkheid/stabiliteit, ook kleine winst onderzoeken; uitvoering tot geïnstalleerde en gecontroleerde versie.” Dat is een richting, geen gemeten baseline of automatische toestemming om een andere app te wijzigen.

## Bewijsgebonden scorecard

Gebruik per gekozen dimensie **huidig | doel | resultaat | bewijs/datum/zekerheid | afweging/volgende stap**. Vermeld het perspectief in de regelnaam. Laat ongebruikte kolommen weg bij een klein advies; onderscheid blijft inhoudelijk aanwezig.

- Houd bij metingen eenheid, versie, omstandigheden, werklast, aantal waarnemingen en variatie bij. Vergelijk gelijkwaardige grootheden. Een lokaal modelbenchmark bewijst geen fysieke schermlatentie of batterijwinst.
- Zonder actuele bron is de toestand onbekend. Onbekend, onvoldoende en niet van toepassing zijn verschillende oordelen. Gebruik de reden voor niet-toepasselijkheid.
- Geef kwalitatieve oordelen een concrete waarneming en grens. Gebruikersfeedback is bewijs voor de gemelde ervaring, geen universele snelheidsmeting.
- Vereiste correctheid, privacy, herstel en andere afgesproken invarianten kunnen niet worden weggemiddeld. Een kleiner maar instabiel product voldoet niet aan “kleiner met behoud van stabiliteit”. Een echte conflicterende prioriteit vraagt een expliciete afweging.
- Minder menselijke interventies telt alleen samen met juiste resultaten, passende toestemming en herstel. Een autonome fout blijft een fout.
- Vergelijk eigen inzet en onderhoudslast eerlijk. Een ingewikkelde oplossing verdient geen hogere score omdat ze meer technieken gebruikt.

## 9000x als ambitietaal

Gebruik de labels alleen wanneer ze helpen bij het gekozen onderwerp. Het zijn ordinale ontwikkelniveaus, geen aantallen productiviteit of menselijke waarde. Geen optelsom, gemiddelde, vermenigvuldiging, capaciteitpercentage of fictief totaalcijfer. Het passende doel hoeft niet overal 9000x te zijn.

Concrete ankers, te verfijnen met taakbewijs:

| Label | Developer: terugkerende fout | AI: taakuitvoering | Human: gebruik |
|---|---|---|---|
| nonx | Probleem wordt niet herkend | Uitkomst ongecontroleerd | Frictie onbegrepen |
| 1x | Individueel herstellen | Losse begeleide opdracht | Frictie kunnen benoemen |
| 10x | Herhaalbare oplossing | Vaste context en controles | Bewuste werkbare routine |
| 100x | Geteste gedeelde oplossing | Betrouwbare overdracht en herstel | Geobserveerde feedback stuurt ontwerp |
| 1000x | Structurele oorzaak grotendeels weg | Samenwerking houdt stand bij relevante uitzonderingen | Samenhangend systeem vermindert aantoonbaar belasting |
| 9000x | Benoemde foutcategorie door ontwerp voorkomen, met bewijs en grenzen | Mens/agent leveren aantoonbaar passend resultaat met minimale onnodige regie | Gebruik sluit aantoonbaar aan zonder onnodig systeembeheer; menselijke keuze blijft |

Deze ankers zijn geen universele ladder voor iemands leven. Ken een niveau alleen toe voor het onderzochte domein en het getoonde bewijs; markeer gewenste niveaus als doel. Een architectuurclaim “fouten onmogelijk” vereist een begrensde foutcategorie en passende verificatie, niet alleen overtuiging.

Voorbeeld proces: huidig drie overdrachten per aanvraag uit vijf geobserveerde dossiers; doel één eigenaar en behoud verplichte controle; resultaat nog onbekend tot pilot; bewijs vijf dossiers, beperkte steekproef. Geen claim “3× effectiever” uit alleen minder overdrachten.
