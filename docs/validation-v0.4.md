# Validatie v0.4.0

Issue #8 centraliseert de gedeelde navigatiestandaard en het generieke
machinecontract. Zes oude documentingangen behouden hun kopankers en verwijzen
naar één canonieke handleiding. Dezelfde releasebestanden worden op bestaande
paden geïnstalleerd; lokale providers, scope, identity en legacyinterfaces blijven
lokaal. De oude centrale taak-/meetingindeling is geen nieuwe default.

Lokaal gecontroleerd:

- 85 locked Rust-tests: 54 engine-unittests, 12 integratietests en 19 updater-/CSS-tests.
- Fmt en Clippy met warnings als fouten slagen.
- Echte CLI-integratie bewijst dat de engine generieke metadata uit het gedeelde
  contract leest en in een geadopteerde installatie een ontbrekend contract weigert.
- Contracttests bewaren lokale uitzonderingen en taakstatussen; ongeldige schema's
  en symlinkcontracten worden geweigerd.
- Onafhankelijke architectuur-, privacy- en herreview uitgevoerd. De v1.1-detailregel
  is rechtgezet en de lokale workflowdefault is via brongebonden writes bijgewerkt.
- De twee geïnstalleerde engines bouwen en testen zelfstandig; package-doctors
  zijn foutvrij en beide System-scopes zijn strict groen. Gewijzigde ingangen en
  kopankers zijn gecontroleerd. De Work-linkstijlcontrole heeft nul bevindingen.
- Achtergrondrender in de bestaande Obsidian-testvault: correcte titel, 23 koppen,
  tabel en codevoorbeelden; geen renderfout. Geen note- of focuswijziging.

De lokale systeemwriter behield zijn eigen beheerbron. Begrensde onderhoudstests
bewaken inlinecodeproza tegenover codefences, de exact gebonden workflowsectie
tegenover oorspronkelijke bronsecties, en stale-planweigering na contractdrift.
Historische formuleringen blijven brongebonden bewaard. Een onafhankelijk
baselinevergelijk toont dat een andere bestaande mixed-source-test al vóór deze
wijziging buiten-scope-writes weigert; deze release verruimt die grens niet.

De volledige vaultchecks blijven bestaande inhoudsschuld melden. Deze release
claimt geen volledige inhoudsmigratie of groene historische vault. Exacte lokale
bevindingen, bytepreimages, receipts en voor/na-uitvoer blijven bij de installatie.
Legacy folder-/roadmapcontracten blijven compatibiliteit voor bestaande callers;
ze worden niet als persoonlijke inhoud naar de publieke bron gekopieerd en hun
historische herstelpins worden niet stilzwijgend vernieuwd.
