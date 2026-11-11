# Validatie v0.3.0

Issues #3 en #5 worden samen opgelost met het laatste besluit: afzonderlijke
kopieerblokken voor To, Onderwerp en Body binnen dezelfde auteursbubble.

De Rust-check is expliciet per concept geactiveerd; ongemarkeerde historie en
ontvangen mail worden niet geconverteerd. Tests dekken rolbehoud van Cc/Bcc,
onbekende ontvangers, bronbindingssyntax, conceptstatus, langere en ingesprongen
sluitfences, letterlijke replyquotes en historische codevoorbeelden. De CLI-test
controleert dat foutieve concepten worden gemeld zonder de bron te wijzigen.

Workspace: 53 engine-unittests, 11 integratietests en 19 updater-/CSS-tests.
Locked tests, fmt en Clippy met warnings als fouten slagen. De onafhankelijke
review bracht renderer-fenceverschillen aan het licht; deze zijn hersteld en
afgedekt met regressietests.

Native Obsidian in beide referentievaults: het fictieve sjabloon wordt in een
verborgen, tijdelijk DOM-element gerenderd; de echte drie kopieerknoppen leveren
alleen respectievelijk adressen, onderwerp en body. Clipboardschrijven wordt
tijdens deze test onderschept en daarna hersteld. Geen vensterfocus, bestaande
note of echt klembord wordt gewijzigd. Bron en conceptstatus blijven zichtbaar
binnen de auteursbubble en buiten de kopieerblokken. Copygate zonder bevindingen.

De checker kan niet bewijzen dat een adres bij de bedoelde persoon hoort;
inhoudelijke broncontrole blijft verplicht. Het voorbeeld bevat uitsluitend
fictieve gegevens. Er is geen mailverzending, taakstatusmutatie, historische
bulkconversie of GitHub Actions-configuratie toegevoegd.
