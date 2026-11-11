# Mailconcepten in Event Note v1.1

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
