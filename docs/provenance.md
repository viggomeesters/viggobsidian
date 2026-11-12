# Herkomst

ViggObsidian start als een opgeschoonde, zelfstandige bron zonder bestaande vaultinhoud of vault-Git-historie. De generieke policy en skillingangen zijn voor dit pakket geschreven. Vier Base-assets komen uit de eigen Vault Navigation-standaard; hun installatiepaden blijven behouden voor bestaande embeds.

De vroegere launcher, stackbootstrap en runtimepin kwamen uit eigen templatebeheer en zijn op 12 november 2026 uitgefaseerd. De bewaarde Git-historie is geen actuele uitvoerinstructie. Repositorywerk volgt GitHub Issue → PR → merge, met lokale controles en onafhankelijke review. Het product gebruikt geen GitHub Actions en installeert geen workflowruntime in aangesloten vaults.

Runtimeafhankelijkheden staan in Cargo.lock. `serde_json` en `sha2` verwerken manifeste JSON en SHA256; `serde-saphyr` leest profiel-/policydocumenten als YAML. Derdepartij-Obsidianplugins zijn geen pakketinhoud en blijven onder hun eigen licentie en installatiemanifest.

De volledige CSS bevat generieke SVG-iconen uit de bestaande stijl. Bijbehorende Lucide/Feather-notices staan in `LICENSES/lucide.txt`, afkomstig van https://github.com/lucide-icons/lucide/blob/main/LICENSE (gecontroleerd 11 november 2026). Het pakket levert die notices ook lokaal mee; runtimeverwijzingen naar Obsidian-/Lucide-iconnamen bundelen geen pluginimplementatie. Persoonlijke avatar-SVG’s en externe dossierafbeeldingen blijven uitsluitend lokale configuratie.
