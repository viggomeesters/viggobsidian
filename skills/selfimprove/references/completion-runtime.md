# Completion- en runtimegates

Een pre-ship audit, review of delegated critic die de agent zelf start, is een
open oplevergate — ook wanneer de code al is gecommit, gedeployed en lokaal
groen.

### Versioned-runtime false-green guard

Voor repositories met een immutable/versioned deployment mag een task met
production- of live-evidence nooit worden gesloten op alleen een groene cronrun,
stille output, een bestaande deployment of een gepushte commit. Vergelijk vlak
voor `finish` minimaal:

1. `git rev-parse HEAD` van de shipped repository;
2. `source_commit` uit de actieve deploymentmanifest(en);
3. de source root die het live wrapper/script daadwerkelijk resolveert;
4. een run/readback die na die activation op exact dezelfde runtimepayload is gestart.

Als HEAD en active `source_commit` verschillen, moet de agent eerst bewijzen dat
het verschil uitsluitend workflow-/ledgermetadata buiten de geïnstalleerde
runtimepayload betreft, bijvoorbeeld met een gerichte `git diff --exit-code
<source_commit>..HEAD -- <alle runtime-owned paden>` plus byte/hash-readback van
het werkelijk geladen script. Zonder dat bewijs is de status maximaal
`implementation ready — rollout open`; finish/production-evidence is dan
verboden. Een latere stille/deferred cronrun op de oude deployment repareert dat
bewijs niet. Rollout, loaded payload-readback en de live smoke moeten eerst
op exact dezelfde runtimeinhoud worden uitgevoerd en letterlijk in
finish-evidence staan.

- Geef geen eindstatus met `klaar`, `opgeleverd`, `ship compleet` of een finale
  bewijssectie zolang zo'n child/delegation/background review nog draait of nog
  niet tegen de actuele HEAD is gereconcilieerd.
- Wacht op het resultaat, classificeer iedere finding tegen de actuele code,
  herstel valide residue en herhaal de finale HEAD/deploy/readback-gates vóór
  het eerste echte opleverbericht.
- Een tussenfase krijgt een begrensd label, bijvoorbeeld:
  `planning/wayfinding afgerond — implementatie en releasechecks volgen` of
  `implementatie gedeployed — onafhankelijke audit nog open`.
- Als een audit pas na een eindbericht binnenkomt, benoem dit als een
  procesfout; noem het niet simpelweg een normale late follow-up wanneer de
  agent die audit zelf vóór ship had gestart.
