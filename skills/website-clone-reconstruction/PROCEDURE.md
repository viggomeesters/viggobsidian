---
name: website-clone-reconstruction
description: Reconstrueer een gekozen website in geautoriseerde eigen code op basis van waargenomen inhoud, vorm en interactie. Gebruik bij een expliciete reconstructie; inspiratie blijft ontwerpprincipes overnemen.
license: MIT
---

# Website reconstrueren

Deze procedure begeleidt het terugbouwen van een gekozen pagina of site. Een visuele referentie zonder reconstructieverzoek gebruikt de normale frontend-route. Selecteer de doelrepository expliciet en lees daar de repo-instructies en eventueel `.agents/skill-adapters/website-clone-reconstruction.md`. Behoud de bestaande stack, componenten, tokens en validatie.

## Scope en bronrechten

Bepaal target-URLs, gewenste nauwkeurigheid en de relevante gebruiksrechten uit de opdracht. Een publieke pagina maakt merkassets, tekst en foto's niet automatisch herbruikbaar. Beperk ongeautoriseerde externe voorbeelden tot ontwerpprincipes en eigen vervangende inhoud. Gebruik geen private of betaalmuurinhoud buiten de gegeven toegang en maak geen misleidende nabootsing. Vraag alleen ontbrekende context die de geautoriseerde reconstructie werkelijk bepaalt.

Bewaar bron-URL, inspectiedatum en bekeken viewports/staten. Onderscheid gekopieerde, gerefereerde en vervangen assets, met hun bekende herkomst. Een ontbrekend distributierecht blijft zichtbaar; geen stille opname van onbekende upstreamcode of tekst.

## Waarnemen vóór bouwen

Inspecteer de echte pagina met een browser of betrouwbare reeds aangeleverde browserresultaten. Alleen een herinnering of screenshot van één toestand is onvoldoende voor een interactief resultaat. Leg de relevante desktop- en mobiele weergave vast en onderzoek:

- volgorde en rol van secties, overlays, sticky/fixed elementen en scrollcontainers;
- typografie, kleuren, ruimte, afmetingen, beelden, iconen en betekenisvolle breekpunten;
- normale, open, geselecteerde, fout-, laad- en lege toestanden voor zover de scope ze omvat;
- scroll-, klik-, hover-, toetsenbord- en tijdgestuurd gedrag;
- gelaagde media, achtergrondbeelden en decoratieve versus betekenisvolle assets.

Test vooral het gedrag dat de architectuurkeuze bepaalt. Een scrollgestuurde sectie is geen kliktab en een time-based carrousel is geen statische kaart. Observeer relevante triggers en overgangen; verzin geen exacte waarden waar de bron ze niet bewijst. Gebruik berekende stijlen of DOMmetingen wanneer nauwkeurigheid telt.

## Van bewijs naar componenten

Maak per betekenisvol onderdeel een beknopte specificatie met bronbewijs, rol in de pagina, inhoud, interactiemodel, benodigde assets, responsgedrag en toetsbare acceptatie. Verbind onbekende delen aan een concrete ontbrekende waarneming. Een specificatie ondersteunt de uitvoering, geen verplicht nieuw documentregister voor een kleine eenvoudige kopie.

Leg componentgrenzen zo dat samenhangende toestanden bij elkaar blijven. Behoud de bestaande foundation voor fonts, spacing, kleuren, routes en metadata wanneer die het doel ondersteunt. Introduceer geen library om het gewenste gedrag te raden; voeg alleen de benodigde bestaande of onderbouwde techniek toe.

Bij expliciet toegestane parallelle uitvoering krijgt iedere bouwer een begrensd onderdeel met het bronbewijs, gedrag, doelbestanden, afhankelijkheden en controles. Een reconstructieverzoek zelf vraagt niet automatisch om subagents.

## Bouwen en controleren

Werk onderdeel voor onderdeel en verbind daarna paginabreed gedrag. Bij een mismatch onderzoek je of de waarneming, specificatie of implementatie verkeerd was; maak een onjuist interactiemodel niet cosmetisch passend.

Controleer met de repositorylokale build-/statische controles en browserresultaten:

- target versus resultaat op desktop en mobiel;
- essentiële inhoud, volgorde en responsieve afbreking;
- geen ongewenste horizontale overloop, ontbrekende beelden of afgesneden labels;
- de gevraagde klik-, hover-/focus-, scroll- en tijdgestuurde handelingen;
- zichtbare toetsenbordfocus, toegankelijke betekenis en relevante reduced-motion;
- onverwachte consolefouten en veilige werking van lokale routes.

Een passing build bewijst geen visuele overeenkomst; een mooi screenshot bewijst geen werking. Herstel concrete gaten of vermeld de exacte onbewezen aspecten.

Lever gewijzigde bestanden, werkelijk gedraaide controles, bruikbare screenshots/preview en resterende verschillen. Publiceer, deploy of wijzig externe bronnen alleen binnen de afzonderlijk gegeven autorisatie. Bewaar bron-/assetprovenance bij het implementatiebewijs.
