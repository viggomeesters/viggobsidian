# Batches, bookmarks en imports

## Parallel voorbereiden, gecontroleerd schrijven

Paralleliseer alleen onafhankelijke bron-/dossieranalyse wanneer passend en
geautoriseerd. De hoofdagent bezit sessies, claimtokens, renew/check en alle
mutaties. Een leesworker krijgt begrensde paden, volledige relevante broncontext
en concrete acceptatie; geen writerbevoegdheid, tokens of Gitmutaties. Behoud
menselijke keuzes en eigenaargrenzen. Levensduur/actieve sessie en claimbereik
worden door de bestaande provider begrensd, niet door nieuwe globale state.

Iedere voorbereiding bevat exacte inventaris, gelezen bronversies/read_set,
doelowner, behouden bijlagen, inhoudelijke voorstellen en open beslissingen.
De hoofdagent leest de betekenis terug vóór ready-review en serialiseert de
bestaande writerrequests. Een technische hashreview vervangt geen semantische
controle. Verlopen/bezette claims, source-/targetdrift, incomplete bijlagen of
ontbrekende provider stoppen het betrokken pakket. Geen tokenloze fallback,
stille tweede sessie of nieuwe queue om een batch sneller groen te maken.

Voltooi onafhankelijke pakketten zodra hun bronreview en gates klaar zijn; geen
onnodige batchbarrière. Een verwerkingslimiet of claimbatchgrootte is geen
gesprekslimiet. Respecteer lokale budgetten, hervatjournals en coverage. Een
prepared payload is alleen bruikbaar bij gelijke bronnen, writer/config en
actuele inhoudelijke beoordeling. Unknown outcomes blijven herstelwerk.

## Bookmarks

Een enkele gekozen browserpagina bewaren en een leeslijst importeren zijn
verschillende effecten/providers. De eerste gebruikt de bestaande single-page-
capture; `importeer leeslijst` kiest expliciet de reading-list-adapter. Alleen
verwerken kiest bestaande bronpakketten en activeert geen van beide importers.
De browser blijft read-only waar de lokale importer dat contract bezit.

Een bookmark bewaart titel, exacte URL, oorspronkelijke bronidentiteit en
beschikbaarheid, geen verzonnen gelezen artikel. Open de bron bij inhoudelijke
verwerking en behoud de leesgrens bij paywall, login, verwijdering of fout.
Hergebruik bestaande bronpakketten en actuele dossierkennis; gewijzigde URL of
inhoud is een herkenbare revisie. Maak alleen een nieuwe werkruimte als daarvoor
echte inhoud en een gekozen eigenaar bestaan; geen lege scaffold per link.

Scheid broninhoud, door de mens gegeven context en agentinterpretatie. Beoordeel
relevantie en betekenis, actualiseer de juiste Wiki met precieze bronpassages en
maak alleen noodzakelijke geautoriseerde acties. Een bookmarkstatus is geen
bewijs van gelezen/uitgevoerd werk. Joint/batch-routes houden dezelfde claims,
complete pakketten, ready-review en semantische completiongrens als gewone bronnen.

## Bronspecifieke imports

iMessage-, leeslijst-, WhatsApp-, mail-, audio- en documentimports gebruiken
uitsluitend hun beschikbare lokale adapter op expliciete importintentie.
Bronselectie, systeemaccount, bereik, deltabinding en originele media blijven
provider-eigen. Kennis lezen activeert geen import. Een lege gekozen selectie
bewijst geen volledige accountgeschiedenis of lege algemene inbox.

Import bewaart originele bytes, bron-ID/revisie, deelnemers/afzenders en
dekkingsgaten. Screenshots dekken geen onzichtbare berichten en transcriptie
bewaart geen audio die niet ontvangen is. Vergelijk eerdere import vóór retry;
dezelfde berichten worden geen nieuw moment. Markeer archived/received/processed
alleen volgens hun verschillende echte providerbetekenis. Een import is geen
kennisactualisatie, bevestigde sociale interactie of uitgevoerde taak.
