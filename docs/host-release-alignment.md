## CLI, pakket en lokale bronregistratie

Controleer bij een update drie afzonderlijke identiteiten: de executable die PATH
daadwerkelijk kiest, `.viggobsidian/installed.json` per gekozen vault en een
afzonderlijk gepinde tekstprovider. Een nieuwe provider installeren vervangt de
CLI of het vaultpakket niet. Bouw de CLI uit de gecontroleerde releasetree,
bewaar de oude executable en activeer de nieuwe versie via een atomisch vervangen
locator. Controleer de echte locator, executablehash en versie uit `help`; alleen een succesvolle build
is geen installatiebewijs. Wijzig een providerpin alleen als ook zijn API,
formaat en exacte executablehash opnieuw beoordeeld zijn.

`plan`/`apply` beheert een al aanwezige lokale `.agents/skills/_bundle/source-lock.json`
alleen als afgeleide lokale release-identiteit binnen dezelfde pakkettransactie. De
preview toont de exacte tien origin-revisionwijzigingen plus bron/preimagehash;
apply verifieert vóór iedere write de oude release-identiteit, alle tien beheerde
SKILL.md-hashes, pad-/symlinkgrenzen en drift. Alleen `origin` verandert naar de
nieuwe `installed.json`-revision; naam, pad, managementrol, hash-authority,
volgorde en overige velden/rijen blijven logisch behouden. De receipt bewaart
beide preimages en rollback weigert postimage- of managed drift voordat hij beide
identiteiten herstelt. Een ontbrekende source-lock blijft ondersteund voor een
installatie zonder source-lock, maar een revisionwijziging met een ontbrekende,
malformed of afwijkende source-lock faalt gesloten; de updater verzint of claimt
geen metadata. Dit is één lock-/journaltransactie met herstel na onderbreking,
geen claim van OS-brede multi-file-atomiciteit.

Draai daarna `doctor`, `skill-check`, de lokale profielcontrole en de relevante
providerregressies met de werkelijk geïnstalleerde CLI. De tien kernen, lokale
bindings en templates behouden hun eigen eigenaar. Deze configuratiemigratie
geeft geen notewrites of ruimere writerrechten. Vaultbrede historische
inhoudsbevindingen blijven afzonderlijk zichtbaar van de gerichte updatechecks.
