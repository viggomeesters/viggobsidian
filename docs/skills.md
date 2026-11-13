# Vault-effecten en lokale providers

De ingang vault kiest een expliciet effect uit core/procedures.json v2. Oude
skillnamen zijn geen routes meer. De map bevat effects en echte bronadapters;
geen metadata voor de verwijderde procedurebibliotheek.

```sh
vault-layout --vault ROOT --skill-route vault
vault-layout --vault ROOT --skill-route vault --skill-capability process
vault-layout --vault ROOT --skill-route vault --skill-capability capture --skill-source imessage
vault-layout --vault ROOT --skill-routes
```

Zonder capability kiest vault read. --source-adapter blijft een ondersteunde
spelling van --skill-source voor bestaande CLI-callers; het is geen oude
skillalias. Een bronadapter is een aanvullende afhankelijkheid van capture,
process, document of mail en vervangt hun basisprovider niet. De volledige
controle leest alle effecten en bronproviders; unavailable is daar een zichtbare
grens. Een gekozen unavailable effect geeft exit 1; ongeldige metadata,
rootmismatch, ontbrekende bestanden, onbekende oude routenaam of hashdrift exit 2.
Exit 0 geeft geen claim, schrijfbevoegdheid of bewijs van live gedrag.

providers.json blijft lokaal, met exact doelpad, capability en beschikbare
instructiesectie/bestanden of unavailable met reden. De checker controleert
release-assets, dependencies, symlinkgrenzen en tijdens de controle gewijzigde
bindings; hij voert geen provider uit. Productwriters, claims en stricte lokale
gates blijven leidend. Een aanwezige guided provider is geen generieke writer.

Lees alleen de inhoudelijke contracten die het effect nodig heeft. Bronopslag,
kennisverwerking en uitvoering houden hun afzonderlijke autorisatie; een gecombineerd
expliciet verzoek kan ze samen autoriseren. Na schrijven volgt semantische readback
bij de bestaande eigenaar. De publieke pakket-CLI is geen bron-/taakwriter.

Obsidian-pluginwerk gebruikt github-work/references/obsidian-plugins.md voor de
grenzen van source-preview, public-release en catalog-install. plugin-check toetst
kandidaat, exacte releasebytes en optioneel echte installatie afzonderlijk.
