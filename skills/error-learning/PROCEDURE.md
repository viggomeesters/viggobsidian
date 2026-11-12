---
name: error-learning
description: "Use when the user requests an error fix or a Hermes automation, provider, gateway, tool, delivery, operation card, or generated artifact fails or regresses. Search persistent incident evidence, debug from evidence and record the verified fix within the selected provider's scope."
version: 2.0.0
author: Hermes Agent
license: MIT
user-invocable: true
triggers:
  - "error fix"
  - "fix error"
  - "fix de error"
  - "fix deze error"
  - "error oplossen"
  - "error database"
  - "structurele fix"
  - "fix dit structureel"
  - "dit lukt steeds niet"
  - "provider failed"
  - "automation-fout"
  - "bijlage niet beschikbaar"
metadata:
  hermes:
    tags: [errors, incidents, debugging, selfimprove, regression, operations]
    related_skills: [systematic-debugging, selfimprove, agent-correction-loop, verification-line]
---

# Error Learning

## Doel

Gebruik één groeiende lokale incidentdatabase zodat dezelfde fout niet telkens
als nieuw raadsel wordt behandeld.

```text
waarneming → fingerprint → eerdere lessen zoeken → rootcause → regressietest
→ fix → live bewijs → verified resolve → preventieregel
```

De database is een technisch geheugen, geen statusdashboard. Een groene volgende
run sluit een incident niet vanzelf: `resolved` vereist bewijs en uitleg waarom
de fix terugkeer voorkomt.

## Wanneer laden

Laad deze skill bij:

- commandovarianten als `error fix`, `fix error`, `fix de error` en
  `error oplossen`;
- cron-/systemd-/gateway-/providerfouten;
- mislukte operation cards of hervattingen;
- `MEDIA:`-bijlagen die niet aankomen;
- terugkerende browser-, deploy- of artifactproblemen;
- verzoeken als “fix structureel”, “dit lukt steeds niet” of “selfimprove dit”.

Niet gebruiken voor normale userfouten, inhoudelijke meningsverschillen of één
verwachte validation rejection die het systeem correct afvangt.

Bij een geautoriseerde repositoryreparatie blijft `github-work` de publieke repository-ingang.
`github-work` moet deze skill dan **verplicht stapelen**: eerst eerdere incidentkennis
zoeken, daarna de repo-fix uitvoeren, en pas afsluiten nadat de waarneming en de
geverifieerde oplossing in deze centrale database staan wanneer de interface
beschikbaar is; bij uitval volstaat de expliciete unavailable-vermelding in
bestaand taakbewijs, zonder te claimen dat registratie of resolve is uitgevoerd.

## Opslag en privacy

Standaarddatabase van het actieve Hermes-profiel:

```text
$HERMES_HOME/state/error-knowledge/errors.sqlite3
```

Officiële interface, eigendom van Hermes core:

```bash
hermes errors --help
```

De skill bevat bewust **geen eigen database-engine of schrijfscript**. Alle reads
en writes lopen via `hermes errors`, dat het profielpad met Hermes'
`get_hermes_home()` bepaalt. Wanneer dit commando ontbreekt of onbereikbaar is: meld eenmaal
`incident_logging_unavailable` in de bestaande taakrapportage en ga door met
geautoriseerd repositorywerk. Logboekuitval blokkeert geen reparatie. Maak geen
projectlog, losse JSON-file of compatibiliteitsdatabase als schaduwwaarheid en
upgrade Hermes niet als neveneffect. Eigenaarschap, levende workers, scope en
publicatiebeperkingen blijven harde grenzen.

Eigenschappen:

- SQLite WAL, databasebestand mode `0600`, parentdirectory `0700`;
- stabiele SHA-256-fingerprint van component + genormaliseerde signatures;
- `incidents` bevat de actuele waarheid;
- `events` is append-only historie;
- bekende token/password/API-key/Bearer-patronen worden vóór opslag geredact;
- bewaar geen volledige prompts, documenten, e-mails of persoonlijke payloads;
- context bevat alleen minimale technische metadata en veilige artifactrefs.

## Verplichte workflow

### 1. Zoek vóór debuggen wanneer de interface beschikbaar is

```bash
hermes errors search "<component foutterm>" --text
```

Gebruik een eerdere fix alleen wanneer omgeving, failure-signature en
verificatiecontract nog overeenkomen. Een oude workaround is geen waarheid.

### 2. Registreer de waarneming wanneer de interface beschikbaar is

```bash
hermes errors record \
  --source '<cron|gateway|operation-card|telegram|manual>' \
  --category '<automation|provider|delivery|runtime|artifact>' \
  --component '<stabiele component-id>' \
  --signature '<stabiele foutklasse zonder timestamp/pid>' \
  --error '<korte veilige fouttekst>' \
  --context-json '{"job_id":"...","phase":"..."}'
```

Dezelfde fingerprint telt als nieuwe occurrence. Wanneer een resolved incident
terugkomt wordt het `regressed`, niet stilletjes opnieuw `open`.

### 3. Debug systematisch

Volg [debug](../debug/PROCEDURE.md):

1. lees volledige fout/logs;
2. reproduceer exact;
3. traceer producer → transport → consumer → readback;
4. formuleer één rootcausehypothese;
5. schrijf eerst een falende regressietest;
6. fix de oorzaak, niet alleen de zichtbare tekst.

Voor providerfouten zijn `retry werkte` en een correcte eindtekst onvoldoende.
Bewijs slotselectie, deadline, persistence/resume en operation-cardstatus.

Voor bestanden/media is `send success` onvoldoende. Bewijs:

- lokaal bestand bestaat en is leesbaar;
- transport koos de juiste native methode;
- platform gaf een remote message/document-id;
- indien mogelijk remote readback of zichtbare ontvangst.

### 4. Resolve alleen met compleet bewijs

```bash
hermes errors resolve \
  --fingerprint '<id>' \
  --root-cause '<waarom het gebeurde>' \
  --fix '<wat structureel gewijzigd is>' \
  --verification '<test + live/readback bewijs>' \
  --prevention '<validator/test/monitor/contract dat terugkeer vangt>' \
  --artifact 'commit:<sha>' \
  --artifact 'test:<naam>'
```

De CLI weigert incomplete resolves. Gebruik `note` of prose buiten de database
voor hypothesen; schrijf geen onbewezen rootcause als resolved waarheid.

### 5. Selfimprove de governing layer

Bij herhaling of gebruikerscorrectie:

- patch producer/runtimecode;
- voeg regressietest toe;
- patch governing skill/instructie;
- koppel automatische failure-ingest waar passend;
- commit/push/deploy en lees de actieve runtime terug.

Memory is alleen voor stabiele voorkeuren. Incidentdetails horen in deze
database, procedures in skills en productstate in de juiste repo.

## CLI-overzicht

```bash
# Open incidenten
hermes errors list --status open --text

# Teruggekeerde fouten
hermes errors list --status regressed --text

# Statistiek
hermes errors stats

# Portable/read-only export
hermes errors export \
  --output "$HERMES_HOME/state/error-knowledge/incidents.jsonl"
```

SQLite blijft de canonieke schrijflaag. **JSONL** is uitsluitend een
portable/read-only export voor inspectie, backup of externe analyse; importeer
wijzigingen niet terug als concurrerende waarheid.

## Outputcontract naar de gebruiker

Rapporteer compact:

1. wat nu praktisch werkt;
2. rootcause in één zin;
3. structurele preventie;
4. live bewijs;
5. eventueel resterend afzonderlijk incident.

Dump niet standaard de hele database of interne fingerprints. Noem de
fingerprint alleen wanneer die nodig is voor vervolgwerk.

## Testprompts

1. **“Automation-fout, fix structureel.”**
   - Zoek database, registreer incident, debug, regressietest, resolve met bewijs.
   - Voorkomt: alleen cron opnieuw starten en groen noemen.

2. **“MEDIA-bijlage is niet beschikbaar.”**
   - Traceer local path → parser → native platformmethod → remote message-id.
   - Voorkomt: `MEDIA:/path` opnieuw als tekst sturen zonder delivery proof.

3. **“Provider failed, pak verder op.”**
   - Zoek eerdere providerincidenten, inspecteer operation state en resumecontract.
   - Voorkomt: een nieuw leeg taskspoor starten of stale operationcard als actief behandelen.

4. **“Deze validatie reject terecht.”**
   - Geen incident registreren als de reject het bedoelde fail-closed contract is.
   - Voorkomt: veiligheids-/groundingguards als bugs repareren.

5. **“Fix de fout: Telegram-bijlage komt niet aan.”**
   - `github-work` routeert de repositorywijziging en stapelt `error-learning` voor
     search → record → rootcause → fix → proof → resolve.
   - Voorkomt: wel code repareren maar de herbruikbare les niet centraal opslaan.

6. **“error fix: gateway hervat op stale state.”**
   - Routeert direct naar deze skill en gebruikt dezelfde centrale SQLite DB.
   - Voorkomt: een los JSON-log of projectspecifieke foutendatabase starten.

## Veelgemaakte fouten

1. **Alles loggen.** Sla alleen herbruikbare technische signatures en minimale
   context op; geen ruwe privédata.
2. **Groen = resolved.** Zonder rootcause/fix/proof/prevention blijft het open.
3. **Per run een nieuw incident.** Gebruik stabiele component en signature;
   timestamps en PIDs worden genormaliseerd.
4. **Database als skillvervanger.** Database onthoudt gebeurtenissen; de skill
   bepaalt de werkwijze.
5. **Automatisch oplossen bij recovery.** Recovery kan transient zijn. Alleen
   verified resolve mag kennis canoniseren.
6. **Alleen lokaal patchen.** Structurele fixes vereisen canonieke source,
   runtime-install/parity en regressieproof.

## Verificatiechecklist

- [ ] Eerdere incidenten gezocht
- [ ] Veilige, stabiele fingerprint geregistreerd
- [ ] Rootcause met logs/reproductie bewezen
- [ ] Regressietest eerst rood en daarna groen
- [ ] Fix op canonieke governing surface
- [ ] Runtime/deployment/readback bewezen
- [ ] Resolve bevat oorzaak, fix, verificatie en preventie
- [ ] Geen secrets of private payloads opgeslagen

## Selected runtime policy

Read `.agents/skill-adapters/error-learning.md` from the explicitly selected target root when present. It supplies permitted runtime interfaces and incident-storage/delivery boundaries. Do not infer a target from this skill's installation. If the required Hermes interface is absent, retain the documented unavailable result rather than inventing a database or writer. Any push, deploy, external delivery or runtime reload mentioned above remains subject to the actual task and target authorization.
