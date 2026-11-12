# Vaultskills

De publieke ingangen zijn `vault`, `vault-log`, `vault-process`, `vault-task`, `vault-tackle`, `vault-hub` en `vault-brief`. Elke ingang verwijst naar hetzelfde expliciete-doelcontract. Ze worden uit een gepinde release op stabiele `.agents/skills/<naam>/SKILL.md`-paden geïnstalleerd; hostprofielen kunnen naar die geïnstalleerde paden blijven verwijzen. De repositorycheckout wordt geen vault en krijgt geen brondata.

Vanaf v0.9 zijn ook de generieke interne procedures, actieve bronreferenties en
compatibilityaliases release-assets. `.viggobsidian/core/procedures.json` is de
volledige machineleesbare dekkingsmatrix; `core/procedures/*.md` bezit de gedeelde
gedragscontracten. De bestaande callerpaden blijven behouden. De zeven publieke
ingangen blijven het native menu; PROCEDURE.md maakt geen nieuwe slashcommand.

Een lokale adapter beschrijft de echte reader/writer, registratie-/claimprovider,
intake, domeingates, Gitregels en strikte checker. `.viggobsidian/providers.json`
bindt de expliciete doelroot en elke capability aan werkelijk aanwezige bestanden
en een lokale instructiesectie, of aan `unavailable` met een concrete reden.
Writer-/registratiescripts, tests, eigen brondata, persoonlijke modellen/domeinen
en eindige herstelregels blijven lokaal. Zes oude JSONL-recepten en een eindige
verhuisreferentie blijven inerte lokale historie; geen actieve gedeelde norm.

`vault-layout --vault ROOT --skill-route ROUTE` controleert uitsluitend de gekozen
gedeelde assets en afhankelijke lokale capability, zonder provideruitvoering.
Een source-import gebruikt daarnaast `--source-adapter imessage|reading-list|daily`.
`--skill-routes` controleert de gehele gedeelde dekking en expliciete providers;
een bekende unavailable provider is daar een zichtbare capabilitygrens. Een
gekozen unavailable route geeft exit 1; ongeldige metadata, rootmismatch, ontbrekende
files of hashdrift exit 2. Exit 0 bewijst geen claim, writeautoriteit of live gedrag.

Per wijziging: lees de ingang, gedeeld beleid, doelroot en lokale adapter; zoek de huidige eigenaar; respecteer registratieclaims en preimagecontroles; schrijf alleen geautoriseerde inhoud via de aangewezen provider; lees het resultaat terug en voer de toepasselijke lokale eindcontrole uit. Bronopslag, verwerking en uitvoering zijn afzonderlijke effecten. Een gecombineerd expliciet verzoek kan ze samen autoriseren.

Een ontbrekende adapter is geen permissie om een generieke writer te verzinnen. De publieke CLI beheert pakketbestanden; hij is geen capture-, taak- of registratiewriter.
