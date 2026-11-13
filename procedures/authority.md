# Doel, bron en provider

Selecteer de doelvault uit de expliciete opdracht of de reeds bewezen sessiecontext.
Lees daar AGENTS.md, `.viggobsidian/core/policy.md`, de lokale adapter en het
gekozen dossier. De bronlocatie van een skill, geïnstalleerd script of repository
selecteert nooit de doelvault. Geef iedere ondersteunde helper zijn expliciete
`--vault` of `--root`; een implementatiepad mag alleen imports vinden.

Lees vóór de gekozen route `.viggobsidian/core/procedures.json` en
`.viggobsidian/providers.json`. De read-only controle is
`vault-layout --vault ROOT --skill-route vault --skill-capability EFFECT`. Zij toont de gedeelde procedure,
benodigde referenties en lokale capability. `--skill-routes` controleert de hele
dekking. Een beschikbare route is geen uitvoerautorisatie. Ontbrekende, ongeldige
of unavailable providers blokkeren alleen hun afhankelijke effect. Installeer
geen vervangende writer en leen geen provider uit een andere vault.

De lokale adapter bezit precieze commando's, domeinregels, locks, claims,
bronregistratie, schemas, limieten, hostregels en herstel. Een guided provider
betekent de reeds toegestane begeleide lokale procedure, geen nieuwe generieke
bestandswriter. Een legacy provider vereist expliciete legacy-intentie en haar
eigen authority; de Markdownprocedure activeert geen JSONL-runtime.

## Eigenaarschap en bronidentiteit

Zoek het bestaande dossier en dezelfde bron vóór aanmaak. Lees de volledige
actuele eigenaar, relevante primaire bronnen en eventuele gebonden work_dossier.
Een naam of fysieke map is geen bewezen identiteit. Behoud taak-/bron-ID,
oorspronkelijke woorden, auteurschap, kanaal, beschikbare brontijd en bijlagen.
Markeer ontbrekende tijd of broninhoud als onbekend. Zoektreffers, archieven,
scenario's en oude agentoutput zijn bewijs, geen actuele opdracht of besluit.

Scheid bron, kennis, actie en presentatie. Raw bewaart oorspronkelijke input;
Wiki onderhoudt de actuele betekenis met precieze bronverwijzingen. Nieuwe
projectacties horen bij de gekozen onderdeel-Raw, met bestaande identiteit en
de lokale canonieke intake. Centrale oude collecties en persoonlijke siblings
blijven alleen compatibiliteit voor bestaande callers. Geen impliciete migratie.

## Effecten en autorisatie

Lezen schrijft niets. Loggen bewaart input en eventueel gevraagd voorwerk.
Verwerken onderhoudt kennis en bronplaatsing, zonder genoemde acties vanzelf uit
te voeren. Oppakken vervolgt een werkelijk gekozen taak. Een gecombineerd
expliciet verzoek mag deze effecten samen autoriseren; vraag niet nogmaals om
reeds opgedragen uitvoering. Een genoemd plan in een bron is geen autorisatie.
Verzending, publicatie, aankopen en runtimewijziging houden hun eigen grenzen.

## Transactie en readback

Bind de werkelijk gelezen bron-/targethashes, complete write-set, checker,
configuratie en adoptieconfiguratie volgens `core/write-validation.md` aan het
canonieke lokale plan. Inspecteer de delta vóór apply. Drift vereist herlezen en
een nieuw plan; wijzig nooit pins, guards, receipts of journals om te passeren.
Gebruik dezelfde claims en sessie, vernieuw en controleer vóór writes. Respecteer
locks, exclusieve publicatie, complete bijlagen en guarded rollback/herstel.

Lees na schrijven alle getroffen actuele notes terug, controleer oorspronkelijke
input, betekenis, links, bronbestanden en eigen diff. Een groene schema/hashcheck
bewijst geen semantische verwerking, sync, app-rendering of externe aflevering.
Volg de doelvaults hostcommitregel en behoud vreemde edits/staging. Geen GitHub
Actions, scheduler of automatische publicatie door een skillinstallatie.

## Dagnotities en weergave

Dag-/weekinhoud blijft menselijk eigendom. Alleen expliciet geautoriseerde
bewerkingen volgen de lokale dagelijkse lock en actuele readback. Maak geen
daily voor een opleverlink. Automatische dagelijkse Bases tonen de op die datum
aangemaakte notes; standaard geen handmatige linkrijen of HTML. Antwoorden komen
in chat, tenzij een bestand of HTML zelf gevraagd is. Nieuwe Event Note-bijdragen
volgen het gedeelde auteurschap-/herkomstcontract; oude bronbytes worden niet als
bijeffect herschreven. Relevante private domeinregels blijven lokaal.
