# Herkomst

ViggObsidian start als een opgeschoonde, zelfstandige bron zonder bestaande vaultinhoud of vault-Git-historie. De generieke policy en skillingangen zijn voor dit pakket geschreven. Vier Base-assets komen uit de eigen Vault Navigation-standaard; hun installatiepaden blijven behouden voor bestaande embeds.

`go` en `scripts/bootstrap-stack.sh` komen uit het eigen `viggo-agent-stack-template`-beheer. De `.go/project.json` pint de runtime op een exacte stackrelease. Het product gebruikt geen GitHub Actions en installeert de Go-runtime niet in aangesloten vaults.

Runtimeafhankelijkheden staan in Cargo.lock. `serde_json` en `sha2` verwerken manifeste JSON en SHA256; `serde-saphyr` leest profiel-/policydocumenten als YAML. Derdepartij-Obsidianplugins zijn geen pakketinhoud en blijven onder hun eigen licentie en installatiemanifest.
