## CLI, pakket en lokale bronregistratie

Controleer bij een update drie afzonderlijke identiteiten: de executable die PATH
daadwerkelijk kiest, `.viggobsidian/installed.json` per gekozen vault en een
afzonderlijk gepinde tekstprovider. Een nieuwe provider installeren vervangt de
CLI of het vaultpakket niet. Bouw de CLI uit de gecontroleerde releasetree,
bewaar de oude executable en activeer de nieuwe versie via een atomisch vervangen
locator. Controleer de echte locator, executablehash en versie uit `help`; alleen een succesvolle build
is geen installatiebewijs. Wijzig een providerpin alleen als ook zijn API,
formaat en exacte executablehash opnieuw beoordeeld zijn.

`plan`/`apply` beheert geen lokale `.agents/skills/_bundle/source-lock.json`.
Na een pakketupdate kan de bronregistratie dus nog naar de vorige release wijzen.
Deze binding krijgt een afzonderlijke beoordeelde migratie: bewaar de exacte
preimage, verifieer alle tien bestaande bronidentiteiten en alle geïnstalleerde
skillhashes tegen het gecontroleerde pakket, en verander uitsluitend de
`origin`-revision naar de nieuwe `installed.json`-revision. Behoud naam, doelpad,
managementrol en hash-authority. Controleer de oude bytes opnieuw vlak voor de
atomische vervanging; lokale drift vereist een nieuw plan. Een ontbrekende of
afwijkende identiteit vraagt onderzoek, geen blind vernieuwde metadata.

Draai daarna `doctor`, `skill-check`, de lokale profielcontrole en de relevante
providerregressies met de werkelijk geïnstalleerde CLI. De tien kernen, lokale
bindings en templates behouden hun eigen eigenaar. Deze configuratiemigratie
geeft geen notewrites of ruimere writerrechten. Vaultbrede historische
inhoudsbevindingen blijven afzonderlijk zichtbaar van de gerichte updatechecks.
