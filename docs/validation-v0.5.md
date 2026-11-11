# Validatie v0.5.0

Issue #9: de CSS-compiler verving canonical `human` door een lokale alias.
Daardoor kregen correct opgeslagen menselijke bubbles de standaard agentstijl.
De compiler behoudt nu beide selectors, inclusief de lokale avatarregels en
negatieve selectors. Assistent-/systeemrollen zijn geen geldige menselijke alias.
Auteursmetadata en historische inhoud worden niet herschreven.

De gedeelde instructies onderscheiden letterlijke menselijke input van
agentinterpretatie en leveren een herziene mail als volledige nieuwe versie
onderaan. Samenhangende uitleg blijft bij die versie; procesmeldingen vervangen
geen resultaat. Een opt-in contract toetst expliciete inputrollen en strikt
stijgende mailversies per stabiel doel-ID. Het hergebruikt de mailveldvalidator.
Alleen gevalideerde kopieerblokken krijgen de lengte-uitzondering; lange uitleg
erbuiten blijft onder de normale grens. Broninhoud met quotes/backticks en
historische codevoorbeelden activeren de nieuwe markers niet.

Lokaal: 89 locked Rust-tests, fmt en Clippy geslaagd. Positieve en negatieve
regressies dekken aliases, reserved roles, versies, ontbrekende velden, lange
mailbody tegenover lange buitenproza en letterlijke markers in codefences.
Onafhankelijke review keurde vrijgave goed na herstel van twee parsergevallen.

Native bewijs uit de bestaande Obsidian-testvault: een tijdelijke, losstaande
WorkspaceLeaf rendert dezelfde fictieve v1 → menselijke correctie → v2 → correctie
→ v3 in Reading View én echte Live Preview. Auteurslabel, user-/boticoon,
links/rechtsuitlijning en drie volledige kopieervelden zijn gecontroleerd. De
werkelijke lokale appearanceconfiguratie toont dezelfde menselijke avatar voor
canonical human en de geconfigureerde alias. De actieve leaf blijft ongewijzigd;
de tijdelijke view en stijlen worden na de proef verwijderd. Geen authentieke
mailinhoud of persoonlijke appearanceconfiguratie staat in dit publieke bewijs.

Syntax bewijst geen waar auteurschap, juiste ontvangers of inhoudelijke
volledigheid. De schrijver leest en beoordeelt de bron; nieuwe instructies en
fixtures ondersteunen die beoordeling. Er is geen historische bulkconversie,
taakstatusmutatie, verzending of GitHub Actions toegevoegd.
