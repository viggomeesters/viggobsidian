//! Author the reviewed compatibility surface from the public coverage map.
//! This writes repository assets only, never selects or edits a target vault.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{error::Error, fs, path::Path};

fn write(root: &Path, name: &str, body: &str) -> Result<(), Box<dyn Error>> {
    let file = root.join(name);
    fs::create_dir_all(file.parent().unwrap())?;
    fs::write(file, body)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() == 2 && args[0] == "--adoption-map" {
        let vault = Path::new(&args[1]).canonicalize()?;
        let distribution: Value =
            serde_json::from_slice(&fs::read(root.join("distribution.json"))?)?;
        let mut hashes = serde_json::Map::new();
        for file in distribution["files"].as_array().unwrap() {
            let raw = file["target"].as_str().unwrap();
            let mut path = vault.clone();
            for part in Path::new(raw).components() {
                if !matches!(part, std::path::Component::Normal(_)) {
                    return Err("unsafe inventory path".into());
                }
                path.push(part);
                if fs::symlink_metadata(&path).is_ok_and(|m| m.is_symlink()) {
                    return Err("inventory symlink".into());
                }
            }
            if path.exists() {
                hashes.insert(
                    raw.into(),
                    json!(format!("{:x}", Sha256::digest(fs::read(path)?))),
                );
            }
        }
        println!("{}", serde_json::to_string_pretty(&hashes)?);
        return Ok(());
    }
    if !args.is_empty() {
        return Err("usage: procedure-assets [--adoption-map TARGET-VAULT]".into());
    }
    let spec: Value = serde_json::from_str(include_str!("../../schemas/procedures.json"))?;
    let mut files = Vec::new();
    for contract in spec["contracts"].as_array().unwrap() {
        let name = contract.as_str().unwrap();
        files.push(json!({"source":format!("procedures/{name}.md"),"target":format!(".viggobsidian/core/procedures/{name}.md")}));
    }
    for (name, route) in spec["routes"].as_object().unwrap() {
        if route["kind"] == "entry" {
            continue;
        }
        let contracts = route["contracts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                format!(
                    "- `.viggobsidian/core/procedures/{}.md`\n",
                    c.as_str().unwrap()
                )
            })
            .collect::<String>();
        let body = format!(
            "---\nname: {name}\ndescription: Gedeelde begrensde route voor {name}.\n---\n\n# {name}\n\nSelecteer de doelvault expliciet en lees daar deze gedeelde contracten:\n\n{contracts}\nLees de lokale capability `{}` in `.viggobsidian/providers.json` en de daar\ngebonden adapter voordat je een effect uitvoert. Controleer met\n`vault-layout --vault ROOT --skill-route {name}`. De procedure is releasebeheerd;\nlokale helpers behouden hun eigen authority en argumentcontract. Een aanwezige\nprocedure bewijst geen beschikbare provider. Alleen de gekozen werking uitvoeren;\nlegacy-intentie, bronopslag, kennisverwerking en taakuitvoering blijven gescheiden.\n",
            route["capability"].as_str().unwrap()
        );
        let source = format!("skills/{name}/PROCEDURE.md");
        write(root, &source, &body)?;
        files.push(json!({"source":source,"target":format!(".agents/skills/{name}/PROCEDURE.md")}));
    }
    for (reference, contract) in spec["references"].as_object().unwrap() {
        let (route, name) = reference.split_once('/').unwrap();
        let mut body = format!(
            "# {name}\n\nLees vanaf de expliciet gekozen doelvault\n`.viggobsidian/core/procedures/{}.md` en het gedeelde authoritycontract.\nDeze referentie behoudt de bestaande callerlocatie; de gedeelde norm staat\nin dat contract. De lokale adapter bezit exacte providerargumenten, datumbeleid,\nlimieten, persoonlijke voorkeuren en begrensde historische herstelroutes.\nEen historisch JSONL/CAS-recept is geen native writer of fallback.\n\nDe route/capability wordt gecontroleerd via\n`vault-layout --vault ROOT --skill-route {route}`.\n",
            contract.as_str().unwrap()
        );
        let headings: &[&str] = match reference.as_str() {
            "vault-task/lifecycle-flow" => &[
                "Intake bij een bestaande eigenaar",
                "Project creëren of adopteren",
                "Plan, uitvoeren en herstellen",
                "Bron naar kennis en acties",
                "Verplaatsen",
                "Wachten, actualiteit en storingen",
                "Identiteit, plaatsing en GitHub in dezelfde cyclus",
                "Agentafronding en actuele roadmap",
            ],
            "vault-task/task-changes" => &[
                "Lezen, claimen en plannen",
                "Velden en taakvormen",
                "Aanmaak en hervatten",
                "Relaties zonder ownerverhuizing",
                "Native agentupdates in een vrije taak",
                "Brongebonden roadmapgeschiedenis",
                "Agentafronding met gepind bewijs",
            ],
            "vault-review/shared-selection" => &[
                "Lezen en oppakken",
                "Loggen en verwerken",
                "Native inventaris binnen expliciete scopes",
                "Begrenzing en herstel",
                "Verhuisde bronpaden in bestaande hubs",
            ],
            _ => &[],
        };
        for heading in headings {
            body.push_str(&format!("\n## {heading}\n\nVolg de betreffende stap in het hierboven genoemde gedeelde contract en\nde actuele lokale provider. Bestaande task-/source-ID's, claims en hersteljournals\nblijven behouden; oude callerlocators geven geen nieuwe schrijfbevoegdheid.\n"));
        }
        let source = format!("skills/{route}/references/{name}.md");
        write(root, &source, &body)?;
        files.push(json!({"source":source,"target":format!(".agents/skills/{route}/references/{name}.md")}));
    }
    let native = "skills/_bundle/contracts/native-vault.md";
    write(
        root,
        native,
        "# Native vault contract\n\nSelecteer de doelvault expliciet. Lees daar `.viggobsidian/core/procedures/authority.md`\nen de lokale providerbinding. Lees voor bronlezen/opslag bovendien\n`.viggobsidian/core/procedures/read-capture.md`, voor inhoudelijke verwerking\n`.viggobsidian/core/procedures/process.md`, voor dailyverwerking `daily.md`\nen voor taken `task.md` in dezelfde map. Kies uitsluitend het geautoriseerde effect.\nGeen rootafleiding uit skillbron, cwd of imports. Bronidentiteit, complete bijlagen,\nclaims, semantische readback, daginhoud en writerweigering behouden hun grenzen.\nLokale writers, bronregisters en legacy-runtimeauthority blijven lokaal.\n",
    )?;
    files.push(json!({"source":native,"target":format!(".agents/{native}")}));
    files.push(
        json!({"source":"schemas/procedures.json","target":".viggobsidian/core/procedures.json"}),
    );
    let mut distribution: Value =
        serde_json::from_slice(&fs::read(root.join("distribution.json"))?)?;
    let entries = distribution["files"].as_array_mut().unwrap();
    for file in files {
        if !entries.iter().any(|old| old["target"] == file["target"]) {
            entries.push(file);
        }
    }
    write(
        root,
        "distribution.json",
        &(serde_json::to_string_pretty(&distribution)? + "\n"),
    )?;
    Ok(())
}
