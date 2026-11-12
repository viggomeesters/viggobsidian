# Validatie v0.6.0

Issue 12 maakt tijdstempels en doelgerichte groepering expliciet afdwingbaar.
`activity_contract: provenance-v1` is een bewuste note-adoptie: de strikte
Rust-checker vereist een geldige lokale Activity-dag en direct na iedere
auteurskop een inlinecode-stempel met tijd, exact dezelfde auteur en kanaal.
Onbekende historische tijd is `—`. Lege of ongeldige opgegeven contractversies,
ontbrekende of ongeldige tijden, auteurs of doelbindingen geven een niet-nul exitcode.

Agentresultaten gebruiken één stabiel doel. Herhaalde gewone resultaatbubbles
voor hetzelfde doel en losse procesbubbles worden geweigerd. Volledige,
gevalideerde maildeliveries mogen een doel herhalen met stijgende versies en
hetzelfde delivery-ID. Dit contract classificeert geen vrije tekst op trefwoorden
en migreert oude ongemarkeerde notes niet. De schrijver beoordeelt doelkeuze,
bronwaarheid, tijdzone en behoud van menselijke input.

Positieve en negatieve unit-/CLI-proeven dekken bekende/onbekende tijden,
middernacht met nieuwe lokale dag, verkeerde auteur, ontbrekend kanaal, ontbrekend
stempel, verkeerde positie van het stempel, onbekend contract, procesbubbles,
herhaalde doelen, volledige mailversies en ongemarkeerde historie. Onafhankelijke
review vond twee parsergaten: een geciteerd onafgesloten codeblok kon een volgende
bubble verbergen, en een codevoorbeeld kon de directe stempelpositie omzeilen.
Beide zijn hersteld en als negatieve regressies toegevoegd.

Alle 93 locked workspace-tests, rustfmt, Clippy en cargo-audit slagen. De lockfile
verandert uitsluitend de eigen pakketversie; er zijn geen nieuwe dependencies.
De technische Rust-diffgate is uitgevoerd vanaf de oorspronkelijke integratiebasis.

De fictieve bronfixture is in de bestaande Obsidian-testvault gecontroleerd met
losstaande Reading View- en Live Preview-leaves. De actieve leaf blijft behouden.
De noodzakelijke inline JavaScript is uitsluitend de native Obsidian-API voor deze
renderproef; de productimplementatie en gedragstests zijn Rust. Er staat geen
authentieke vaultinhoud, persoonlijke configuratie of brontranscript in dit bewijs.

De aangewezen echte mailnote is expliciet aangesloten op provenance-v1, met
behouden menselijke input, mailvelden en bijlagen. De strikte controle slaagt.
Een geïsoleerde kopie met uitsluitend aangepaste Finder-locator slaagt eveneens;
verwijdering van één menselijk tijdstempel levert exact de verwachte
strict-activity-provenance-bevinding en exitcode 1. De live note is nooit voor
de negatieve proef beschadigd. Syntax bewijst geen juiste historische minuut.
