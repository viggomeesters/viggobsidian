# Mailconcepten in Event Note v1.1

## Herkomst en groepering afdwingen

Nieuwe Event Notes en expliciet omgezette notes gebruiken frontmatter
`activity_contract: provenance-v1`. Behoud de auteurskop voor identiteit en
toegankelijkheid; de weergave verbergt haar visueel. Zet herkomst eenmaal onderaan:

```markdown
> [!activity-message|assistant] Assistant
> <!-- viggobsidian:activity:v1 kind=result goal=follow-up -->
> Het resultaat.
>
> *09:42 · Assistant · draft*
```

De canonical footer is een aparte laatste cursieve alinea van de buitenste
bubble: `HH:mm · werkelijke auteur · kanaal`, met een lege geciteerde regel ervoor.
Zet haar na en buiten geneste Details, zodat open- en dichtklappen haar niet
verplaatst. Gebruik gewone tekst in de velden, geen HTML of Markdown.
De weergave toont haar kleiner, gedempt en rechts uitgelijnd, zonder cursief,
codechip of zichtbare auteurskop. Gewone cursieve brontekst blijft behouden.
Gebruik geen extra auteurhandtekening. `vault-layout --strict` weigert ongeldige
of dubbele herkomst. Uur en minuut moeten geldig zijn. Bij onbekende historische
tijd gebruik je `—`, nooit een geschatte minuut. De Activity-dagscheiding is de
oorspronkelijke lokale dag, oplopend; de schrijver controleert de tijdzone en het
bronbericht. Bestandstijden en antwoordtijden zijn geen bewijs van verzendtijd.

Compatibiliteit: bestaande provenance-v1-bubbles met de direct volgende inlinecode
`HH:mm · werkelijke auteur · kanaal` blijven geldig, evenals de #32-auteurskop
`Auteur <span class="activity-meta">HH:mm · kanaal</span>`; de auteur moet exact
met de auteurskop overeenkomen. Eén bubble gebruikt precies één vorm. Het nieuwe
footerdoel maakt oude bronbytes niet automatisch ongeldig en activeert geen bulkconversie.
Expliciete omzetting bewaart bewezen tijd, kanaal, auteur, input en historie;
wijzig uitsluitend de herkomstpresentatie en behoud preimage/herstelbewijs.
Een gerichte omzetting leest de actuele volledige note opnieuw, bindt de exacte
preimage en wijzigt alleen expliciet gekozen bubbles. Verwijder uitsluitend de
bewezen topstempel of de exacte #32-span; behoud de auteurskop. Voeg dezelfde
drie velden als afzonderlijke laatste cursieve alinea toe buiten Details.
Vergelijk na afloop oorspronkelijke inhoud, bronlinks, markers, taak-/bron-ID's,
mailvelden en alle versies bytegewijs buiten deze herkomstwijziging. Weiger drift,
dubbele of ambigue herkomst; verzin geen tijd/kanaal en verander geen importtijd
in berichttijd. Reeds bestaande oude eindfooters kunnen historische kanaal-/
handtekeningvormen bevatten: behoud ze tot expliciete brongebonden adoptie;
zonder `activity_contract` ontstaat geen nieuwe verplichte herkomstmigratie.

De broncheckout heeft hiervoor een begrensde read-only previewtool:
`cargo run --locked --bin activity-footer-preview -- SOURCE EXPECTED_SHA256 HEADING_LINE[,HEADING_LINE...]`.
De geselecteerde oorspronkelijke, 1-gebaseerde regel(s) moeten levende buitenste
Activity-koppen zijn. Alleen LF/UTF-8 en exact ondersteunde herkomstvormen worden
omgezet. De tool weigert drift, ongeldige velden en extra herkomstachtige tekst;
zij schrijft uitsluitend de volledige kandidaat naar stdout. Controleer de diff
en gebruik daarna de gekozen vaults geautoriseerde writer met dezelfde preimage;
redirect nooit stdout over het bronbestand. Geen automatische adoptie, bulkmutatie
of wijziging van taakstatus. Bewaar de originele bytes voor herstel.

Voor een afzonderlijk geautoriseerde migratie van oude Activity-opmaak bestaat
ook een read-only preview:

```sh
cargo run --locked --bin activity-history-preview -- SOURCE EXPECTED_SHA256 < request.json
```

De request gebruikt `schema: viggobsidian.activity-history-migration.v1`, de
1-gebaseerde regel `activity_line` van de enige oorspronkelijke buitenste
`## Activity`, en de werkelijk uitgevoerde opmaakactie als `date` (`YYYY-MM-DD`),
`time`, `author` en `channel`. Deze velden dateren de nieuwe opmaakbijdrage,
nooit de historische bron. De helper stelt hun waarheid, eigenaarschap en
schrijfautorisatie niet vast en schrijft uitsluitend kandidaattekst naar stdout.
Redirect die uitvoer nooit over de bron.

De bestaande YAML en header blijven bytegelijk, met uitsluitend de nieuwe
`activity_contract: provenance-v1`-regel. Ongeldige/dubbele YAML, merge keys of
een reeds bestaand Activity-contract vereisen afzonderlijke beoordeling. De
volledige oude Activity-suffix blijft reversibel in één historische bubble;
oorspronkelijke auteurs, tijden, taakstatus, IDs en brontekst worden niet geraden
of vervangen. Headerregels gebruiken LF; historische CRLF en een ontbrekende
slotnewline blijven behouden. De archiefmarker bewaart `final-newline` en de
`literal-width` van toegevoegde inlinecode. Alleen oorspronkelijke controlcomment-
regels worden letterlijk gecodeerd, zodat zij geen nieuwe agent-/mailactie
activeren. Strip de toegevoegde quote-prefixes en die afgebakende inlinecode;
verwijder alleen bij `final-newline=0` de extra laatste LF voor bytevergelijking.

`processing_lines: [FIRST, LAST]` kiest desgewenst een volledige oorspronkelijke
verwerkingssamenvatting met haar echte Wiki-link, zonder auteurskop of footer.
De huidige presentatie noemt deze verwerking expliciet historisch; zij claimt
geen herbeoordeling. Alle overige verwerkingstekst blijft in de volledige bron.

Geciteerde koppen zijn geen betrouwbare native Obsidian-koplocators. Bewaar
benodigde navigatie daarom expliciet met `passages`, maximaal 128 per note:
`[{"first_line": FIRST, "last_line": LAST, "block_id": "historical-example"}]`.
Iedere range begint bij één unieke, niet-geciteerde oorspronkelijke ATX-kop en
eindigt vóór de volgende kop van gelijk of hoger niveau, inclusief alle kinderen.
Zij krijgt een afzonderlijke huidige navigatiekopie met haar blok-ID vóór de
echte footer. Kopsyntax wordt uitsluitend in die afgeleide kopie gewone vetgedrukte
tekst; de volledige oorspronkelijke koppen blijven in de bronhistorie staan.
Overlappende parent-/childkopieën bewaren ieder hun volledige bronbereik.

De lokale writer bezit daarnaast de exacte inbound-locatorcorrecties, volledige
preimages, bron-/configuratiepins, write-set, CAS en herstel. Controleer de echte
Obsidian-blokcache, normale navigatie en embeds; de Rust-kopparser alleen bewijst
geen native Obsidian-navigatie. De preview verleent geen rechten voor een nieuwe
writer, bronoverdracht, registratie, taakafhandeling of vaultbrede write. Bron en
volledige kandidaat zijn elk begrensd op 1 MiB; expansie wordt vóór encoding en
iedere passagekopie gecontroleerd. De gewone strikte checks blijven ongewijzigd.

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
