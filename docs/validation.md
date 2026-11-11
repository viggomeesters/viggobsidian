# Validatie v0.1.0

Lokaal gecontroleerd op 11 november 2026:

- Rust: 16 regressietests; fmt en Clippy met warnings als fouten.
- Onafhankelijke review van updater, herstel, doelrootselectie, skills, lokale providerbruggen en publieke inhoud. Gevonden Unicode- en hervattingsfouten zijn gerepareerd en met regressies afgedekt.
- Fixtures: nieuwe installatie, identieke herhaling, update en rollback, expliciete adoptie, lokale drift, verouderde/tampered previews, bronhashwijziging, padtraversal, symlinks, locks, gedeeltelijke apply, onderbroken meerbestandsrollback en behoud van taak-/bronbytes.
- Twee afzonderlijke live-installaties: hetzelfde pakket, expliciete lokale adapters en schone package-doctor. Herhaalde apply meldt `unchanged`. Vier bestaande Base-assets worden zonder bytewijziging als releasebeheerd geregistreerd.
- Bestaande lokale skillbundler: 24 tests. Lokale quality-routing: 11 tests, inclusief positieve en negatieve release-adapterbinding.

Dit bewijst geen volledige profielmigratie van bestaande inhoud. De lokale structuurcontrole blijft bestaande legacy projectroots en samengestelde componentnummering melden. Bestaande diepe layout-/bron-/providercontroles blijven verplicht en rapporteren hun eigen al aanwezige bevindingen. Andere persoonlijke skilldrift valt buiten de zeven overgenomen ingangen en wordt niet door vernieuwde hashes verborgen.

De publicatiescan controleerde de nieuwe productbestanden en volledige nieuwe Git-historie; er is geen vaultgeschiedenis geïmporteerd. De publiek bedoelde GitHub-identiteit blijft zichtbaar. Er zijn geen GitHub Actions, achtergrondservices of automatische vaultupdates toegevoegd.
