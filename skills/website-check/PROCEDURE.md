---
name: website-check
description: Controleer een gekozen website op bereikbaarheid, actuele inhoud, ontbrekende ingangen en primaire werking. Rapporteert bewijs en voorstellen; wijzigt de website niet.
---

# Websitecontrole

Lees de concrete vraag, gekozen site en repo-/projecteigenaar. Selecteer de doelroot expliciet en lees daar eventueel `.agents/skill-adapters/website-check.md`. Site-URL, inventaris, bestaande upkeep-CLI en presentatievoorkeuren horen bij de betreffende siteconfig, niet in deze algemene procedure. Gebruik `audit` voor de bevindingen en `research` voor nader brononderzoek.

## Begrensd onderzoeken

Lees de bestaande repo-instructies en controleer of een betrouwbare read-only websitechecker of offline fixture beschikbaar is. Inspecteer zijn scope en argumentcontract vóór gebruik. Gebruik voor uitvoer het expliciete rapportpad of de geldende tijdelijke bestemming; schrijf geen status, schedulerconfig of websitebestanden als bijeffect.

Controleer de gekozen homepage/routes, zichtbare ingangen, links en door de opdracht relevante demo's. Leg meetmodus, controledatum, daadwerkelijk bezochte URLs, bronversie/hashes waar beschikbaar en fouten vast. Een offline fixture blijft offline; ontbrekende metingen worden niet stil met live netwerk aangevuld. Respecteer toegangsgrenzen en beperk de scan tot de opdracht; geen brede crawler als vervanging voor een ontbrekende provider.

## Volledigheid en werking

Vergelijk de zichtbare selectie onafhankelijk van de gebruikte inventaris met relevante actuele publieke projectbronnen of een tweede aangewezen bron. Een inventaris en de daaruit gegenereerde pagina zijn niet onafhankelijk. Repoactiviteit bewijst geen nieuwe bruikbare demo. Zonder tweede bron blijft volledigheid onbekend.

Controleer gewijzigde beschrijvingen tegen de werkelijke functie en publieke bron. Een oudere tekst of manifestwaarde is niet vanzelf de waarheid. HTTP 200 bewijst bereikbaarheid, geen geslaagde interactie.

Doe per relevante demo een kleine, niet-verstorende rooktest met ondersteunde achtergrondbediening: bijvoorbeeld openen, een veilige lokale invoer en het verwachte zichtbare resultaat. Geen echte berichten, aankopen, externe datamutaties of accountwijzigingen. Leg handeling en resultaat vast; als testen niet kan, vermeld functioneel niet getest. Een screenshot alleen bewijst de handeling niet.

## Oordeel en oplevering

Lees intro, labels, beelden en acties in hun echte context. Beoordeel of de site het gekozen gebruikersdoel ondersteunt en noodzakelijke ingangen vindbaar zijn. Gebruik de actuele frontend-/copygate bij zichtbare copy; verzin geen aanvullend kopje of slogan om een rapport compleet te laten lijken.

Onderscheid bewezen defecten, inhoudelijke voorstellen en onbekende metingen. Timeout, rate limit, ontbrekende rechten en onbereikbare bron zijn geen geslaagde controle. Prioriteer concrete bevindingen op effect en lever bron/route, bewijs en aanbevolen vervolgstap. Forceer geen verbeterpunt wanneer de onderzochte scope goed is.

Bewaar gevraagd bewijs via de bestaande eigenaar/writer. Een check verandert geen copy, inventaris, pagina, scheduler, deployment of Gitgeschiedenis. Herstel of publicatie vereist zijn eigen geautoriseerde opdracht en repo-contract.
