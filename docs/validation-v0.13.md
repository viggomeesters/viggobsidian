# Skillbibliotheek v0.13

De uitbreiding scheidt algemeen herbruikbare methoden van lokale werk-/privébindingen.
175 ingangs- en procedurebestanden zijn individueel beoordeeld. Bestaande gedeelde
vaultprocedures blijven behouden; gemengde methoden krijgen een publieke kern en
lokale adapters. Compatibiliteitsnamen verwijzen naar één canonieke procedure en
activeren geen uitgefaseerde workflowruntime.

`schemas/skill-library.json` specificeert 218 extra release-assets, inclusief de
benodigde methodereferenties en licentieverantwoording. Het aantal assets is geen
aantal unieke actieve skills. De eindige allowlist is in de updater gecompileerd;
een pakket kan geen eigen lokale adapters, schrijvers of scripts toevoegen.

Lokale verificatie omvat Rust fmt, Clippy en workspace-tests, expliciete dekking van
de allowlist, afwijzing van lokale provider-/adaptertargets, relatieve distributielinks
en een aanvullende scan op concrete hostpaden en echte maildomeinen. Een scan alleen
bewijst geen privacy: de publieke inhoud, provenance en gedragsgrenzen zijn daarnaast
onafhankelijk gelezen. Lokale broninventarissen en authentieke voorbeelden zijn niet
opgenomen in deze repository.

Vijf onafhankelijke afleidingsproeven betroffen tekst-only mailconcepten, plan-only
repositorywerk, coaching zonder actielus, documentbewerking met ontbrekende provider
en expliciet read-only hercontrole. Die proeven vonden geen blokkerende regressie;
zij bewijzen geen echte externe verzending, vendorruntime of live productie-uitvoering.

Niet-vrij herdistribueerbare specialisten worden niet gekopieerd of door parafrase
als algemene implementatie uitgebracht. De bibliotheek bevat daarvoor uitsluitend
onafhankelijk geschreven providerrouters. Hun bestaande gelicenseerde implementaties
en niet-publieke bindingsgegevens blijven lokaal, buiten het releasepakket.
