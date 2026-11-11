# Validatie v0.2.0

De volledige Rust-engine en CSS zijn uit de twee bestaande varianten samengevoegd. De strengere Activity-, source-inventory- en quoted-codefencecontroles zijn behouden, samen met projectcodes, intake, linkstijl en Work-packagecontrole. Eén core gebruikt lokale scopes en providercontracten; er zijn geen aparte productprofielen of lokaal onderhouden codeforks.

Lokaal gecontroleerd op 11 november 2026:

- 46 engine-unittests, 11 engine-integratietests en 19 updater-/CSS-tests: alle 76 geslaagd. De integratiesuite behoudt beide oorspronkelijke testsets.
- Workspace fmt en Clippy met warnings als fouten geslaagd.
- Beide reguliere vaultinstallaties zelfstandig gebouwd en getest met hun eigen Cargo.lock; bestaande launchers blijven werken.
- Update-/derivatietests bewaken de shared CSS-bron, lokale configuratiehash, stale-planweigering, idempotentie en rollback zonder lokale configuratie te overschrijven.
- De bestaande skillbundler heeft 25 geslaagde tests. De lokale writerbinding weigert een oud plan na scope-, binary- of appearancewijziging; de gerichte bestaande integratietest slaagt.
- Native Obsidian-rendering op 800 en 1.200 pixels: 100 gerenderde links plus relevante callout-/kop-/tabelelementen vergeleken; geen stijl- of tekstverschil. De oorspronkelijke metadata-CSS-klassen zijn expliciet in de achtergrondrender toegepast. Copygate zonder bevindingen; er is geen nieuwe interfacetekst toegevoegd.
- Onafhankelijke code-/privacyreview uitgevoerd. Een gemelde combinatie van full-check en mutatorflags is gerepareerd en heeft een regressietest. Persoonlijke avatar, identiteit, bestemmingen en externe dossierafbeeldingen blijven in lokale configuratie.

De volledige strikte checks zijn uitgevoerd en zijn geen algemene groenclaim voor historische inhoud. Op dezelfde invoer houdt de gedeelde engine in de ene referentie-installatie exact de bestaande bevindingen, zonder toegevoegde of verdwenen layoutmeldingen. De andere installatie krijgt nieuwe v1.1-bevindingen omdat die controle daar voorheen ontbrak. Werkelijke aantallen, bronpaden en lees-/herstelcontext staan uitsluitend in de lokale vaults. Er zijn geen inhoudsrecords gewijzigd om de check groen te maken.

De installatie behoudt expliciete herstelreceipts. Alle gedeelde bronnen worden regulier geïnstalleerd; er zijn geen symlinks naar een ontwikkelcheckout. Fixtures verwijzen in de repository naar eigen canonieke assets en worden lokaal als reguliere bestanden geleverd. Het bestaande zelfstandige backupgereedschap blijft buiten deze publicatie. Er zijn geen services, automatische updates of GitHub Actions toegevoegd.
