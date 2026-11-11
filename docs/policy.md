# Vault Navigation

ViggObsidian beheert gedeeld beleid, schema’s, CSS en vaultskills. Een geïnstalleerd bestand is een versiegebonden distributiekopie. Wijzig de bron in deze repository via Issue → PR → review → merge → release; pas de release daarna bewust op iedere vault toe.

## Eén profiel

`numbered-v1` gebruikt `00-inbox`, `10-daily`, `20-projects`, programmaroots `21-naam` tot en met `29-naam`, `30-topics`, `90-templates` en `99-system`. `20-projects` bezit losse projecten. Een programma groepeert samenhangende projecten en krijgt geen vaste productgebonden naam. Een project staat in precies één programma of bij de losse projecten; verwijs elders naar dat project.

Projectmappen heten `YYYY-MM-projectnaam`, op basis van de startmaand. Onderdelen heten `code-NN-onderdeel`, met dezelfde unieke projectcode in kleine letters, zoals `sdp-01-objectmodel`. De zichtbare code en metadata mogen `SDP-01` tonen. Het project-00 heet `code-00-project-wiki`. Daar staan overzicht, besluiten, deelnemers en rollen. Ieder onderdeel heeft een onderhouden `wiki/` en oorspronkelijke bronnen in `raw/`. Onderdeelnummers beschrijven de inhoudelijke volgorde. Het 00-onderdeel houdt eveneens Wiki en Raw. Bestaande identiteitsvelden zoals `component-id` en `component_id` blijven behouden; een spellingwijziging maakt geen nieuw object.

Topics zijn blijvende onderwerpen zonder begrensd projectresultaat. Een architectuuronderzoek, privacytraject of overstap met een resultaat en eindpunt is een project. Maak geen centrale personen-, rollen- of takenverzameling als neveneffect van dit profiel. Taken blijven bij hun bestaande inhoudelijke eigenaar en behouden hun identiteit, status en datum.

## Wiki en Raw

Lees de actuele eigenaar en de volledige relevante bron vóór een wijziging. Raw bewaart oorspronkelijke input, bijlagen, herkomst, registratie en bestaande taak-ID’s. Wiki beschrijft de onderhouden kennis en verwijst bij de inhoudelijke uitspraak naar de echte bron en passage. Maak onderscheid tussen feit, voorstel, besluit en uitgevoerde actie. Markeer een bron pas als verwerkt nadat de inhoud inhoudelijk is beoordeeld; vermeld waarom een bron niets toevoegt. Een groene validator bewijst geen inhoudelijke verwerking.

Foldernotes, context, navigatie en bron-/activiteitweergave volgen de gedeelde CSS en `vault-layout`-engine. Lokale schema’s bezitten exacte scopes en delegaties; lokale writers behouden hun authority. Broninventarissen, volledige v1.1-Activity-bubbels, Base-contracten, projectcodes en linknavigatie worden uit dezelfde Rust-bron gecontroleerd. Een strengere samengevoegde controle mag bestaande inhoudsafwijkingen zichtbaar maken; schakel geen controles uit om een migratie groen te laten lijken.

## Daily

Daily’s staan onder `10-daily/notes/YYYY-MM-DD.md`. Toon automatisch de notes die op die dag zijn aangemaakt via een embedded Base met `file.ctime.date() == date(this.file.name)`, sluit de daily zelf uit en sorteer `file.ctime` aflopend. De notenaam bepaalt de dag. Bewaar menselijke daginhoud en bestaande links; voeg voor agentoutput geen handmatige linkrijen toe als automatische dekking volstaat. Een kopie/import kan een nieuwe filesystem-aanmaakdatum krijgen; dit overzicht is geen bronhistorie.

## Bronautoriteit en lokale adapters

Het pakket bezit geen persoonsgegevens, vaultinhoud, registratiedatabase, credentials, queues, cloud-/mailconfig of runtimeauthority. De expliciet gekozen vault bezit haar lokale adapter en domeinregels. Een skillbron is nooit de doelvault. Lees de doelroot `AGENTS.md` en `.viggobsidian/local-adapter.md`; bind iedere reader, writer en controle aan diezelfde root. Bij een ontbrekende vereiste adapter stopt alleen de afhankelijke mutatie. Gebruik geen directe write of andere vault als fallback bij providerweigering.

Legacy projectroots worden als expliciete lokale adoptieschuld geregistreerd. Ze veranderen het profiel niet en een pakketupdate verhuist geen inhoud. Een structurele migratie vergt een eigen geautoriseerde opdracht met identiteits-, link- en herstelcontrole.

## Updates

Preview vóór apply, met exacte pakketversie, manifesthash en per-bestand preimage. Alleen manifestbestanden binnen het beheerde oppervlak mogen veranderen. Een al bestaand onbeheerd bestand vereist een expliciet gecontroleerde adoptiehash. Lokale afwijkingen zijn conflicten. De updater bewaart voor iedere write de oude bytes en een herstelreceipt. Rollback weigert gewijzigde postimages. Er is geen automatische pull, scheduler of push.
