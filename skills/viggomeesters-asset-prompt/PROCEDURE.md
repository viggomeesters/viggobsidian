---
name: viggomeesters-asset-prompt
description: Maak een brongebonden visual-assetprompt voor een gekozen site of publicatie, met stijlbinding, verplichte labels en provenance. Historische naam; geen ingebakken persoonsmerk.
---

# Assetprompt vanuit een bestaande stijl

Gebruik deze route voor een afbeeldingsprompt die moet aansluiten op een bestaande visuele identiteit. Voor HTML-compositie of een interactief rapport gebruik je `visual-output`; dit is geen tweede HTML-renderer.

Selecteer de content-/stijleigenaar uit de opdracht. Lees eventuele `.agents/skill-adapters/viggomeesters-asset-prompt.md` vanaf de expliciet gekozen doelroot. Een publieke siterepository kan haar eigen site-/assetconfig en stijlcontract bezitten; dat maakt die config geen privégeheim en rechtvaardigt geen duplicaat in een privévault. Volg geen persoonlijke hardcoded hostpaden.

## Bronrollen

Lees de gezaghebbende stijlbeschrijving, relevante prompttemplate, assetcatalogus en bestaande referentiebeelden. Houd canonieke stijl, aanvullende layoutinspiratie, uitvoer/generator en consumerend project uit elkaar. Een generatorrepo is geen stijlbron tenzij expliciet aangewezen. Door de gebruiker gegeven merk- of screenshotreferenties gaan voor een algemene bibliotheek.

Kies en inspecteer relevante echte voorbeelden. Leg vast welke bron de typografie, kleur, compositie en dichtheid bepaalt. Een catalogusregel zonder het beeld te zien is onvoldoende voor een getrouwe stijlclaim. Bij onbereikbare bronnen blijft de stijlmatch onbewezen; schrijf geen verzonnen stijlversie.

## Promptpakket

Lever een bruikbaar pakket met:

- assetdoel: hero, Open Graph, diagram, projectkaart of ander gevraagd gebruik;
- doelpublicatie/site, relevante paginatitel, slug en datum voor zover bekend;
- stijlbron en bewezen versie, plus primaire en eventuele aanvullende referenties;
- kernboodschap, concrete compositie en vereiste leesbare labels;
- toepasselijke negatieve constraints: wat zou het doel of de stijl aantoonbaar schaden;
- generatorprompt en eventueel een afzonderlijke negatieve prompt;
- voorgestelde assetlocatie en prompt-/provenancerecord bij de consumerende eigenaar.

Gebruik echte labels en de beschikbare broninhoud. Vul geen marketingclaims, statistieken, testimonials of logo's in om de afbeelding te vullen. Een onbekende publicatiedatum, versie of generator blijft onbekend. Schrijf de prompt voor het daadwerkelijke gekozen generatie-/bewerkingsmiddel; een promptpakket is nog geen gegenereerd beeld.

## Publiek veilig delen

Behandel private werk-/klantbeelden als bronmateriaal met eigen toegang, niet als vrij te publiceren assets. Geef in publieke provenance hoogstens de noodzakelijke veilige broncategorie of reeds publieke bronlink; exporteer geen lokale paden, klantnamen, interne URL's, tickets, secrets of private project-ID's. Anonimiseren vraagt inhoudelijke beoordeling, niet uitsluitend een termscan.

Een opdracht om een assetprompt te maken autoriseert geen upload van bronmateriaal, publicatie, repositorypush of wijziging van een site. Houd onduidelijke rechten/toegang zichtbaar en gebruik geen ongeautoriseerde merkassets.

## Controle

Controleer dat prompt en metadata hetzelfde onderwerp en doel beschrijven, labels daadwerkelijk uit de bron komen, stijl-/referentierollen kloppen en alle noodzakelijke compositie-informatie aanwezig is. Bij expliciet gevraagde generatie of publicatie volgen vervolgens de desbetreffende skill, bronrechten, uitvoergates en autorisatie. Bewaar de bewerkbare prompt en het resultaat bij dezelfde inhoudelijke eigenaar.
