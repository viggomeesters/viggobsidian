# Vaultskills

De publieke ingangen zijn `vault`, `vault-log`, `vault-process`, `vault-task`, `vault-tackle`, `vault-hub` en `vault-brief`. Elke ingang verwijst naar hetzelfde expliciete-doelcontract. Ze worden uit een gepinde release op stabiele `.agents/skills/<naam>/SKILL.md`-paden geïnstalleerd; hostprofielen kunnen naar die geïnstalleerde paden blijven verwijzen. De repositorycheckout wordt geen vault en krijgt geen brondata.

Een lokale adapter beschrijft de echte reader/writer, registratie-/claimprovider, intake, domeingates, Gitregels en strikte checker. Lokale scripts en interne `PROCEDURE.md`-routes blijven onder hun eigen eigenaar. Maak hiervoor geen nieuwe native ingangen. Persoonlijke coaching, mail, financiën en vendorplugins vallen buiten deze eerste vaultskillset.

Per wijziging: lees de ingang, gedeeld beleid, doelroot en lokale adapter; zoek de huidige eigenaar; respecteer registratieclaims en preimagecontroles; schrijf alleen geautoriseerde inhoud via de aangewezen provider; lees het resultaat terug en voer de toepasselijke lokale eindcontrole uit. Bronopslag, verwerking en uitvoering zijn afzonderlijke effecten. Een gecombineerd expliciet verzoek kan ze samen autoriseren.

Een ontbrekende adapter is geen permissie om een generieke writer te verzinnen. De publieke CLI beheert pakketbestanden; hij is geen capture-, taak- of registratiewriter.
