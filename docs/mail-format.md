# Mailconcepten in Event Note v1.1

## Herkomst en groepering afdwingen

Nieuwe Event Notes en expliciet omgezette notes gebruiken frontmatter
`activity_contract: provenance-v1`. `vault-layout --strict` weigert daarin iedere
menselijke of agentbubble zonder direct volgend stempel
`HH:mm · werkelijke auteur · kanaal` (inlinecode). De naam moet exact overeenkomen
met de bubbleauteur; uur en minuut moeten geldig zijn. Bij onbekende historische
tijd gebruik je `—`, nooit een geschatte minuut. De Activity-dagscheiding is de
oorspronkelijke lokale dag, oplopend; de schrijver controleert de tijdzone en het
bronbericht. Bestandstijden en antwoordtijden zijn geen bewijs van verzendtijd.

Elke agentbubble krijgt precies één directe marker:
`<!-- viggobsidian:activity:v1 kind=result goal=follow-up -->`.
Gebruik een stabiel doel met kleine letters, cijfers en koppeltekens. Herhaalde
resultaatbubbles voor hetzelfde doel worden afgekeurd: voeg procesuitleg bij het
resultaat, met oorspronkelijke momenten herkenbaar in de historie onder Meer.
Een losse `kind=process`-bubble is ongeldig. Menselijke beurten blijven volledig
in hun eigen human-bubble; ze hebben geen agentdoelmarker nodig. Volledige,
gevalideerde maildeliveries mogen hetzelfde doel herhalen met stijgende versies.
De bestaande maildeliverycontrole blijft daarnaast actief.

Dit contract is expliciete adoptie, geen bulkconversie van oude notes. Onbekende
contractversies falen gesloten. De checker bewijst aanwezigheid, syntaxis en
doelbinding, geen waar auteurschap, juiste historische minuut of semantisch goed
gekozen doelen. Beoordeel die tegen de primaire bron. Nooit oorspronkelijke
input verwijderen of doel-ID's verzinnen om een weigering te omzeilen.

## Menselijke input en opleverversies

Letterlijke menselijke input krijgt een eigen `activity-message|human`-bubble,
met de werkelijke menselijke auteur. De identiteit van de schrijvende tool
verandert het auteurschap niet. Agentinterpretatie staat in een agentbubble.
Gebruik bij nieuw vastgelegde letterlijke input de marker
`<!-- viggobsidian:input:v1 author=human -->` binnen die human-bubble; de checker
kan de gemarkeerde rol toetsen, maar niet bewijzen wie een zin heeft bedacht.

Iedere nieuwe of herziene oplevering krijgt een nieuwe bubble onderaan met het
volledige bruikbare resultaat. Gebruik bij mail een stabiel doel-ID en stijgend
versienummer: `<!-- viggobsidian:mail-delivery:v1 id=follow-up version=1 -->` en
zichtbaar `**Mailconcept v1**`. Houd hetzelfde ID voor correcties van dezelfde mail;
v2/v3 volgen onderaan. Verander v1/v2 niet achteraf om de nieuwste versie te leveren.
Een losse melding “aangepast” onderaan vervangt nooit een volledige nieuwe versie.
Beperk procesbubbles: groepeer toelichting en bronbinding bij hetzelfde resultaat,
met lange onderbouwing onder Details. Meng geen auteurs of oplevermomenten.
Een volledig gevalideerde maildelivery blijft direct zichtbaar, ook boven de
gewone lengtegrens. Historische versies blijven behouden; de checker herschrijft
of vouwt ze niet automatisch. Syntax bewijst geen inhoudelijke volledigheid.

## Kopieerbare velden

Nieuwe en inhoudelijk gewijzigde mailconcepten gebruiken het gedeelde sjabloon
`.viggobsidian/core/mail-draft.md`. Dit contract geldt per concept binnen één
gedateerde, benoemde Activity-bubble. Het verplaatst geen historische mails.
Gebruik bij lange inhoud de bestaande gevouwen `Meer`-callout; marker, velden en
bronbinding staan op dezelfde quotediepte binnen die callout.

To, Onderwerp en Body krijgen afzonderlijke `text`-codeblokken met de labels
**To**, **Onderwerp**, **Body** erbuiten. Alleen gecontroleerde e-mailadressen
staan in To, gescheiden door `; `. Cc en Bcc zijn optioneel, elk een eigen
gelabeld blok tussen To en Onderwerp. Behoud de aangetroffen rol; bedenk geen Bcc.
Het onderwerp is één regel zonder `Onderwerp:`. De body bevat aanhef, boodschap
en afsluiting als platte tekst. Geen wikilinks, Markdown-opmaak of statusregels
in kopieerblokken; echte URLs blijven staan. Gebruik een langere fence als de
tekst zelf backticks bevat. Quote ook lege regels en fences.

Controleer ieder adres inhoudelijk tegen de bestaande contact- of mailbron.
Zet `Bron: [[exacte-bron#passage]]` buiten de kopieerblokken, binnen dezelfde
bubble; benoem daar welke bron welk adres en welke ontvangersrol onderbouwt.
De checker controleert bronbindingssyntax, niet de waarheid van de identiteit.
Een aparte persoonlijke opvolgmail krijgt een eigen gelabelde bubble/concept,
met eigen bronbinding en ontvanger; voeg die persoon niet stil aan groeps-To toe.

Gebruik `recipients=resolved` alleen als alle bedoelde ontvangers gecontroleerd
zijn. Anders `recipients=unresolved` met een zichtbare `Ontbreekt: ...`-regel
buiten de blokken. Laat To weg als geen adres bevestigd is; gebruik geen naam,
voorbeeldadres of placeholder. Zijn enkele adressen bekend, neem alleen die op
en beschrijf wie ontbreekt. Noem zo'n concept nooit verzendklaar.

`Mailstatus: concept` beschrijft uitsluitend het concept, zonder goedkeuring of
verzending te claimen. Werkelijke bevestiging of verzending wordt als een aparte
gedateerde activiteit met echte bron vastgelegd. Opmaak wijzigt nooit taak-ID,
taakstatus, actiedatum, registratie of eigenaar en verstuurt niets.

De expliciete commentmarkers activeren `strict-mail-draft` in de normale strikte
Rust-layoutcheck. Meerdere concepten in één note worden apart gecontroleerd.
Ongemarkeerde historie blijft buiten deze nieuwe controle; bestaande Event Note-
en navigatiecontroles blijven gelden. Het sjabloon gebruikt uitsluitend fictieve
gegevens; vervang deze nooit zonder bewijs door echte adressen.
