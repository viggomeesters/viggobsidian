use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

type ProviderDocumentMutation = Box<dyn Fn(&mut Value)>;

#[test]
fn named_wiki_guides_opt_in_to_the_existing_overview_layout() {
    let fixture = Fixture::new("wiki-guide-overview");
    let folder = "99-system/vault/example/wiki";
    let guide = format!("{folder}/guide.md");
    fixture.write(
        "99-system/contracts/system-layout.yaml",
        format!("collections: []\nfixed_files: [{guide}]\ndelegated: {{}}\n"),
    );
    fixture.write(
        &format!("{folder}/wiki.md"),
        "---\ntype: note\ncategory: knowledge\nlayer: wiki\n---\n# Wiki\n",
    );
    let metadata = "---\ntype: reference\ncategory: guide\nlayer: wiki\ncssclasses: [vault-wiki, vault-wiki-overview]\n---\n";
    let header = "# Guide\n\n> [!navigation|shortcuts]\n> - [[fictional-inbox|Back]]\n> - [[fictional-raw|Raw]]\n> - [Finder](file:///fictional/wiki)\n\n";
    let facts = "> [!wiki-facts]\n> - Region **North**\n> - Service **Express**\n\n";
    let toc = "```wiki-toc\n```\n\n";
    let sections = "## Intake\n\n## Processing\n\n## Follow-up\n\n## Checks\n";
    let guide_findings = |note: String| {
        fixture.write(&guide, note);
        let report: Value = serde_json::from_slice(&fixture.strict(folder).stdout).unwrap();
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["path"] == guide)
            .map(|f| f["code"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    assert!(guide_findings(format!("{metadata}{header}{facts}{toc}{sections}")).is_empty());
    let notice = "<!-- viggobsidian:knowledge-owner -->\n> [!info] Actuele kennis\n> [[wiki|Open actuele kennis]]\n<!-- /viggobsidian:knowledge-owner -->\n";
    assert!(guide_findings(format!("{metadata}{notice}{header}{facts}{toc}{sections}")).is_empty());
    assert_eq!(
        guide_findings(format!("{metadata}{header}{facts}{sections}")),
        ["wiki-toc"]
    );
    assert_eq!(
        guide_findings(format!("{metadata}{header}{toc}{sections}")),
        ["wiki-facts"]
    );
    let ordinary = metadata.replace(", vault-wiki-overview", "");
    assert!(guide_findings(format!("{ordinary}{header}{sections}")).is_empty());
}

#[test]
fn shared_source_preparation_goldens_pass_native_raw_gates() {
    let fixture = Fixture::new("shared-source-preparation");
    fixture.write(
        "30-topics/example/example.md",
        "---\ntype: note\ncategory: knowledge\n---\n# Example\n",
    );
    fixture.write(
        "30-topics/example/raw/raw.md",
        "---\ntype: collection\ncategory: index\n---\n# Raw\n",
    );
    fixture.write("30-topics/example/wiki/wiki.md", "---\ntype: note\ncategory: knowledge\n---\n# Wiki\n\n## Existing knowledge\n\nExisting knowledge remains here.\n");
    let package = "30-topics/example/raw/20261102-source";
    let path = format!("{package}/20261102-source.md");
    for expected in [
        include_str!("../fixtures/source-prepared-v1.json"),
        include_str!("../fixtures/source-processed-v1.json"),
    ] {
        let expected: String = serde_json::from_str(expected).unwrap();
        let rendered = expected.replace("/fictional/vault", fixture.0.to_str().unwrap());
        fixture.write(&path, &rendered);
        let out = fixture.strict(package);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
        fixture.write(&path, rendered + "\nUnwrapped original source.\n");
        let invalid = fixture.strict(package);
        assert!(!invalid.status.success());
        assert!(String::from_utf8_lossy(&invalid.stdout).contains("strict-event-header"));
    }
}

#[test]
fn adopted_engine_reads_shared_metadata_contract_and_refuses_missing_core() {
    let (fixture, raw) = intake_fixture("21-program", "shared-navigation-contract");
    let schema = fixture.0.join("99-system/contracts/vault-schema.yaml");
    let local = fs::read_to_string(&schema).unwrap().replace(
        "required: [type, category]",
        "required: [type, category, legacy_only]",
    );
    fs::write(&schema, local).unwrap();
    fixture.write(".viggobsidian/installed.json", "{}");
    fixture.write(
        ".viggobsidian/core/navigation-contract.yaml",
        include_str!("../fixtures/navigation-contract.yaml"),
    );
    let input = json!({"date":"2026-11-11", "slug":"shared-task", "title":"Shared contract", "request":"Fictional input"});
    let created = intake(&fixture, &raw, &input);
    assert!(
        created.status.success(),
        "{}",
        String::from_utf8_lossy(&created.stdout)
    );
    let receipt: Value = serde_json::from_slice(&created.stdout).unwrap();
    let package = Path::new(receipt["path"].as_str().unwrap())
        .parent()
        .unwrap()
        .to_str()
        .unwrap();
    assert!(fixture.strict(package).status.success());
    fs::remove_file(
        fixture
            .0
            .join(".viggobsidian/core/navigation-contract.yaml"),
    )
    .unwrap();
    assert_eq!(fixture.strict(package).status.code(), Some(2));
}

#[test]
fn check_all_rejects_mutating_mode_combinations_before_writes() {
    let fixture = Fixture::new("exclusive-checks");
    fixture.write("untouched.md", "[[unresolved/target]]");
    for args in [
        vec!["--check-all", "--fix-link-style"],
        vec![
            "--check-all",
            "--new-work-task",
            "21-program/2026-11-example/ex-00-project-wiki/raw",
        ],
        vec!["--check-all", "--check-work"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
            .arg("--vault")
            .arg(&fixture.0)
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(
            fs::read_to_string(fixture.0.join("untouched.md")).unwrap(),
            "[[unresolved/target]]"
        );
        assert!(!fixture.0.join("21-program").exists());
    }
}

#[test]
fn link_style_cli_checks_repairs_and_rechecks_without_touching_sources() {
    let fixture = Fixture::new("link-style");
    let wiki = "d/wiki/guide/guide.md";
    let raw = "d/raw/20261111-source/20261111-source.md";
    let original_raw = "---\nwiki_target: '[[d/wiki/guide/guide#Bron]]'\n---\n# Bron\n\n## Activity\n> [!activity-message|user] Alex\n> [[d/wiki/guide/guide]]\n";
    fixture.write(
        wiki,
        "---\nlayer: wiki\n---\n# Guide\n\n[[d/raw/20261111-source/20261111-source#Bron]]\n",
    );
    fixture.write(raw, original_raw);
    fixture.write(
        "10-daily/notes/2026-11-11.md",
        "# Vandaag\n\n| Tijd | Notitie |\n| --- | --- |\n| 12:00 | [[d/wiki/guide/guide]] |\n",
    );
    let run = |mode: &str| {
        Command::new(env!("CARGO_BIN_EXE_vault-layout"))
            .arg("--vault")
            .arg(&fixture.0)
            .args([mode, "--json"])
            .output()
            .unwrap()
    };
    assert_eq!(run("--link-style").status.code(), Some(1));
    assert!(run("--fix-link-style").status.success());
    assert_eq!(
        fs::read_to_string(fixture.0.join(raw)).unwrap(),
        original_raw
    );
    let fixed = fs::read_to_string(fixture.0.join(wiki)).unwrap();
    assert!(fixed.contains("cssclasses: [vault-wiki]"));
    assert!(fixed.contains("[[20261111-source#Bron|Bron ↗]]"));
    assert!(
        fs::read_to_string(fixture.0.join("10-daily/notes/2026-11-11.md"))
            .unwrap()
            .contains("[[guide]]")
    );
    assert!(
        fixture
            .0
            .join("10-daily/notes/.raycast-capture.lock")
            .is_file()
    );
    let again = run("--fix-link-style");
    assert!(again.status.success());
    let report: Value = serde_json::from_slice(&again.stdout).unwrap();
    assert_eq!(report["findings"], 0);
    assert_eq!(report["changed"], json!({}));
    assert!(run("--link-style").status.success());
    fixture.write("broken.md", "[[unresolved/target]]");
    assert_eq!(run("--fix-link-style").status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(fixture.0.join("broken.md")).unwrap(),
        "[[unresolved/target]]"
    );
}

struct Fixture(PathBuf);

fn procedure_fixture(name: &str) -> Fixture {
    let fixture = Fixture::new(name);
    // Compiled generic bytes, never a live vault or source-checkout fallback.
    let bundle: Value =
        serde_json::from_str(include_str!("../fixtures/procedure-assets.json")).unwrap();
    assert_eq!(bundle["schema"], "viggobsidian.procedure-test-fixture.v1");
    let mut entries = Vec::new();
    let mut files = serde_json::Map::new();
    for (target, body) in bundle["files"].as_object().unwrap() {
        let bytes = body["content"].as_str().unwrap().as_bytes();
        let hash = format!("{:x}", Sha256::digest(bytes));
        fixture.write(target, bytes);
        files.insert(target.into(), json!(hash));
        entries.push(json!({"source":body["source"],"target":target,"sha256":hash}));
    }
    let manifest=json!({"schema":"viggobsidian.package.v1","version":"0.9.0","revision":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","files":entries}).to_string();
    fixture.write(".viggobsidian/installed.json",json!({"schema":"viggobsidian.installed.v1","version":"0.9.0","revision":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","manifest_sha256":format!("{:x}",Sha256::digest(manifest.as_bytes())),"manifest_bytes":manifest,"files":files}).to_string());
    let spec: Value = serde_json::from_slice(
        &fs::read(fixture.0.join(".viggobsidian/core/procedures.json")).unwrap(),
    )
    .unwrap();
    let mut capabilities = serde_json::Map::new();
    for name in spec["effects"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .chain(
            spec["source_adapters"]
                .as_object()
                .unwrap()
                .values()
                .map(|v| v.as_str().unwrap()),
        )
    {
        capabilities.insert(name.into(),json!({"state":"unavailable","mode":"unavailable","reason":"Not installed in this fixture"}));
    }
    capabilities.insert("read".into(),json!({"state":"available","mode":"read-only","instructions":"provider/read.md","section":"read","required_files":["provider/reader.txt"]}));
    fixture.write(
        "provider/read.md",
        "# Local provider\n\n## read\nUse bounded read-only current Markdown.\n",
    );
    fixture.write("provider/reader.txt", "Do not execute this file.\n");
    fixture.write(".viggobsidian/providers.json",json!({"schema":"viggobsidian.providers.v1","vault":fixture.0.canonicalize().unwrap(),"capabilities":capabilities}).to_string());
    fixture
}

fn providers_v2(first: &Fixture, second: &Fixture) {
    let first_file = first.0.join(".viggobsidian/providers.json");
    let second_file = second.0.join(".viggobsidian/providers.json");
    let first_document: Value = serde_json::from_slice(&fs::read(&first_file).unwrap()).unwrap();
    let second_document: Value = serde_json::from_slice(&fs::read(&second_file).unwrap()).unwrap();
    let first_root = first.0.canonicalize().unwrap();
    let second_root = second.0.canonicalize().unwrap();
    let bindings = json!([
        {"vault":second_root,"capabilities":second_document["capabilities"]},
        {"vault":first_root,"capabilities":first_document["capabilities"]}
    ]);
    let document = json!({"schema":"viggobsidian.providers.v2","bindings":bindings});
    fs::write(&first_file, document.to_string()).unwrap();
    fs::write(&second_file, document.to_string()).unwrap();
}

fn skill_check(fixture: &Fixture, args: &[&str]) -> (Output, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
        .arg("--vault")
        .arg(&fixture.0)
        .args(args)
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")))
        .output()
        .unwrap();
    let report = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output, report)
}

#[test]
fn procedure_preflight_reads_explicit_target_without_executing_or_mutating() {
    let fixture = procedure_fixture("procedures-explicit");
    let before = fs::read(fixture.0.join("provider/reader.txt")).unwrap();
    let (output, report) = skill_check(&fixture, &["--skill-route", "vault"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(report["write_authority"], false);
    assert_eq!(report["routes"][0]["mode"], "read-only");
    assert_eq!(
        fs::read(fixture.0.join("provider/reader.txt")).unwrap(),
        before
    );
    assert_eq!(
        report["providers_sha256"],
        format!(
            "{:x}",
            Sha256::digest(fs::read(fixture.0.join(".viggobsidian/providers.json")).unwrap())
        )
    );
    assert_eq!(
        skill_check(
            &fixture,
            &["--skill-route", "vault", "--skill-capability", "hub"]
        )
        .0
        .status
        .code(),
        Some(1)
    );
    let (output, report) = skill_check(&fixture, &["--skill-routes"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(report["routes"].as_array().unwrap().len(), 31);
    let missing = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
        .args(["--skill-route", "vault"])
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(2));
}

#[test]
fn providers_v2_selects_the_exact_requested_root_without_checking_other_host_dependencies() {
    let first = procedure_fixture("providers-v2-first");
    let second = procedure_fixture("providers-v2-second");
    providers_v2(&first, &second);
    let first_file = first.0.join(".viggobsidian/providers.json");
    let second_file = second.0.join(".viggobsidian/providers.json");
    let mut document: Value = serde_json::from_slice(&fs::read(&first_file).unwrap()).unwrap();
    document["bindings"][0]["capabilities"]["read"]["external_files"] =
        json!(["/fictional/unavailable/host-provider.txt"]);
    document["bindings"][0]["capabilities"]["capture"] =
        json!({"state":"unavailable","mode":"unavailable","reason":"Unavailable on this host"});
    fs::write(&first_file, document.to_string()).unwrap();
    fs::write(&second_file, document.to_string()).unwrap();

    let first_before = fs::read(&first_file).unwrap();
    let second_before = fs::read(&second_file).unwrap();
    let (first_output, first_report) = skill_check(&first, &["--skill-route", "vault"]);
    assert!(first_output.status.success(), "{first_report}");
    assert_eq!(first_report["routes"][0]["capability"], "read");
    assert_eq!(first_report["routes"][0]["state"], "available");
    assert_eq!(fs::read(&first_file).unwrap(), first_before);

    let (second_output, second_report) = skill_check(
        &second,
        &["--skill-route", "vault", "--skill-capability", "capture"],
    );
    assert_eq!(second_output.status.code(), Some(1), "{second_report}");
    assert_eq!(second_report["routes"][0]["capability"], "capture");
    assert_eq!(second_report["routes"][0]["state"], "unavailable");
    assert_eq!(fs::read(&second_file).unwrap(), second_before);
}

#[test]
fn providers_v2_refuses_malformed_or_ambiguous_binding_identities_without_mutation() {
    let first = procedure_fixture("providers-v2-invalid-first");
    let second = procedure_fixture("providers-v2-invalid-second");
    providers_v2(&first, &second);
    let config = first.0.join(".viggobsidian/providers.json");
    let original = fs::read(&config).unwrap();
    let first_root = first.0.canonicalize().unwrap();
    let second_root = second.0.canonicalize().unwrap();
    let parent = first_root.parent().unwrap();
    let unnormalized = parent.join(first_root.file_name().unwrap()).join("..");

    let cases: Vec<(&str, ProviderDocumentMutation)> = vec![
        (
            "unknown document key",
            Box::new(|value| value["unknown"] = json!(true)),
        ),
        (
            "empty bindings",
            Box::new(|value| value["bindings"] = json!([])),
        ),
        (
            "unknown binding key",
            Box::new(|value| value["bindings"][0]["unknown"] = json!(true)),
        ),
        (
            "non-string root",
            Box::new(|value| value["bindings"][0]["vault"] = json!(42)),
        ),
        (
            "relative root",
            Box::new(|value| value["bindings"][0]["vault"] = json!("relative/vault")),
        ),
        (
            "unnormalized root",
            Box::new(move |value| value["bindings"][0]["vault"] = json!(unnormalized)),
        ),
        (
            "repeated separator",
            Box::new(|value| value["bindings"][0]["vault"] = json!("/fictional//vault")),
        ),
        (
            "embedded current-directory segment",
            Box::new(|value| value["bindings"][0]["vault"] = json!("/fictional/./vault")),
        ),
        (
            "trailing separator",
            Box::new(|value| value["bindings"][0]["vault"] = json!("/fictional/vault/")),
        ),
        (
            "null byte in root",
            Box::new(|value| value["bindings"][0]["vault"] = json!("/fictional/\0vault")),
        ),
        (
            "non-object capabilities",
            Box::new(|value| value["bindings"][0]["capabilities"] = json!([])),
        ),
        (
            "missing requested root",
            Box::new(|value| value["bindings"][1]["vault"] = json!("/fictional/missing-vault")),
        ),
        (
            "duplicate root",
            Box::new(move |value| value["bindings"][1]["vault"] = json!(second_root)),
        ),
    ];
    for (name, mutate) in cases {
        let mut document: Value = serde_json::from_slice(&original).unwrap();
        mutate(&mut document);
        let bytes = document.to_string().into_bytes();
        fs::write(&config, &bytes).unwrap();
        let output = skill_check(&first, &["--skill-route", "vault"]).0;
        assert_eq!(output.status.code(), Some(2), "{name}");
        assert_eq!(fs::read(&config).unwrap(), bytes, "{name}");
    }
}

#[test]
fn procedure_importer_is_additional_and_every_source_provider_is_validated() {
    let fixture = procedure_fixture("procedures-import");
    let file = fixture.0.join(".viggobsidian/providers.json");
    let mut providers: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    providers["capabilities"]["imessage"] = providers["capabilities"]["read"].clone();
    fs::write(&file, providers.to_string()).unwrap();
    let (output, report) = skill_check(
        &fixture,
        &[
            "--skill-route",
            "vault",
            "--skill-capability",
            "process",
            "--source-adapter",
            "imessage",
        ],
    );
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert_eq!(report["routes"].as_array().unwrap().len(), 2);
    assert_eq!(report["routes"][0]["capability"], "process");
    providers["capabilities"]["imessage"] = json!({});
    fs::write(&file, providers.to_string()).unwrap();
    assert_eq!(
        skill_check(&fixture, &["--skill-routes"]).0.status.code(),
        Some(2)
    );
    assert_eq!(
        skill_check(
            &fixture,
            &[
                "--skill-route",
                "vault",
                "--skill-capability",
                "task",
                "--source-adapter",
                "daily"
            ]
        )
        .0
        .status
        .code(),
        Some(2)
    );
}

#[test]
fn procedure_preflight_rejects_root_drift_missing_provider_and_release_drift() {
    let fixture = procedure_fixture("procedures-binding");
    let config = fixture.0.join(".viggobsidian/providers.json");
    let original = fs::read(&config).unwrap();
    let mut value: Value = serde_json::from_slice(&original).unwrap();
    value["vault"] = json!(fixture.0.join("other"));
    fs::write(&config, value.to_string()).unwrap();
    assert_eq!(
        skill_check(&fixture, &["--skill-route", "vault"])
            .0
            .status
            .code(),
        Some(2)
    );
    fs::write(&config, &original).unwrap();
    fs::remove_file(fixture.0.join("provider/reader.txt")).unwrap();
    assert_eq!(
        skill_check(&fixture, &["--skill-route", "vault"])
            .0
            .status
            .code(),
        Some(2)
    );
    fixture.write("provider/reader.txt", "Do not execute this file.\n");
    fixture.write(
        ".viggobsidian/core/procedures/read-capture.md",
        "Locally changed contract",
    );
    assert_eq!(
        skill_check(&fixture, &["--skill-route", "vault"])
            .0
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn procedure_preflight_refuses_alias_escape_symlinks_and_unknown_transforms() {
    let fixture = procedure_fixture("procedures-malformed");
    let config = fixture.0.join(".viggobsidian/providers.json");
    let original = fs::read(&config).unwrap();
    let mut value: Value = serde_json::from_slice(&original).unwrap();
    value["capabilities"]["read"]["required_files"] = json!(["../escape"]);
    fs::write(&config, value.to_string()).unwrap();
    assert_eq!(
        skill_check(&fixture, &["--skill-route", "vault"])
            .0
            .status
            .code(),
        Some(2)
    );
    fs::write(&config, &original).unwrap();
    let reader = fixture.0.join("provider/reader.txt");
    fs::remove_file(&reader).unwrap();
    symlink(fixture.0.join("provider/read.md"), &reader).unwrap();
    assert_eq!(
        skill_check(&fixture, &["--skill-route", "vault"])
            .0
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        skill_check(&fixture, &["--skill-route", "unknown"])
            .0
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        skill_check(&fixture, &["--skill-route", "vault", "--check-all"])
            .0
            .status
            .code(),
        Some(2)
    );
    fs::remove_file(&reader).unwrap();
    fixture.write("provider/reader.txt", "Do not execute this file.\n");
    let installed = fixture.0.join(".viggobsidian/installed.json");
    let mut state: Value = serde_json::from_slice(&fs::read(&installed).unwrap()).unwrap();
    let mut manifest: Value =
        serde_json::from_str(state["manifest_bytes"].as_str().unwrap()).unwrap();
    let target = manifest["files"][0]["target"].as_str().unwrap().to_owned();
    manifest["files"][0]["transform"] = json!("arbitrary-procedure-transform");
    let raw = manifest.to_string();
    state["manifest_bytes"] = json!(raw);
    state["manifest_sha256"] = json!(format!("{:x}", Sha256::digest(raw.as_bytes())));
    state["derived"][&target]["sha256"] = state["files"][&target].clone();
    fs::write(installed, state.to_string()).unwrap();
    assert_eq!(
        skill_check(&fixture, &["--skill-route", "vault"])
            .0
            .status
            .code(),
        Some(2)
    );
}

fn adoption_command(fixture: &Fixture, mode: &str, input: &Value) -> (Output, Value) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
        .arg("--vault")
        .arg(&fixture.0)
        .arg(mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    serde_json::to_writer(child.stdin.take().unwrap(), input).unwrap();
    let output = child.wait_with_output().unwrap();
    let value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output, value)
}

fn adopt(fixture: &Fixture, owners: Value) {
    let (output, plan) = adoption_command(
        fixture,
        "--adoption-plan",
        &json!({"schema":"viggobsidian.adoption-request.v1","owners":owners}),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let (output, _) = adoption_command(fixture, "--adoption-apply", &plan);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn appended() -> &'static str {
    "\n> [!activity-day|2026-11-12] 12 November 2026\n\n> [!activity-message|human] Alex\n> `— · Alex · chat`\n> New source-bound information.\n"
}

#[cfg(unix)]
struct RelocationFixture {
    vault: Fixture,
}

#[cfg(unix)]
impl RelocationFixture {
    const OLD: &'static str = "15-projects";
    const NEW: &'static str = "20-projects";
    const LEGACY: &'static str = "2026-11-example/legacy/legacy.md";
    const STRICT: &'static str = "2026-11-example/strict/strict.md";
    const HISTORICAL: &'static str = "---\ntype: note\ncategory: knowledge\nactivity_contract: provenance-v1\n---\n# Legacy\n\n## Activity\n\n> [!activity-day|2026-11-11] 11 November 2026\n\n> [!activity-message|human] Alex\n> Historical source with no stamp.\n";
    fn new(name: &str) -> Self {
        let vault = Fixture::new(name);
        vault.write(".viggobsidian/integration.json", r#"{"schema":"viggobsidian.integration.v1","profile":"numbered-v1","adoption":{"legacy_project_roots":["15-projects","17-other"]},"local_adapter":"example-local.md"}"#);
        vault.write("15-projects/15-projects.md", "# Projects\n");
        vault.write("15-projects/2026-11-example/2026-11-example.md", "---\ntype: project\nproject-month: \"2026-11\"\nproject-code: EX\nproject_id: stable-project-id\n---\n# Example\n");
        vault.write(
            "15-projects/2026-11-example/.original.bin",
            b"Original source bytes\x00",
        );
        vault.write(
            "15-projects/2026-11-example/legacy/task.md",
            "---\ntask-id: 00000000-0000-4000-8000-000000000001\n---\n# A task\n",
        );
        vault.write(&format!("15-projects/{}", Self::LEGACY), Self::HISTORICAL);
        vault.write(
            &format!("15-projects/{}", Self::STRICT),
            "---\ntype: note\ncategory: knowledge\nlayer: wiki\n---\n# Strict\n",
        );
        vault.write("outside/outside.md", "# Original outside\n");
        adopt(
            &vault,
            json!({"15-projects/2026-11-example/legacy":"legacy", "outside":"legacy"}),
        );
        // Pin a strict owner next to the legacy owner.
        let path = vault.0.join(".viggobsidian/adoption.json");
        let mut state: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        state["owners"]["15-projects/2026-11-example/strict"] =
            json!({"state":"strict","files":{}});
        fs::write(path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
        Self { vault }
    }

    fn run(&self, mode: &str, input: &Value) -> (Output, Value) {
        let mut child = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
            .arg("--vault")
            .arg(&self.vault.0)
            .args(["--project-relocation", mode])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        serde_json::to_writer(child.stdin.take().unwrap(), input).unwrap();
        let output = child.wait_with_output().unwrap();
        let value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
        (output, value)
    }

    fn success(&self, mode: &str, input: &Value) -> Value {
        let (output, value) = self.run(mode, input);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        value
    }

    fn request() -> Value {
        json!({"schema":"viggobsidian.project-relocation-request.v1","source":Self::OLD})
    }

    fn simulated_obsidian_folder_move(&self) {
        // This represents the external FileManager boundary in the native
        // test. The production CLI never renames content.
        fs::rename(self.vault.0.join(Self::OLD), self.vault.0.join(Self::NEW)).unwrap();
    }

    fn simulated_obsidian_rootnote_rename(&self) {
        fs::rename(
            self.vault.0.join(Self::NEW).join("15-projects.md"),
            self.vault.0.join(Self::NEW).join("20-projects.md"),
        )
        .unwrap();
    }
}

#[test]
#[cfg(unix)]
fn project_relocation_preserves_project_routes_adoption_and_write_boundaries() {
    let fixture = RelocationFixture::new("project-relocation-preserves");
    let old_coll = format!("{}/{}", RelocationFixture::OLD, RelocationFixture::LEGACY);
    let new_coll = format!("{}/{}", RelocationFixture::NEW, RelocationFixture::LEGACY);
    let state_path = fixture.vault.0.join(".viggobsidian/adoption.json");
    let integration_path = fixture.vault.0.join(".viggobsidian/integration.json");
    fs::set_permissions(&state_path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::set_permissions(&integration_path, fs::Permissions::from_mode(0o640)).unwrap();
    let old_state: Value = serde_json::from_slice(&fs::read(&state_path).unwrap()).unwrap();
    let readback = |path: &str| {
        write_check(
            &fixture.vault,
            "baseline",
            json!([{"path":path,"expect":"present"}]),
            Value::Null,
        )
    };
    let (_, before) = readback(&old_coll);
    assert!(
        before["legacy_findings"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty())
    );
    let plan = fixture.success("plan", &RelocationFixture::request());
    assert_eq!(plan["project_routes"][0]["id"], "stable-project-id");
    assert_eq!(
        fixture.success("begin", &plan)["status"],
        "prepared-for-obsidian"
    );
    assert_eq!(readback(&old_coll).0.status.code(), Some(2));
    assert_eq!(fixture.run("apply", &plan).0.status.code(), Some(2));
    fixture.simulated_obsidian_folder_move();
    assert_eq!(fixture.run("validate", &plan).0.status.code(), Some(2));
    assert_eq!(fixture.run("abort", &plan).0.status.code(), Some(2));
    fixture.simulated_obsidian_rootnote_rename();
    assert_eq!(
        fixture.success("validate", &plan)["status"],
        "verified-move-inventory"
    );
    assert_eq!(
        fixture.success("apply", &plan)["status"],
        "adoption-and-integration-rebound"
    );
    assert_eq!(
        fs::metadata(&state_path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(&integration_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
    let next: Value = serde_json::from_slice(&fs::read(&state_path).unwrap()).unwrap();
    let old_owner = "15-projects/2026-11-example/legacy";
    let new_owner = "20-projects/2026-11-example/legacy";
    assert_eq!(
        old_state["owners"][old_owner]["files"][&old_coll],
        next["owners"][new_owner]["files"][&new_coll]
    );
    assert_eq!(next["owners"][old_owner], Value::Null);
    assert_eq!(
        next["owners"]["20-projects/2026-11-example/strict"]["state"],
        "strict"
    );
    assert_eq!(next["owners"]["outside"], old_state["owners"]["outside"]);
    let integration: Value = serde_json::from_slice(
        &fs::read(fixture.vault.0.join(".viggobsidian/integration.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        integration["adoption"]["legacy_project_roots"],
        json!(["17-other"])
    );
    assert_eq!(integration["local_adapter"], "example-local.md");
    let (_, new_baseline) = readback(&new_coll);
    assert_eq!(
        before["legacy_findings"]
            .to_string()
            .replace("15-projects/", "20-projects/"),
        new_baseline["legacy_findings"].to_string(),
    );
    let changes = json!([{"path":new_coll,"expect":"present"}]);
    assert_eq!(
        write_check(&fixture.vault, "validate", changes.clone(), before)
            .0
            .status
            .code(),
        Some(2)
    );
    fixture.vault.write(
        &new_coll,
        format!("{}{}", RelocationFixture::HISTORICAL, appended()),
    );
    let (out, report) = write_check(&fixture.vault, "validate", changes, new_baseline);
    assert!(out.status.success(), "{report}");
    let strict_path = format!("20-projects/{}", RelocationFixture::STRICT);
    fixture
        .vault
        .write(&strict_path, RelocationFixture::HISTORICAL);
    let (_, report) = readback(&strict_path);
    assert!(
        !report["legacy_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["path"] == strict_path)
    );
    assert!(
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["path"] == strict_path)
    );
    let route = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
        .args(["--vault"])
        .arg(&fixture.vault.0)
        .args(["--json", "--resolve-code", "EX"])
        .output()
        .unwrap();
    assert!(
        route.status.success(),
        "{}",
        String::from_utf8_lossy(&route.stderr)
    );
    assert!(String::from_utf8_lossy(&route.stdout).contains("20-projects/2026-11-example"));
}

#[test]
#[cfg(unix)]
fn project_relocation_refuses_conflicts_drift_and_protected_link_rewrites() {
    let fixture = RelocationFixture::new("project-relocation-drift");
    let plan = fixture.success("plan", &RelocationFixture::request());
    fixture
        .vault
        .write("20-projects/20-projects.md", "# Existing root");
    assert_eq!(fixture.run("begin", &plan).0.status.code(), Some(2));
    fs::remove_dir_all(fixture.vault.0.join("20-projects")).unwrap();
    fixture.vault.write(
        ".viggobsidian/adoption.lock",
        "owned by another transaction",
    );
    assert_eq!(fixture.run("begin", &plan).0.status.code(), Some(2));
    fs::remove_file(fixture.vault.0.join(".viggobsidian/adoption.lock")).unwrap();
    for name in ["pending.json", "rollback.json"] {
        let path = format!(".viggobsidian/{name}");
        fixture
            .vault
            .write(&path, "unresolved release transaction without a lock");
        assert_eq!(fixture.run("begin", &plan).0.status.code(), Some(2));
        fs::remove_file(fixture.vault.0.join(&path)).unwrap();
    }
    fixture.vault.write("15-projects/extra.md", "unreviewed");
    assert_eq!(fixture.run("begin", &plan).0.status.code(), Some(2));
    fs::remove_file(fixture.vault.0.join("15-projects/extra.md")).unwrap();
    fixture.success("begin", &plan);
    fixture.simulated_obsidian_folder_move();
    fixture.simulated_obsidian_rootnote_rename();
    let schema = fixture
        .vault
        .0
        .join("99-system/contracts/vault-schema.yaml");
    let original_schema = fs::read(&schema).unwrap();
    fixture
        .vault
        .write("99-system/contracts/vault-schema.yaml", "changed");
    assert_eq!(fixture.run("validate", &plan).0.status.code(), Some(2));
    fs::write(schema, original_schema).unwrap();
    let changed = format!("20-projects/{}", RelocationFixture::LEGACY);
    fixture.vault.write(&changed, "[[20-projects/20-projects]]");
    assert_eq!(fixture.run("validate", &plan).0.status.code(), Some(2));
    fixture.vault.write(&changed, RelocationFixture::HISTORICAL);
    fixture
        .vault
        .write("outside/outside.md", "external protected link was changed");
    assert_eq!(fixture.run("recover", &plan).0.status.code(), Some(2));
    assert!(
        fixture
            .vault
            .0
            .join(".viggobsidian/project-relocation.pending.json")
            .exists()
    );
}

#[test]
#[cfg(unix)]
fn project_relocation_refuses_code_or_id_ambiguity_in_another_registered_root() {
    let fixture = RelocationFixture::new("project-relocation-route-conflict");
    let plan = fixture.success("plan", &RelocationFixture::request());
    let path = "21-program/2026-11-other/2026-11-other.md";
    fixture.vault.write(path, "---\ntype: project\nproject-month: \"2026-11\"\nproject-code: EX\nproject_id: distinct-id\n---\n");
    assert_eq!(fixture.run("begin", &plan).0.status.code(), Some(2));
    fixture.vault.write(path, "---\ntype: project\nproject-month: \"2026-11\"\nproject-code: OTHER\nproject_id: stable-project-id\n---\n");
    assert_eq!(
        fixture
            .run("plan", &RelocationFixture::request())
            .0
            .status
            .code(),
        Some(2)
    );
}

#[test]
#[cfg(unix)]
fn project_relocation_refuses_an_unchanged_adoption_image_without_opening_a_hold() {
    let fixture = RelocationFixture::new("project-relocation-no-adoption");
    let path = fixture.vault.0.join(".viggobsidian/adoption.json");
    let mut state: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    state["owners"]
        .as_object_mut()
        .unwrap()
        .retain(|owner, _| owner == "outside");
    fs::write(&path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
    let (out, _) = fixture.run("plan", &RelocationFixture::request());
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no existing adoption owners"));
    assert!(
        !fixture
            .vault
            .0
            .join(".viggobsidian/project-relocation.pending.json")
            .exists()
    );
}

#[test]
#[cfg(unix)]
fn project_relocation_recovers_from_adoption_checkpoint_and_never_rebaselines() {
    let fixture = RelocationFixture::new("project-relocation-recovery");
    let plan = fixture.success("plan", &RelocationFixture::request());
    fixture.success("begin", &plan);
    assert_eq!(fixture.success("abort", &plan)["status"], "aborted-unmoved");
    fixture.success("begin", &plan);
    fixture.simulated_obsidian_folder_move();
    fixture.simulated_obsidian_rootnote_rename();
    // Exact observable state left by interruption after the first durable
    // replacement, before replacing the local integration state.
    fixture.vault.write(
        ".viggobsidian/adoption.json",
        serde_json::to_vec_pretty(&plan["adoption"]["after"]).unwrap(),
    );
    let mut altered = plan.clone();
    altered["adoption"]["after"]["owners"] = json!({});
    assert_eq!(fixture.run("recover", &altered).0.status.code(), Some(2));
    assert_eq!(
        fixture.success("recover", &plan)["status"],
        "adoption-and-integration-rebound"
    );
    assert_eq!(
        fixture.success("recover", &plan)["status"],
        "already-applied"
    );
    let receipts: Vec<_> = fs::read_dir(fixture.vault.0.join(".viggobsidian"))
        .unwrap()
        .filter(|ent| {
            ent.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("project-relocation-")
        })
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(receipts.len(), 1);
    assert_eq!(
        fs::metadata(&receipts[0]).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn legacy_adoption_preserves_three_errors_but_checks_each_added_bubble() {
    let fixture = Fixture::new("legacy-append");
    let path = "topics/legacy/legacy.md";
    let original = "---\ntype: note\ncategory: knowledge\nactivity_contract: provenance-v1\n---\n# Legacy\n\n## Activity\n\n> [!activity-day|2026-11-11] 11 November 2026\n\n> [!activity-message|human] Alex\n> Old source A.\n\n> [!activity-message|human] Blake\n> Old source B.\n\n> [!activity-message|human] Casey\n> Old source C.\n";
    fixture.write(path, original);
    adopt(&fixture, json!({"topics/legacy":"legacy"}));
    let changes = json!([{"path":path,"expect":"present"}]);
    let (_, baseline) = write_check(&fixture, "baseline", changes.clone(), Value::Null);
    assert!(
        baseline["legacy_findings"].as_array().unwrap().len() >= 3,
        "{baseline}"
    );
    fixture.write(path, format!("{original}{}", appended()));
    let (output, report) = write_check(&fixture, "validate", changes.clone(), baseline.clone());
    assert!(output.status.success(), "{report}");
    assert_eq!(fixture.strict("topics/legacy").status.code(), Some(1));
    // Same author and error text: set subtraction alone would miss this error.
    fixture.write(
        path,
        format!(
            "{original}{}",
            appended().replace("> `— · Alex · chat`\n", "")
        ),
    );
    assert_eq!(
        write_check(&fixture, "validate", changes.clone(), baseline.clone())
            .0
            .status
            .code(),
        Some(1)
    );
    fixture.write(
        path,
        format!(
            "{}{}",
            original.replace("Old source A.", "Rewritten source."),
            appended()
        ),
    );
    assert_eq!(
        write_check(&fixture, "validate", changes.clone(), baseline.clone())
            .0
            .status
            .code(),
        Some(1)
    );
    fixture.write(path, format!("{original}{}", appended()));
    // Config drift, including disappearance, is an execution failure.
    let state = fs::read(fixture.0.join(".viggobsidian/adoption.json")).unwrap();
    fs::remove_file(fixture.0.join(".viggobsidian/adoption.json")).unwrap();
    assert_eq!(
        write_check(&fixture, "validate", changes.clone(), baseline)
            .0
            .status
            .code(),
        Some(2)
    );
    fixture.write(".viggobsidian/adoption.json", state);
    // A new source never borrows an old source's exemption.
    let fresh = "topics/legacy/new/new.md";
    let creation = json!([{"path":fresh,"expect":"present"}]);
    let (_, baseline) = write_check(&fixture, "baseline", creation.clone(), Value::Null);
    fixture.write(fresh, original);
    assert_eq!(
        write_check(&fixture, "validate", creation, baseline)
            .0
            .status
            .code(),
        Some(1)
    );
}

#[test]
fn adoption_promotes_only_clean_owners_and_never_downgrades_strict_children() {
    let fixture = Fixture::new("legacy-promotion");
    fixture.write("HOME.md", "---\ntype: note\ncategory: index\n---\n# Home\n");
    fixture.write(
        "topics/topics.md",
        nav(
            &fixture,
            "topics",
            "HOME",
            "> [!navigation]\n> - [[topics/child/child|Child]]\n",
        ),
    );
    let child = "topics/child/child.md";
    let good = nav(&fixture, "topics/child", "topics/topics", "");
    fixture.write(child, &good);
    adopt(&fixture, json!({"topics":"legacy","topics/child":"strict"}));
    let changes = json!([{"path":child,"expect":"present"}]);
    let (_, baseline) = write_check(&fixture, "baseline", changes.clone(), Value::Null);
    fixture.write(child, "Broken former strict entrance\n");
    let (output, report) = write_check(&fixture, "validate", changes.clone(), baseline);
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(report["legacy_findings"].as_array().unwrap().is_empty());
    let request =
        json!({"schema":"viggobsidian.adoption-request.v1","owners":{"topics/child":"legacy"}});
    assert_eq!(
        adoption_command(&fixture, "--adoption-plan", &request)
            .0
            .status
            .code(),
        Some(2)
    );
    let request = json!({"schema":"viggobsidian.adoption-request.v1","owners":{"topics":"strict"}});
    assert_eq!(
        adoption_command(&fixture, "--adoption-plan", &request)
            .0
            .status
            .code(),
        Some(2)
    );
    fixture.write(child, &good);
    adopt(&fixture, json!({"topics":"strict"}));
    // State is no longer legacy even if note metadata is subsequently removed.
    fixture.write(child, "No metadata\n");
    let (_, baseline) = write_check(&fixture, "baseline", changes.clone(), Value::Null);
    assert_eq!(
        write_check(&fixture, "validate", changes, baseline)
            .0
            .status
            .code(),
        Some(1)
    );
}

#[test]
fn adoption_refuses_stale_plans_and_symlinks_without_rebaselining() {
    let fixture = Fixture::new("adoption-drift");
    fixture.write(
        "topics/old/old.md",
        "---\ntype: note\ncategory: source\n---\nOld input\n",
    );
    let request =
        json!({"schema":"viggobsidian.adoption-request.v1","owners":{"topics/old":"legacy"}});
    let (_, plan) = adoption_command(&fixture, "--adoption-plan", &request);
    fixture.write("topics/old/old.md", "changed\n");
    assert_eq!(
        adoption_command(&fixture, "--adoption-apply", &plan)
            .0
            .status
            .code(),
        Some(2)
    );
    adopt(&fixture, json!({"topics/old":"legacy"}));
    assert_eq!(
        adoption_command(&fixture, "--adoption-plan", &request)
            .0
            .status
            .code(),
        Some(2)
    );
    let state = fixture.0.join(".viggobsidian/adoption.json");
    fs::remove_file(&state).unwrap();
    symlink(fixture.0.join("topics/old/old.md"), &state).unwrap();
    assert_eq!(
        write_check(
            &fixture,
            "baseline",
            json!([{"path":"topics/old/old.md","expect":"present"}]),
            Value::Null
        )
        .0
        .status
        .code(),
        Some(2)
    );
}

#[test]
fn adoption_recovers_exact_interruption_and_replays_without_resetting_history() {
    let fixture = Fixture::new("adoption-recovery");
    fixture.write(
        "topics/old/old.md",
        "---\ntype: note\ncategory: source\n---\nOriginal.\n",
    );
    let request =
        json!({"schema":"viggobsidian.adoption-request.v1","owners":{"topics/old":"legacy"}});
    let (_, plan) = adoption_command(&fixture, "--adoption-plan", &request);
    let bytes = serde_json::to_vec_pretty(&plan["next"]).unwrap();
    let receipt = format!(".viggobsidian/adoption-{:x}.json", Sha256::digest(&bytes));
    fixture.write(
        &receipt,
        serde_json::to_vec(&json!({"plan":plan,"previous":null})).unwrap(),
    );
    fixture.write(
        ".viggobsidian/adoption.lock",
        format!("{:x}", Sha256::digest(serde_json::to_vec(&plan).unwrap())),
    );
    fixture.write(".viggobsidian/adoption.pending", b"foreign bytes");
    assert_eq!(
        adoption_command(&fixture, "--adoption-recover", &plan)
            .0
            .status
            .code(),
        Some(2)
    );
    assert!(fixture.0.join(".viggobsidian/adoption.lock").exists());
    fixture.write(".viggobsidian/adoption.pending", bytes);
    let (output, report) = adoption_command(&fixture, "--adoption-recover", &plan);
    assert!(
        output.status.success(),
        "{} {report}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.0.join(".viggobsidian/adoption.lock").exists());
    assert!(!fixture.0.join(".viggobsidian/adoption.pending").exists());
    let (output, report) = adoption_command(&fixture, "--adoption-apply", &plan);
    assert!(output.status.success(), "{report}");
    assert_eq!(report["status"], "already-applied");
}

#[test]
fn strict_parent_and_legacy_child_cannot_be_created_in_one_plan() {
    let fixture = Fixture::new("adoption-mixed-request");
    fixture.write("HOME.md", "---\ntype: note\ncategory: index\n---\n# Home\n");
    fixture.write(
        "topics/topics.md",
        nav(
            &fixture,
            "topics",
            "HOME",
            "> [!navigation]\n> - [[topics/child/child|Child]]\n",
        ),
    );
    fixture.write(
        "topics/child/child.md",
        nav(&fixture, "topics/child", "topics/topics", ""),
    );
    let request = json!({"schema":"viggobsidian.adoption-request.v1","owners":{"topics":"strict","topics/child":"legacy"}});
    assert_eq!(
        adoption_command(&fixture, "--adoption-plan", &request)
            .0
            .status
            .code(),
        Some(2)
    );
    adopt(&fixture, json!({"topics":"legacy"}));
    assert_eq!(
        adoption_command(&fixture, "--adoption-plan", &request)
            .0
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn historical_identity_mismatch_and_new_chronology_regression_remain_blocking() {
    let fixture = Fixture::new("adoption-integrity");
    let path = "topics/old/old.md";
    let original = "---\ntype: note\ncategory: source\nactivity_contract: provenance-v1\n---\n# Source\n\n## Activity\n\n> [!activity-day|2026-11-12] 12 November 2026\n\n> [!activity-message|human] Alex\n> `— · Blake · chat`\n> Original.\n";
    fixture.write(path, original);
    adopt(&fixture, json!({"topics/old":"legacy"}));
    let changes = json!([{"path":path,"expect":"present"}]);
    let (_, baseline) = write_check(&fixture, "baseline", changes.clone(), Value::Null);
    assert_eq!(
        write_check(&fixture, "validate", changes, baseline)
            .0
            .status
            .code(),
        Some(1)
    );
    // Fresh fixture with a missing stamp: append cannot go back before history.
    let other = Fixture::new("adoption-chronology");
    let original = original.replace("> `— · Blake · chat`\n", "");
    other.write(path, &original);
    adopt(&other, json!({"topics/old":"legacy"}));
    let changes = json!([{"path":path,"expect":"present"}]);
    let (_, baseline) = write_check(&other, "baseline", changes.clone(), Value::Null);
    other.write(
        path,
        format!(
            "{original}{}",
            appended().replace("2026-11-12", "2026-11-10")
        ),
    );
    assert_eq!(
        write_check(&other, "validate", changes, baseline)
            .0
            .status
            .code(),
        Some(1)
    );
}

fn write_check(fixture: &Fixture, phase: &str, changes: Value, baseline: Value) -> (Output, Value) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
        .arg("--vault")
        .arg(&fixture.0)
        .arg("--write-set")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    serde_json::to_writer(child.stdin.take().unwrap(), &json!({"schema":"viggobsidian.write-set.v1","phase":phase,"changes":changes,"baseline":baseline})).unwrap();
    let output = child.wait_with_output().unwrap();
    let report = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output, report)
}

fn nav(fixture: &Fixture, folder: &str, back: &str, children: &str) -> String {
    format!(
        "---\ntype: collection\ncategory: index\nlayer: navigation\n---\n# Navigation\n\n> [!navigation|shortcuts]\n> - [[{back}|Back]]\n> - [Finder](file://{}/{folder})\n\n{children}",
        fixture.0.display()
    )
}

#[test]
fn direct_dossier_changes_cover_raw_dependencies_and_mail_manifests() {
    let fixture = Fixture::new("write-set-direct-wiki");
    fixture.write(
        "topics/dossier/dossier.md",
        "---\ntype: note\ncategory: knowledge\nlayer: wiki\n---\n# Wiki\n",
    );
    fixture.write("topics/dossier/raw/raw.md", "Invalid Raw entrance");
    fixture.write(
        "topics/dossier/raw/20261112-source/20261112-source.md",
        "Invalid source",
    );
    let changes = json!([{"path":"topics/dossier/dossier.md","expect":"present"},
        {"path":"topics/dossier/raw/20261112-source/.mail-source.json","expect":"present"}]);
    let (output, baseline) = write_check(&fixture, "baseline", changes.clone(), Value::Null);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(baseline["owned_scopes"], json!({"topics/dossier":true}));
    assert!(
        baseline["owned_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["path"].as_str().unwrap().contains("20261112-source"))
    );
    let (output, _) = write_check(&fixture, "validate", changes, baseline);
    assert_eq!(output.status.code(), Some(1));
    let (_, direct) = write_check(
        &fixture,
        "baseline",
        json!([{"path":"topics/dossier/dossier.md","expect":"present"}]),
        Value::Null,
    );
    assert_eq!(direct["owned_scopes"], json!({"topics/dossier":true}));
}

#[test]
fn exact_write_set_ignores_finance_but_rejects_changed_health_and_boundary_regressions() {
    let fixture = Fixture::new("write-set-health");
    fixture.write("HOME.md", "---\ntype: note\ncategory: index\n---\n# Home");
    fixture.write("topics/topics.md", nav(&fixture,"topics","HOME","> [!navigation]\n> - [[topics/health/health|Health]]\n> - [[topics/finance/finance|Finance]]\n"));
    let health = nav(&fixture, "topics/health", "topics/topics", "");
    fixture.write("topics/health/health.md", &health);
    fixture.write(
        "topics/finance/finance.md",
        "Unrelated invalid legacy content.",
    );
    let changes = json!([{"path":"topics/health/health.md","expect":"present"}]);
    let (output, baseline) = write_check(&fixture, "baseline", changes.clone(), Value::Null);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    fixture.write(
        "topics/health/health.md",
        health.replace("# Navigation", "# Updated Health"),
    );
    let (output, report) = write_check(&fixture, "validate", changes.clone(), baseline.clone());
    assert!(output.status.success(), "{report}");
    assert!(!report.to_string().contains("topics/finance/finance.md"));
    assert_eq!(fixture.strict("topics").status.code(), Some(1));
    fixture.write("topics/health/health.md", "Invalid changed Health.");
    let (output, report) = write_check(&fixture, "validate", changes.clone(), baseline.clone());
    assert_eq!(output.status.code(), Some(1), "{report}");
    fixture.write("topics/health/health.md", &health);
    fixture.write(
        "topics/extra/extra.md",
        nav(&fixture, "topics/extra", "topics/topics", ""),
    );
    let (output, report) = write_check(&fixture, "validate", changes, baseline);
    assert_eq!(
        output.status.code(),
        Some(1),
        "new missing parent tile must fail: {report}"
    );
}

#[test]
fn typed_handoff_preserves_old_owner_but_checks_destination_and_sibling_bytes() {
    let f = Fixture::new("typed-handoff");
    f.write("HOME.md", "---\ntype: note\ncategory: index\n---\n# Home");
    f.write(
        "topics/topics.md",
        nav(
            &f,
            "topics",
            "HOME",
            "> [!navigation]\n> - [[topics/old/old|Old]]\n> - [[topics/new/new|New]]\n",
        ),
    );
    let old =
        "---\ntype: reference\ncategory: legacy\n---\n\n# Old\n\nOriginal historical prose.\n";
    f.write("topics/old/old.md", old);
    f.write("topics/old/history.txt", "Immutable sibling.");
    f.write(
        "topics/new/new.md",
        nav(&f, "topics/new", "topics/topics", ""),
    );
    let proof = json!({"kind":"owner-handoff","before":old,"target":"topics/new/new.md"});
    let changes = json!([{"path":"topics/old/old.md","expect":"present","proof":proof}]);
    let (out, baseline) = write_check(&f, "baseline", changes.clone(), Value::Null);
    assert!(out.status.success(), "{baseline}");
    let (fm, body) = old.split_once("\n---\n").unwrap();
    let after = format!(
        "{fm}\n---\n\n<!-- viggobsidian:knowledge-owner -->\n> [!info] Actuele kennis\n> [[topics/new/new|Open actuele kennis]]\n<!-- /viggobsidian:knowledge-owner -->\n{body}"
    );
    f.write("topics/old/old.md", &after);
    let (out, r) = write_check(&f, "validate", changes.clone(), baseline.clone());
    assert!(out.status.success(), "{r}");
    f.write("topics/old/old.md", old);
    let mut mixed = changes.clone();
    mixed
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"topics/old/history.txt","expect":"present"}));
    let (_, mixed_baseline) = write_check(&f, "baseline", mixed.clone(), Value::Null);
    f.write("topics/old/old.md", &after);
    f.write("topics/old/history.txt", "Rewritten ordinary file.");
    let (out, _) = write_check(&f, "validate", mixed, mixed_baseline);
    assert!(
        !out.status.success(),
        "changed nonproof files must remain strict even with an identical old finding"
    );
    f.write("topics/old/history.txt", "Immutable sibling.");
    f.write("topics/old/history.txt", "Foreign edit.");
    let (out, _) = write_check(&f, "validate", changes.clone(), baseline.clone());
    assert!(!out.status.success());
    f.write("topics/old/history.txt", "Immutable sibling.");
    f.write("topics/new/new.md", "Broken destination.");
    let (out, _) = write_check(&f, "validate", changes, baseline);
    assert!(!out.status.success());
}

#[test]
fn verified_link_repair_requires_paired_archive_and_retains_only_exact_existing_debt() {
    let f = Fixture::new("verified-source-link");
    f.write("HOME.md", "---\ntype: note\ncategory: index\n---\n# Home\n");
    f.write(
        "topics/topics.md",
        nav(
            &f,
            "topics",
            "HOME",
            "> [!navigation]\n> - [[topics/old/old|Old]]\n",
        ),
    );
    let owner = "topics/old/old.md";
    let archive = "topics/old/before.md.txt";
    let before = "---\ntype: reference\ncategory: legacy\n---\n# Historical commands\n\n> Human example [[old-note]].\n";
    f.write(owner, before);
    f.write("topics/old/history.txt", "Preserve sibling bytes.");
    f.write("external/source.md", "# Source\nReal source bytes.\n");
    let root = f.0.canonicalize().unwrap();
    let target = root.join("external/source.md");
    use sha2::{Digest, Sha256};
    let digest = format!("{:x}", Sha256::digest(fs::read(&target).unwrap()));
    let uri = format!("file://{}", target.display());
    let proof = json!({"kind":"verified-link-repair","before":before,"archive":archive,"mapping":{"old-note":{"kind":"file","root":root.join("external"),"uri":uri,"sha256":digest}}});
    let archive_proof = json!({"kind":"source-preimage-archive","before":null,"source":owner,"source_before":before});
    let changes = json!([{"path":owner,"expect":"present","proof":proof},{"path":archive,"expect":"present","proof":archive_proof}]);
    let (out, baseline) = write_check(&f, "baseline", changes.clone(), Value::Null);
    assert!(
        out.status.success(),
        "{} {baseline}",
        String::from_utf8_lossy(&out.stderr)
    );
    let after = before.replace("[[old-note]]", &format!("[old-note]({uri})"));
    f.write(owner, &after);
    f.write(archive, before);
    let (out, report) = write_check(&f, "validate", changes.clone(), baseline.clone());
    assert!(
        out.status.success(),
        "{} {report}",
        String::from_utf8_lossy(&out.stderr)
    );
    f.write(archive, "Foreign source");
    assert!(
        !write_check(&f, "validate", changes.clone(), baseline.clone())
            .0
            .status
            .success()
    );
    f.write(archive, before);
    f.write("topics/old/history.txt", "Foreign sibling edit.");
    assert!(
        !write_check(&f, "validate", changes.clone(), baseline.clone())
            .0
            .status
            .success()
    );
    f.write("topics/old/history.txt", "Preserve sibling bytes.");
    f.write("external/source.md", "Changed source target.");
    assert!(
        !write_check(&f, "validate", changes.clone(), baseline.clone())
            .0
            .status
            .success()
    );
    f.write("external/source.md", "# Source\nReal source bytes.\n");
    let mut orphan = changes.clone();
    orphan.as_array_mut().unwrap().remove(1);
    assert!(
        !write_check(&f, "validate", orphan, baseline.clone())
            .0
            .status
            .success()
    );
    let mut injected = after;
    injected.push_str("\nUnproved owner rewrite.\n");
    f.write(owner, injected);
    assert!(
        !write_check(&f, "validate", changes, baseline)
            .0
            .status
            .success()
    );
}

#[test]
fn managed_wiki_section_pairs_archive_preserves_siblings_and_keeps_new_findings_strict() {
    let f = Fixture::new("managed-wiki-section");
    f.write(
        "99-system/contracts/navigation-scope.yaml",
        "technical: []\nreadme_collections: []\n",
    );
    f.write(
        ".viggobsidian/core/navigation-contract.yaml",
        include_bytes!("../fixtures/navigation-contract.yaml"),
    );
    f.write(
        "99-system/vault/vault-navigation/standaard/wiki/vault-raw.base",
        include_bytes!("../fixtures/vault-raw.base"),
    );
    f.write(
        "99-system/vault/vault-navigation/standaard/wiki/vault-dossier.base",
        include_bytes!("../fixtures/vault-dossier.base"),
    );
    f.write(
        ".obsidian/community-plugins.json",
        "[\"vault-raw-summary\"]",
    );
    for name in ["manifest.json", "main.js", "styles.css"] {
        f.write(
            &format!(".obsidian/plugins/vault-raw-summary/{name}"),
            "fictional structural fixture only",
        );
    }
    f.write("HOME.md", "---\ntype: note\ncategory: index\n---\n# Home\n");
    f.write(
        "topics/topics.md",
        nav(
            &f,
            "topics",
            "HOME",
            "> [!navigation]\n> - [[topics/example/example|Example]]\n",
        ),
    );
    f.write(
        "topics/example/example.md",
        format!("---\ntype: dossier\ncategory: knowledge\nlayer: wiki\ncssclasses: [folder-note, vault-navigation, vault-dashboard, dashboard-option-2, dashboard-variant-2-2, vault-dossier-dashboard]\n---\n\n# Example\n\n> [!navigation|shortcuts]\n> - [[topics/topics|Back]]\n> - [Finder](file://{}/topics/example)\n\n> [!navigation]\n> - [[topics/example/wiki/wiki|Wiki]]\n> - [[topics/example/raw/raw|Raw]]\n\n```vault-task-summary\n```\n\n![[vault-dossier.base#Open tasks]]\n",f.0.display()),
    );
    f.write(
        "topics/example/raw/raw.md",
        format!("---\ntype: collection\ncategory: index\nlayer: navigation\ncssclasses: [folder-note, vault-navigation, vault-dashboard, dashboard-option-2, dashboard-variant-2-2, raw-hub-table]\n---\n\n# Raw\n\n> [!navigation|shortcuts]\n> - [[topics/example/example|Back]]\n> - [[topics/example/wiki/wiki|Wiki]]\n> - [Finder](file://{}/topics/example/raw)\n\n```vault-raw-summary\n```\n\n![[vault-raw.base#Latest sources]]\n",f.0.display()),
    );
    f.write("topics/example/raw/20261117-report/20261117-report.md","---\ntype: note\ncategory: source\nwiki_status: pending\n---\n# Fictional retained source\n");
    f.write(".obsidian/snippets/folder-note.css",".vault-dashboard a[data-href$=\"/raw/raw\"] { --dashboard-accent: blue; --dashboard-icon: raw-icon; }\n.vault-dashboard a[data-href$=\"/wiki/wiki\"] { --dashboard-accent: green; --dashboard-icon: wiki-icon; }\n");
    let owner = "topics/example/wiki/wiki.md";
    let archive = "topics/example/raw/20261117-report/_source-assets/before.md.txt";
    let before = "---\ntype: note\ncategory: knowledge\nlayer: wiki\ncssclasses: [vault-navigation, vault-dashboard, vault-wiki]\n---\n\n# Example\n\n> [!navigation|shortcuts]\n> - [[topics/example/example|Back]]\n> - [[topics/example/raw/raw|Raw]]\n> - [Finder](file:///different-host/topics/example/wiki)\n\n<!-- report:v1 begin -->\n## Daily report\n\nOld result.\n<!-- report:v1 end -->\n\n## Retained history\n\nHistory stays.\n";
    f.write(owner, before);
    let proof = json!({"kind":"managed-wiki-section","before":before,"archive":archive,"marker":"report:v1","replacement":"\n## Daily report\n\nNew result.\n"});
    let archive_proof = json!({"kind":"source-preimage-archive","before":null,"source":owner,"source_before":before});
    let changes = json!([{"path":owner,"expect":"present","proof":proof},{"path":archive,"expect":"present","proof":archive_proof}]);
    let (out, baseline) = write_check(&f, "baseline", changes.clone(), Value::Null);
    assert!(out.status.success(), "{baseline}");
    let after = before.replace("Old result.", "New result.");
    f.write(owner, &after);
    f.write(archive, before);
    let (out, report) = write_check(&f, "validate", changes.clone(), baseline.clone());
    assert!(out.status.success(), "{report}");
    assert_eq!(report["links_checked"], true);
    assert_eq!(report["link_report"]["checked_paths"], json!([owner]));
    assert_eq!(
        report["link_report"]["legacy_issues"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let mut lost_issue = baseline.clone();
    lost_issue["link_report"]["issues"] = json!([]);
    assert!(
        !write_check(&f, "validate", changes.clone(), lost_issue)
            .0
            .status
            .success()
    );
    f.write("99-system/contracts/navigation-scope.yaml",format!("technical:\n  - owner: HOME.md\n    paths: [{owner}]\n    reason: fictional exclusion\nreadme_collections: []\n"));
    let (excluded, excluded_report) =
        write_check(&f, "validate", changes.clone(), baseline.clone());
    assert!(!excluded.status.success());
    assert_eq!(excluded_report["links_checked"], false);
    f.write(
        "99-system/contracts/navigation-scope.yaml",
        "technical: []\nreadme_collections: []\n",
    );
    let mut no_link_baseline = baseline.clone();
    no_link_baseline
        .as_object_mut()
        .unwrap()
        .remove("link_report");
    assert!(
        !write_check(&f, "validate", changes.clone(), no_link_baseline)
            .0
            .status
            .success()
    );
    let mut repeated = changes.clone();
    repeated[0]["proof"]["replacement"] = json!(
        "\n## Daily report\n\nNew result. [New broken link](file:///different-host/topics/example/wiki)\n"
    );
    f.write(owner, before);
    fs::remove_file(f.0.join(archive)).unwrap();
    let (_, repeat_baseline) = write_check(&f, "baseline", repeated.clone(), Value::Null);
    f.write(
        owner,
        before.replace(
            "Old result.",
            "New result. [New broken link](file:///different-host/topics/example/wiki)",
        ),
    );
    f.write(archive, before);
    assert!(
        !write_check(&f, "validate", repeated, repeat_baseline)
            .0
            .status
            .success()
    );
    f.write(owner, &after);
    assert!(
        report["legacy_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["path"] == owner && r["code"] == "dossier-dashboard-finder")
    );
    f.write(owner, after.clone() + "\nUnproved suffix rewrite.\n");
    assert!(
        !write_check(&f, "validate", changes.clone(), baseline.clone())
            .0
            .status
            .success()
    );
    f.write(owner, &after);
    f.write(archive, "Changed archive");
    assert!(
        !write_check(&f, "validate", changes.clone(), baseline.clone())
            .0
            .status
            .success()
    );
    f.write(archive, before);
    let sibling = "topics/example/raw/20261117-report/20261117-report.md";
    let original = fs::read(f.0.join(sibling)).unwrap();
    f.write(sibling, "Foreign changed source");
    assert!(
        !write_check(&f, "validate", changes.clone(), baseline.clone())
            .0
            .status
            .success()
    );
    f.write(sibling, original);
    let mut orphan = changes.clone();
    orphan.as_array_mut().unwrap().pop();
    assert!(
        !write_check(&f, "validate", orphan, baseline.clone())
            .0
            .status
            .success()
    );
    let bad_before = before.replace(
        "Old result.",
        "Old result.\n\n> [!wiki-facts]\n> Misplaced facts.",
    );
    f.write(owner, &bad_before);
    fs::remove_file(f.0.join(archive)).unwrap();
    let mut bad_changes = changes.clone();
    bad_changes[0]["proof"]["before"] = json!(bad_before);
    bad_changes[1]["proof"]["source_before"] = json!(bad_before);
    bad_changes[0]["proof"]["replacement"] =
        json!("\n## Daily report\n\nNew result.\n\n> [!wiki-facts]\n> Misplaced facts.\n");
    let (_, bad_baseline) = write_check(&f, "baseline", bad_changes.clone(), Value::Null);
    f.write(owner, bad_before.replace("Old result.", "New result."));
    f.write(archive, &bad_before);
    assert!(
        !write_check(&f, "validate", bad_changes, bad_baseline)
            .0
            .status
            .success(),
        "a new replacement cannot inherit non-navigation findings by matching their old code"
    );
    f.write(owner, before);
    fs::remove_file(f.0.join(archive)).unwrap();
    let new_raw = "topics/example/raw/20261118-source/20261118-source.md";
    let mut new_changes = changes.clone();
    new_changes
        .as_array_mut()
        .unwrap()
        .push(json!({"path":new_raw,"expect":"present"}));
    let (_, new_baseline) = write_check(&f, "baseline", new_changes.clone(), Value::Null);
    f.write(owner, &after);
    f.write(archive, before);
    assert!(
        !write_check(&f, "validate", new_changes.clone(), new_baseline.clone())
            .0
            .status
            .success(),
        "new declared Markdown cannot be absent"
    );
    f.write(new_raw,"---\ntype: note\ncategory: source\nwiki_status: pending\n---\n# Fictional new source\n\n[[missing-target]]\n");
    let (broken, broken_report) = write_check(&f, "validate", new_changes, new_baseline);
    assert!(!broken.status.success());
    assert!(
        broken_report["link_report"]["checked_paths"]
            .as_array()
            .unwrap()
            .contains(&json!(new_raw))
    );
    assert!(
        broken_report["link_report"]["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["path"] == new_raw && row["kind"] == "broken-wikilink")
    );
}

#[test]
fn typed_locator_destinations_bind_to_the_selected_vault_without_waiving_legacy_checks() {
    let f = Fixture::new("typed-locator-destinations");
    f.write("HOME.md", "---\ntype: note\ncategory: index\n---\n# Home");
    f.write(
        "topics/topics.md",
        nav(
            &f,
            "topics",
            "HOME",
            "> [!navigation]\n> - [[topics/old/old|Old]]\n> - [[topics/new/new|New]]\n",
        ),
    );
    f.write(
        "topics/new/new.md",
        nav(&f, "topics/new", "topics/topics", ""),
    );
    f.write("topics/old/history.txt", "Immutable legacy sibling.");
    let root = f.0.canonicalize().unwrap();
    let vault_name = root.file_name().unwrap().to_str().unwrap();
    // Absolute file URIs reject hidden components, including this shared .tmp
    // fixture root. The pure destination test covers their positive binding.
    let encoded_name = vault_name.replace('-', "%2D");
    let obsidian_uri = format!("obsidian://open?vault={encoded_name}&file=topics%2Fnew%2Fnew");
    let old_file = "file:///unavailable/original.md";
    let old_obsidian = "obsidian://open?vault=previous&file=old/source.md";
    let unavailable = "file:///unavailable/history.md";
    let before = format!(
        "---\ntype: reference\ncategory: legacy\nexternal-tasks:\n- path: topics/previous/task.md\n  task-id: task:fictional\nstatus: done\n---\n\n# Old\n\n[[old-note]]\n[File]({old_file})\n[Obsidian]({old_obsidian})\n[History]({unavailable})\n"
    );
    let proof = json!({"kind":"locator-rebind","before":before,"mapping":{
        "old-note":"topics/new/new.md", "topics/previous/task.md":"topics/new/new.md",
        old_file:obsidian_uri, old_obsidian:obsidian_uri, unavailable:null
    }});
    let changes = json!([{"path":"topics/old/old.md","expect":"present","proof":proof}]);
    f.write("topics/old/old.md", &before);
    // A planned extensionless URI may name a note created after baseline.
    fs::remove_file(f.0.join("topics/new/new.md")).unwrap();
    let (out, baseline) = write_check(&f, "baseline", changes.clone(), Value::Null);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(baseline["changes"], changes);
    assert!(!baseline["owned_findings"].as_array().unwrap().is_empty());
    f.write(
        "topics/new/new.md",
        nav(&f, "topics/new", "topics/topics", ""),
    );
    let after = before.replace("path: topics/previous/task.md", "path: topics/new/new.md")
        .replace("[[old-note]]", "[[topics/new/new]]")
        .replace(old_file, &obsidian_uri).replace(old_obsidian, &obsidian_uri)
        .replace(&format!("[History]({unavailable})"), &format!("History (historische verwijzing; oorspronkelijke lokale bron niet teruggevonden: `{unavailable}`)"));
    assert!(after.contains("task-id: task:fictional\nstatus: done"));
    f.write("topics/old/old.md", &after);
    let (out, report) = write_check(&f, "validate", changes.clone(), baseline.clone());
    assert!(
        out.status.success(),
        "{report}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(report["legacy_findings"], baseline["owned_findings"]);

    // An exact proof and unchanged sibling inventory are still mandatory.
    let mut altered = changes.clone();
    altered[0]["proof"]["mapping"][unavailable] = json!(obsidian_uri);
    f.write("topics/old/old.md", after.replace(&format!("History (historische verwijzing; oorspronkelijke lokale bron niet teruggevonden: `{unavailable}`)"), &format!("[History]({obsidian_uri})")));
    let (out, _) = write_check(&f, "validate", altered, baseline.clone());
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("exact changes"));
    f.write("topics/old/old.md", &after);
    f.write("topics/old/history.txt", "Foreign edit.");
    assert!(
        !write_check(&f, "validate", changes.clone(), baseline.clone())
            .0
            .status
            .success()
    );
    f.write("topics/old/history.txt", "Immutable legacy sibling.");
    fs::remove_file(f.0.join("topics/new/new.md")).unwrap();
    let (out, _) = write_check(&f, "validate", changes.clone(), baseline.clone());
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("missing target"));
    f.write("topics/new/new.md", "<!-- viggobsidian:knowledge-owner -->");
    let (out, _) = write_check(&f, "validate", changes, baseline);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("handoff cycle"));

    for target in [
        "obsidian://open?vault=another-vault&file=topics%2Fnew%2Fnew.md".to_owned(),
        format!("file://{}/../outside.md", root.display()),
        format!("file://{}-other/topics/new/new.md", root.display()),
        format!("obsidian://open?vault={vault_name}&file=topics%2F..%2Foutside.md"),
    ] {
        let proof = json!({"kind":"locator-rebind","before":before,"mapping":{old_file:target}});
        let changes = json!([{"path":"topics/old/old.md","expect":"present","proof":proof}]);
        f.write("topics/old/old.md", &before);
        let (out, _) = write_check(&f, "baseline", changes, Value::Null);
        assert!(!out.status.success(), "unbound target accepted: {target}");
    }
    // A URI cannot bypass the ordinary symlink guard either.
    fs::remove_file(f.0.join("topics/new/new.md")).unwrap();
    symlink(f.0.join("topics/old/old.md"), f.0.join("topics/new/new.md")).unwrap();
    f.write("topics/old/old.md", &before);
    let (out, _) = write_check(
        &f,
        "baseline",
        json!([{"path":"topics/old/old.md","expect":"present","proof":proof}]),
        Value::Null,
    );
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("symbolic link"));
}

#[test]
fn typed_root_base_links_preserve_legacy_bytes_and_require_a_regular_target() {
    let f = Fixture::new("typed-root-base-links");
    f.write("HOME.md", "---\ntype: note\ncategory: index\n---\n# Home");
    f.write("topics/topics.md", nav(&f, "topics", "HOME", ""));
    let owner = "topics/legacy/legacy.md";
    let before = "---\ntype: collection\ncategory: index\n---\n\n# Legacy\n\n[[previous.base|Overview]] and [[previous.base]].\n\n`previous.base`\n\n> Original previous.base and [[previous.base]].\n";
    let after = before.replacen(
        "[[previous.base|Overview]] and [[previous.base]]",
        "[[OVERVIEW.base|Overview]] and [[OVERVIEW.base]]",
        1,
    );
    f.write(owner, before);
    f.write("OVERVIEW.base", "views: []\n");
    let (out, _) = write_check(
        &f,
        "baseline",
        json!([{"path":"OVERVIEW.base","expect":"present"}]),
        Value::Null,
    );
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("write-set paths"));
    let changes = json!([{"path":owner,"expect":"present","proof":{"kind":"locator-rebind","before":before,"mapping":{"previous.base":"OVERVIEW.base"}}}]);
    let (out, baseline) = write_check(&f, "baseline", changes.clone(), Value::Null);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    f.write(owner, &after);
    let (out, report) = write_check(&f, "validate", changes.clone(), baseline.clone());
    assert!(
        out.status.success(),
        "{report}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(report["legacy_findings"], baseline["owned_findings"]);
    fs::remove_file(f.0.join("OVERVIEW.base")).unwrap();
    let (out, _) = write_check(&f, "validate", changes.clone(), baseline.clone());
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("missing target"));
    fs::create_dir(f.0.join("OVERVIEW.base")).unwrap();
    assert!(
        !write_check(&f, "validate", changes.clone(), baseline.clone())
            .0
            .status
            .success()
    );
    fs::remove_dir(f.0.join("OVERVIEW.base")).unwrap();
    symlink(f.0.join(owner), f.0.join("OVERVIEW.base")).unwrap();
    let (out, _) = write_check(&f, "validate", changes, baseline);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("symbolic link"));
}

#[test]
fn navigation_has_one_grid_and_respects_declared_dates() {
    let f = Fixture::new("navigation-grid-date");
    f.write("HOME.md", "---\ntype: note\ncategory: index\n---\n# Home");
    let child = nav(&f, "topics/child", "topics/topics", "").replacen(
        "\n---\n",
        "\nnavigation_date: '2024-03'\n---\n",
        1,
    );
    f.write("topics/child/child.md", child);
    f.write("topics/topics.md",nav(&f,"topics","HOME","> [!navigation]\n> - [[topics/child/child|Child]]\n\n> [!navigation]\n> - [[topics/child/child|Child]]\n").replacen("\n---\n","\ncssclasses: [vault-dashboard]\n---\n",1));
    let r: Value = serde_json::from_slice(&f.strict("topics").stdout).unwrap();
    for code in ["navigation-grid", "navigation-date"] {
        assert!(
            r["findings"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["code"] == code),
            "{r}"
        );
    }
    f.write(
        "topics/topics.md",
        nav(
            &f,
            "topics",
            "HOME",
            "> [!navigation]\n> - [[topics/child/child|2024-03 · Child]]\n",
        ),
    );
    let r: Value = serde_json::from_slice(&f.strict("topics").stdout).unwrap();
    assert!(
        !r["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["code"] == "navigation-grid" || v["code"] == "navigation-date"),
        "{r}"
    );
}

#[test]
fn exact_write_set_creation_and_deletion_require_parent_navigation() {
    let fixture = Fixture::new("write-set-create");
    fixture.write("HOME.md", "---\ntype: note\ncategory: index\n---\n# Home");
    fixture.write(
        "topics/topics.md",
        nav(
            &fixture,
            "topics",
            "HOME",
            "> [!navigation]\n> - [[topics/health/health|Health]]\n",
        ),
    );
    let health = nav(&fixture, "topics/health", "topics/topics", "");
    fixture.write("topics/health/health.md", &health);
    let child = "topics/health/new/new.md";
    let changes = json!([{"path":child,"expect":"present"},{"path":"topics/health/health.md","expect":"present"}]);
    let (_, baseline) = write_check(&fixture, "baseline", changes.clone(), Value::Null);
    fixture.write(
        child,
        nav(&fixture, "topics/health/new", "topics/health/health", ""),
    );
    assert_eq!(
        write_check(&fixture, "validate", changes.clone(), baseline.clone())
            .0
            .status
            .code(),
        Some(1)
    );
    fixture.write(
        "topics/health/health.md",
        format!("{health}> [!navigation]\n> - [[topics/health/new/new|New]]\n"),
    );
    let (output, report) = write_check(&fixture, "validate", changes, baseline);
    assert!(output.status.success(), "{report}");
    let deletion = json!([{"path":child,"expect":"absent"},{"path":"topics/health/health.md","expect":"present"}]);
    let (_, baseline) = write_check(&fixture, "baseline", deletion.clone(), Value::Null);
    fs::remove_dir_all(fixture.0.join("topics/health/new")).unwrap();
    assert_eq!(
        write_check(&fixture, "validate", deletion.clone(), baseline.clone())
            .0
            .status
            .code(),
        Some(1)
    );
    fixture.write("topics/health/health.md", health);
    let (output, report) = write_check(&fixture, "validate", deletion, baseline);
    assert!(output.status.success(), "{report}");
}

#[test]
fn write_set_rejects_escapes_symlinks_unknown_fields_and_wrong_baselines() {
    let fixture = Fixture::new("write-set-input");
    for path in [
        "../escape.md",
        "/absolute/file.md",
        "topics/../file.md",
        "topics//file.md",
        ".hidden/file.md",
    ] {
        assert_eq!(
            write_check(
                &fixture,
                "baseline",
                json!([{"path":path,"expect":"present"}]),
                Value::Null
            )
            .0
            .status
            .code(),
            Some(2)
        );
    }
    fs::create_dir_all(fixture.0.join("topics")).unwrap();
    symlink(&fixture.0, fixture.0.join("topics/link")).unwrap();
    assert_eq!(
        write_check(
            &fixture,
            "baseline",
            json!([{"path":"topics/link/file.md","expect":"present"}]),
            Value::Null
        )
        .0
        .status
        .code(),
        Some(2)
    );
    let changes = json!([{"path":"topics/file.md","expect":"present"}]);
    assert_eq!(
        write_check(&fixture, "validate", changes.clone(), json!({}))
            .0
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        write_check(
            &fixture,
            "baseline",
            json!([changes[0], changes[0]]),
            Value::Null
        )
        .0
        .status
        .code(),
        Some(2)
    );
}

#[test]
fn provenance_contract_is_enforced_by_strict_cli_and_legacy_stays_compatible() {
    let fixture = Fixture::new("provenance-cli");
    let note = "topics/dossier/raw/20261112-example/20261112-example.md";
    let good = format!(
        "---\ntype: reference\ncategory: source\nwiki_status: pending\ncssclasses: [vault-navigation, vault-dashboard, vault-event, vault-event-v1-1]\nactivity_contract: provenance-v1\n---\n# Example\n\n> [!navigation|shortcuts]\n> - [[topics/dossier/raw/raw|Back]]\n> - [Finder](file://{}/topics/dossier/raw/20261112-example)\n\n> [!wiki-facts|dossier]\n> - Dossier **[[topics/dossier/dossier|Dossier]]**\n\n```vault-event-detail\n```\n\n> [!event-context] Context\n> Fictional source.\n\n## Activity\n\n> [!activity-day|2026-11-12] 12 Nov 2026\n\n> [!activity-message|human] Alex\n> `00:01 · Alex · Codex-chat`\n> Input.\n\n> [!activity-message|codex] Codex\n> `— · Codex · draft`\n> <!-- viggobsidian:activity:v1 kind=result goal=example -->\n> Resultaat.\n",
        fixture.0.display()
    );
    fixture.write(note, &good);
    let scope = "topics/dossier/raw/20261112-example";
    let output = fixture.strict(scope);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    for bad in [
        good.replace("> `00:01 · Alex · Codex-chat`\n", ""),
        good.replace("00:01 · Alex", "00:01 · Someone"),
        good.replace("kind=result", "kind=process"),
        good.replace("provenance-v1", "provenance-v9"),
    ] {
        fixture.write(note, bad);
        let output = fixture.strict(scope);
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stdout).contains("strict-activity-provenance"));
    }
    let header = good
        .replace(
            "Alex\n> `00:01 · Alex · Codex-chat`",
            "Alex <span class=\"activity-meta\">00:01 · Codex-chat</span>",
        )
        .replace(
            "Codex\n> `— · Codex · draft`",
            "Codex <span class=\"activity-meta\">— · draft</span>",
        );
    fixture.write(note, &header);
    assert!(fixture.strict(scope).status.success());
    fixture.write(note, header.replace("00:01", "24:00"));
    assert!(!fixture.strict(scope).status.success());
    fixture.write(
        note,
        good.replace("activity_contract: provenance-v1\n", "")
            .replace("> `00:01 · Alex · Codex-chat`\n", ""),
    );
    assert!(fixture.strict(scope).status.success());
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(".tmp");
        fs::create_dir_all(&root).unwrap();
        let path = root.join(format!("layout-{name}-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        let fixture = Self(path);
        fixture.write(
            "99-system/contracts/vault-schema.yaml",
            include_bytes!("../fixtures/layout-contract.yaml"),
        );
        fixture
    }

    fn write(&self, path: &str, contents: impl AsRef<[u8]>) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn strict(&self, path: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_vault-layout"))
            .arg("--vault")
            .arg(&self.0)
            .args(["--strict", "--json", path])
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn intake_fixture(root: &str, name: &str) -> (Fixture, String) {
    let fixture = Fixture::new(name);
    let project = format!("{root}/2026-11-example");
    let component = format!("{project}/ex-00-project-wiki");
    fixture.write(
        &format!("{project}/2026-11-example.md"),
        "---\ntype: project\nproject-code: EX\nproject_id: example-project\n---\n",
    );
    fixture.write(
        &format!("{component}/ex-00-project-wiki.md"),
        "---\ncategory: project-component\ncode: EX-00\ncomponent-id: example-component\n---\n",
    );
    fixture.write(
        &format!("{component}/raw/raw.md"),
        "---\ntype: note\ncategory: index\n---\n",
    );
    (fixture, format!("{component}/raw"))
}

fn intake(fixture: &Fixture, raw: &str, input: &Value) -> Output {
    intake_bytes(fixture, raw, &serde_json::to_vec(input).unwrap())
}

fn intake_bytes(fixture: &Fixture, raw: &str, input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
        .arg("--vault")
        .arg(&fixture.0)
        .args(["--new-work-task", raw])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

fn promotion_plan(fixture: &Fixture, input: &Value) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
        .arg("--vault")
        .arg(&fixture.0)
        .arg("--plan-work-task-promotion")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn promotion_plan_is_source_bound_read_only_and_uses_the_existing_component_allocator() {
    let (fixture, raw) = intake_fixture("21-program", "promotion-plan");
    let input = json!({"date":"2026-11-11","slug":"prior","title":"Prior task","request":"Earlier complete request"});
    assert!(intake(&fixture, &raw, &input).status.success());
    let input = json!({"date":"2026-11-11","slug":"source","title":"Source title","request":"Exact fictional source\n![[document.pdf]]"});
    let created = intake(&fixture, &raw, &input);
    assert!(
        created.status.success(),
        "{}",
        String::from_utf8_lossy(&created.stderr)
    );
    let result: Value = serde_json::from_slice(&created.stdout).unwrap();
    let path = result["path"].as_str().unwrap();
    let original = fs::read_to_string(fixture.0.join(path)).unwrap();
    let source = original
        .lines()
        .filter(|l| {
            !l.starts_with("task-code:") && !l.starts_with("task-id:") && !l.starts_with("name:")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .replacen("type: \"task\"", "type: \"note\"", 1);
    fixture.write(path, &source);
    fixture.write(
        &format!("{}/{}/document.pdf", raw, "20261111-source"),
        b"fictional binary\0\xff",
    );
    let request = json!({"schema":"viggobsidian.work-task-promotion.v1","path":path,"source_sha256":format!("{:x}",Sha256::digest(source.as_bytes())),"request_id":"71000000-0000-4000-8000-000000000001"});
    let first = promotion_plan(&fixture, &request);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let plan: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(plan["task-code"], "EX-00-T02");
    assert_eq!(plan["task-id"], "task:71000000-0000-4000-8000-000000000001");
    assert!(plan["name"].as_str().unwrap().ends_with("Source title"));
    assert_eq!(
        plan["members"],
        json!(["20261111-prior", "20261111-source", "raw.md"])
    );
    assert_eq!(fs::read_to_string(fixture.0.join(path)).unwrap(), source);
    assert_eq!(promotion_plan(&fixture, &request).stdout, first.stdout);
    let next = intake(
        &fixture,
        &raw,
        &json!({"date":"2026-11-11","slug":"next","title":"Next task","request":"Another request"}),
    );
    assert!(next.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&next.stdout).unwrap()["task-code"],
        "EX-00-T02"
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&promotion_plan(&fixture, &request).stdout).unwrap()["task-code"],
        "EX-00-T03"
    );
    // Valid task-code metadata with corrupt UTF-8 source bytes must fail closed.
    // Skipping it would allocate the existing T03 a second time.
    let prior_path = format!("{raw}/20261111-prior/20261111-prior.md");
    let prior = fs::read_to_string(fixture.0.join(&prior_path)).unwrap();
    let mut corrupt = prior.replace("EX-00-T01", "EX-00-T03").into_bytes();
    corrupt.push(0xff);
    fixture.write(&prior_path, &corrupt);
    assert!(!promotion_plan(&fixture, &request).status.success());
    let duplicate = intake(
        &fixture,
        &raw,
        &json!({"date":"2026-11-11","slug":"duplicate","title":"Must refuse","request":"No duplicate code"}),
    );
    assert!(!duplicate.status.success());
    assert!(!fixture.0.join(format!("{raw}/20261111-duplicate")).exists());
    assert_eq!(fs::read(fixture.0.join(&prior_path)).unwrap(), corrupt);
    fixture.write(&prior_path, &prior);
    let mut bad = request.clone();
    bad["source_sha256"] = json!("0".repeat(64));
    assert!(!promotion_plan(&fixture, &bad).status.success());
    bad = request.clone();
    bad["path"] = json!("../outside.md");
    assert!(!promotion_plan(&fixture, &bad).status.success());
    bad = request.clone();
    bad["request_id"] = json!("not-a-uuid");
    assert!(!promotion_plan(&fixture, &bad).status.success());
    bad = request.clone();
    bad["free_metadata"] = json!({"task-code":"BYPASS-T01"});
    assert!(!promotion_plan(&fixture, &bad).status.success());
    for binding in [
        "source_identity: imported",
        "work_dossier: owned",
        "generated_by: compiler",
        "readonly: false",
        "wiki_status: processed",
    ] {
        let changed = source.replacen("---\n", &format!("---\n{binding}\n"), 1);
        fixture.write(path, &changed);
        bad = request.clone();
        bad["source_sha256"] = json!(format!("{:x}", Sha256::digest(changed.as_bytes())));
        assert!(
            !promotion_plan(&fixture, &bad).status.success(),
            "{binding}"
        );
    }
    fixture.write(path, &source);
    let other = format!("{raw}/20261111-foreign");
    symlink(
        fixture.0.join(format!("{raw}/20261111-prior")),
        fixture.0.join(other),
    )
    .unwrap();
    assert!(!promotion_plan(&fixture, &request).status.success());
}

#[test]
fn local_intake_is_valid_and_idempotent_in_every_work_program() {
    for root in [
        "20-projects",
        "21-program",
        "22-program",
        "23-program",
        "24-program",
        "25-program",
        "26-program",
    ] {
        let (fixture, raw) = intake_fixture(root, &format!("intake-{root}"));
        let input = json!({"date":"2026-11-11","slug":"source-task","title":"Voorbereiding · Demo","request":"Letterlijke input\n\nTweede alinea.","work_date":"2026-11-12"});
        let result = intake(&fixture, &raw, &input);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let first: Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(first["task-code"], "EX-00-T01");
        assert_eq!(first["reused"], false);
        let created = fs::read_to_string(fixture.0.join(first["path"].as_str().unwrap())).unwrap();
        assert!(created.contains("\n>\n> *— · Human · Work intake*\n"));
        assert!(!created.contains("activity-meta"));
        assert!(created.contains("activity_contract: \"provenance-v1\""));
        assert!(!created.contains("> `— ·"));
        let path = first["path"].as_str().unwrap();
        let content = fs::read(fixture.0.join(path)).unwrap();
        let package = Path::new(path).parent().unwrap().to_str().unwrap();
        let check = fixture.strict(package);
        assert!(
            check.status.success(),
            "{}",
            String::from_utf8_lossy(&check.stdout)
        );
        let retry = intake(&fixture, &raw, &input);
        let again: Value = serde_json::from_slice(&retry.stdout).unwrap();
        assert_eq!(again["task-id"], first["task-id"]);
        assert_eq!(again["reused"], true);
        let mut shortened = input.clone();
        shortened["title"] = json!("Demo");
        assert!(
            !intake(&fixture, &raw, &shortened).status.success(),
            "suffix title must not reuse another title"
        );
        let mut changed = input.clone();
        changed["request"] = json!("Different input");
        assert!(!intake(&fixture, &raw, &changed).status.success());
        assert_eq!(fs::read(fixture.0.join(path)).unwrap(), content);
        let mut next = input;
        next["slug"] = json!("next-task");
        let created: Value = serde_json::from_slice(&intake(&fixture, &raw, &next).stdout).unwrap();
        assert_eq!(created["task-code"], "EX-00-T02");
        assert_ne!(created["task-id"], first["task-id"]);
    }
}

#[test]
fn intake_reuses_exact_source_and_keeps_the_source_time_on_retry() {
    let (fixture, raw) = intake_fixture("21-program", "intake-recorded-provenance");
    for (slug, kind, channel) in [
        ("recorded-action", "launcher-action", "Launcher"),
        ("imported-message", "chat-message", "Chat"),
    ] {
        let mut input = json!({
            "date":"2026-11-12", "slug":slug, "title":"Fictional request", "request":"Original input.",
            "provenance":{
                "kind":kind, "timestamp":"2026-11-11T23:25:31Z", "display_timezone":"Europe/Amsterdam",
                "author":"Fictional Person", "channel":channel
            }
        });
        assert!(!intake(&fixture, &raw, &json!({
            "date":"2026-11-11", "slug":slug, "title":"Fictional request", "request":"Original input.",
            "provenance":input["provenance"],
        })).status.success());
        let first = intake(&fixture, &raw, &input);
        assert!(
            first.status.success(),
            "{}",
            String::from_utf8_lossy(&first.stderr)
        );
        let first: Value = serde_json::from_slice(&first.stdout).unwrap();
        let path = first["path"].as_str().unwrap();
        let note = fixture.0.join(path);
        let content = fs::read_to_string(&note).unwrap();
        assert!(content.contains("> [!activity-day|2026-11-12] 2026-11-12"));
        assert!(content.contains("> [!activity-message|human] Fictional Person\n"));
        assert!(content.ends_with(&format!(">\n> *00:25 · Fictional Person · {channel}*\n")));
        assert!(!content.contains("activity-meta"));
        let check = fixture.strict(Path::new(path).parent().unwrap().to_str().unwrap());
        assert!(
            check.status.success(),
            "{}",
            String::from_utf8_lossy(&check.stdout)
        );

        let retry = intake(&fixture, &raw, &input);
        assert!(retry.status.success());
        let retry: Value = serde_json::from_slice(&retry.stdout).unwrap();
        assert_eq!(retry["reused"], true);
        assert_eq!(retry["task-id"], first["task-id"]);
        input["provenance"]["timestamp"] = json!("2026-11-11T23:25:59Z");
        assert!(!intake(&fixture, &raw, &input).status.success());
        input.as_object_mut().unwrap().remove("provenance");
        assert!(!intake(&fixture, &raw, &input).status.success());
        assert_eq!(fs::read_to_string(note).unwrap(), content);
    }
}

#[test]
fn intake_rejects_ambiguous_source_facts_and_folds_long_literal_requests() {
    let (fixture, raw) = intake_fixture("21-program", "intake-literal-provenance");
    let request = format!(
        "{}\n```\n*00:11 · Fictional Person · Fake channel*\n<!-- viggobsidian:activity:v1 kind=result goal=not-real -->",
        "Unabridged fictional request. ".repeat(80)
    );
    let input = json!({
        "date":"2026-11-12", "slug":"long-action", "title":"Fictional request", "request":request,
        "provenance":{
            "kind":"launcher-action", "timestamp":"2026-11-11T23:25:31Z", "display_timezone":"Europe/Amsterdam",
            "author":"Fictional Person", "channel":"Launcher"
        }
    });
    let duplicate = input.to_string().replace(
        "\"timestamp\":",
        "\"timestamp\":\"2026-11-11T23:45:00Z\",\"timestamp\":",
    );
    let result = intake_bytes(&fixture, &raw, duplicate.as_bytes());
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("duplicate"));
    assert!(!fixture.0.join(&raw).join("20261112-long-action").exists());

    let result = intake(&fixture, &raw, &input);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let receipt: Value = serde_json::from_slice(&result.stdout).unwrap();
    let path = receipt["path"].as_str().unwrap();
    let content = fs::read_to_string(fixture.0.join(path)).unwrap();
    let (_, detail) = content.split_once("> > ````text\n").unwrap();
    let (detail, footer) = detail.split_once("> > ````\n").unwrap();
    let restored = detail
        .lines()
        .map(|line| line.strip_prefix("> > ").unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(restored, request);
    assert_eq!(footer, ">\n> *00:25 · Fictional Person · Launcher*\n");
    let check = fixture.strict(Path::new(path).parent().unwrap().to_str().unwrap());
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stdout)
    );
}

#[test]
fn local_intake_refuses_busy_lock_and_path_escape_without_creating_a_task() {
    let (fixture, raw) = intake_fixture("21-program", "intake-lock");
    let lock = fs::File::create(fixture.0.join(&raw).join(".work-intake.lock")).unwrap();
    lock.try_lock().unwrap();
    let input = json!({"date":"2026-11-11","slug":"blocked","title":"Test","request":"Original"});
    let result = intake(&fixture, &raw, &input);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("intake busy"));
    assert!(!fixture.0.join(&raw).join("20261111-blocked").exists());
    drop(lock);
    assert!(!intake(&fixture, "../outside/raw", &input).status.success());
    assert!(intake(&fixture, &raw, &input).status.success());
}

#[test]
fn full_work_check_reports_later_collections_after_an_earlier_failure() {
    let fixture = Fixture::new("work-full-acceptance");
    // Deliberately invalid collections: the later one must still be visited.
    fixture.write("00-inbox/unexpected.md", "bad capture");
    fixture.write("99-system/vault/test.md", "bad guide");
    fixture.write("90-templates/example.md", "template");
    let result = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
        .arg("--vault")
        .arg(&fixture.0)
        .arg("--check-work")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("Checking 00-inbox"));
    assert!(text.contains("Checking 99-system"));
    assert!(text.contains("unexpected.md"));
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("Work package:")
    );
}

#[test]
fn strict_raw_task_requires_event_note_v1_activity() {
    let fixture = Fixture::new("raw-task-activity");
    let package = "topics/dossier/raw/20261108-example";
    let note = format!("{package}/20261108-example.md");
    let write_task = |metadata, activity| {
        fixture.write(
            &note,
            format!(
                "---\ntype: task\ncategory: project-task\nstatus: 🟢 done\nwiki_status: pending\ncssclasses: [vault-navigation, vault-dashboard, vault-event]\n{metadata}---\n\n# Task\n\n> [!navigation|shortcuts]\n> - [[topics/dossier/raw/raw|Back]]\n> - [Finder](file://{}/{package})\n\n> [!wiki-facts|dossier]\n> - Dossier **[[topics/dossier/dossier|Dossier]]**\n\n```vault-task-detail\n```\n\n## Activity\n\n{activity}\n",
                fixture.0.display()
            ),
        );
    };
    let old = "# 🟢 done · 8 Nov 2026\n`14:47 · Agent · vault-task`\n\nTaak afgerond.";
    for metadata in ["activity_format: chat-v1\n", ""] {
        write_task(metadata, old);
        let result = fixture.strict(package);
        assert_eq!(result.status.code(), Some(1));
        let report: Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(report["findings"].as_array().unwrap().len(), 1);
        assert_eq!(report["findings"][0]["code"], "strict-task-activity");
    }
    write_task(
        "",
        "> [!activity-day|2026-11-08] 8 november 2026\n\n> [!activity-message|codex] Codex\n> Taak afgerond.\n>\n> *14:47 · vault-task*",
    );
    let result = fixture.strict(package);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let untouched = fs::read(fixture.0.join(&note)).unwrap();
    let invalid_mail = format!(
        "{}\n> <!-- viggobsidian:mail-draft:v1 recipients=resolved -->\n> Mailstatus: concept\n> <!-- /viggobsidian:mail-draft -->\n",
        String::from_utf8_lossy(&untouched)
    );
    fixture.write(&note, &invalid_mail);
    let report = fixture.strict(package);
    assert_eq!(report.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&report.stdout).contains("strict-mail-draft"));
    assert_eq!(
        fs::read(fixture.0.join(&note)).unwrap(),
        invalid_mail.as_bytes()
    );
}

#[test]
fn strict_system_check_requires_its_policy() {
    let fixture = Fixture::new("system-policy");
    let output = fixture.strict("99-system");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("system-layout.yaml"));
}

#[test]
fn explicit_writer_history_paths_keep_other_files_and_symlinks_blocked() {
    let fixture = Fixture::new("writer-history-policy");
    let history = "99-system/agents/guide-wijzigingshistorie.md";
    fixture.write(
        "99-system/contracts/system-layout.yaml",
        "collections: [99-system/agents]\nfixed_files: [99-system/agents/guide.md]\ndelegated: {}\n",
    );
    fixture.write(
        "99-system/agents/agents.md",
        format!("---\ntype: collection\ncategory: index\nlayer: navigation\n---\n# Agents\n\n> [!navigation|shortcuts]\n> - [[99-system/99-system|Back]]\n> - [Finder](file://{}/99-system/agents)\n", fixture.0.display()),
    );
    fixture.write("99-system/agents/guide.md", "# Existing owner\n");
    let original = "# History\n\nPreserved previous wording, evidence and owner.\n";
    fixture.write(history, original);
    let findings = || {
        let report: Value =
            serde_json::from_slice(&fixture.strict("99-system/agents").stdout).unwrap();
        report["findings"].as_array().unwrap().clone()
    };
    assert!(
        findings()
            .iter()
            .any(|f| f["code"] == "loose-file" && f["path"] == history)
    );
    let schema_path = fixture.0.join("99-system/contracts/vault-schema.yaml");
    let mut schema: Value =
        serde_saphyr::from_str(&fs::read_to_string(&schema_path).unwrap()).unwrap();
    schema["strict_layout_exceptions"]["writer_files"] =
        json!({history:"Exact local history writer output"});
    fs::write(&schema_path, serde_json::to_vec(&schema).unwrap()).unwrap();
    assert!(findings().is_empty());
    let other = "99-system/agents/other-wijzigingshistorie.md";
    fixture.write(other, "# Unregistered\n");
    assert!(
        findings()
            .iter()
            .any(|f| f["code"] == "loose-file" && f["path"] == other)
    );
    fs::remove_file(fixture.0.join(other)).unwrap();
    assert_eq!(
        fs::read_to_string(fixture.0.join(history)).unwrap(),
        original
    );
    fs::remove_file(fixture.0.join(history)).unwrap();
    symlink("guide.md", fixture.0.join(history)).unwrap();
    assert!(
        findings()
            .iter()
            .any(|f| f["code"] == "symlink" && f["path"] == history)
    );
}

#[test]
fn delegation_preserves_physical_navigation_coverage() {
    let fixture = Fixture::new("delegated-navigation");
    fixture.write(
        "99-system/contracts/system-layout.yaml",
        "collections: [99-system/agents]\nfixed_files: []\ndelegated:\n  99-system/agents/archive: historical archive\n  99-system/agents/__pycache__: generated cache\n",
    );
    fixture.write("99-system/agents/archive/data.json", "{}");
    fixture.write("99-system/agents/__pycache__/cache.pyc", "");
    let write_index = |tile| {
        fixture.write(
            "99-system/agents/agents.md",
            format!(
                "---\ntype: collection\ncategory: index\nlayer: navigation\n---\n# Agents\n\n> [!navigation|shortcuts]\n> - [[99-system/99-system|Back]]\n> - [Finder](file://{}/99-system/agents)\n{tile}",
                fixture.0.display()
            ),
        );
    };
    let codes = || {
        let report: Value =
            serde_json::from_slice(&fixture.strict("99-system/agents").stdout).unwrap();
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["code"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    write_index("");
    assert_eq!(codes(), ["missing-tile", "delegated-folder-note"]);
    write_index("\n> [!navigation]\n> - [[99-system/agents/archive/archive|Archive]]\n");
    assert_eq!(codes(), ["delegated-folder-note"]);
    fixture.write("99-system/agents/archive/archive.md", "# Archive");
    assert!(codes().is_empty());
}

#[test]
fn strict_navigation_checks_destinations_and_accepts_legacy_shortcuts() {
    let fixture = Fixture::new("navigation");
    let folder = "topics/collection";
    fixture.write("topics/topics.md", "# Topics");
    fixture.write("other/topics.md", "# Other topics");
    fixture.write("HOME.md", "# Home");
    let write_note = |back, finder: &Path| {
        fixture.write(
            "topics/collection/collection.md",
            format!(
                "---\ntype: collection\ncategory: index\nlayer: navigation\n---\n# Collection\n\n> [!navigation|back]\n> - [[{back}|Terug]]\n> - [Finder](file://{})\n",
                finder.display()
            ),
        );
    };
    let current = fixture.0.join(folder);
    write_note("topics/topics", &current);
    assert!(fixture.strict(folder).status.success());
    write_note("topics", &current);
    assert_eq!(
        fixture.strict(folder).status.code(),
        Some(1),
        "ambiguous Back"
    );
    fs::remove_file(fixture.0.join("other/topics.md")).unwrap();
    assert!(fixture.strict(folder).status.success(), "unique short Back");
    write_note("HOME", &current);
    assert_eq!(fixture.strict(folder).status.code(), Some(1), "wrong Back");
    write_note("topics/topics", &fixture.0.join("topics"));
    assert_eq!(
        fixture.strict(folder).status.code(),
        Some(1),
        "wrong Finder"
    );
}

#[test]
fn strict_mail_checks_manifest_paths_binding_and_source_bytes() {
    let fixture = Fixture::new("mail-manifest");
    let folder = "00-inbox/20261108-message";
    let note = "20261108-message.md";
    let eml = "20261108-message.eml";
    let attachment = "attachments/file.txt";
    let marker = ".mail-source.json";
    let write = |member, bytes: &[u8]| fixture.write(&format!("{folder}/{member}"), bytes);
    let codes = || {
        let output = fixture.strict(folder);
        assert!(matches!(output.status.code(), Some(0 | 1)));
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| item["code"] == "mail-manifest")
            .map(|item| item["message"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    write(note, b"---\ntype: event\ncategory: email\n---\n# Mail\n");
    write(eml, b"example email");
    write(attachment, b"example attachment");
    let eml_hash = format!("{:x}", Sha256::digest(b"example email"));
    let attachment_hash = format!("{:x}", Sha256::digest(b"example attachment"));
    let valid = json!({
        "schema": "viggo-email-ingestion.package.v1", "eml": eml,
        "capture_foldernote": note, "raw_sha256": eml_hash,
        "files": {(eml): eml_hash, (attachment): attachment_hash},
        "attachments": [{"file": attachment, "sha256": attachment_hash, "bytes": 18}],
    });
    let write_manifest = |value: &Value| write(marker, value.to_string().as_bytes());

    assert_eq!(codes().len(), 1, "missing manifest");
    write_manifest(&valid);
    assert!(codes().is_empty(), "valid original");
    write(
        note,
        b"---\ntype: event\ncategory: email\ndate_basis: ingested\nevent_date: null\ningested_at: 2026-11-08T12:00:00+01:00\n---\n# Mail\n",
    );
    assert!(
        fixture.strict(folder).status.success(),
        "unknown original date"
    );
    write(note, b"---\ntype: event\ncategory: email\ndate_basis: message-date\nevent_date: 2026-11-09\n---\n# Mail\n");
    assert_eq!(
        fixture.strict(folder).status.code(),
        Some(1),
        "mismatched original date"
    );
    write(note, b"---\ntype: event\ncategory: email\ndate_basis: message-date\nevent_date: 2026-11-08\n---\n# Mail\n");

    write(attachment, b"changed attachment");
    assert!(codes()[0].contains("hash mismatch"));
    write(attachment, b"example attachment");
    write(eml, b"changed email");
    assert!(codes()[0].contains("hash mismatch"));
    write(eml, b"example email");

    let mut invalid = valid.clone();
    invalid["attachments"][0]["bytes"] = json!(1);
    write_manifest(&invalid);
    assert!(codes()[0].contains("attachment metadata mismatch"));
    invalid = valid.clone();
    invalid["files"]["../escape"] = json!(eml_hash);
    write_manifest(&invalid);
    assert!(codes()[0].contains("invalid mail manifest member"));
    invalid = valid.clone();
    invalid["capture_foldernote"] = json!("other.md");
    write_manifest(&invalid);
    assert!(codes()[0].contains("matching folder note"));
    invalid = valid.clone();
    invalid["eml"] = json!("other.eml");
    write_manifest(&invalid);
    assert!(codes()[0].contains("same-name EML"));

    write_manifest(&valid);
    let member = fixture.0.join(folder).join(attachment);
    fs::remove_file(&member).unwrap();
    assert_eq!(codes().len(), 1, "missing attachment");
    symlink(fixture.0.join(folder).join(eml), member).unwrap();
    assert!(codes()[0].contains("symbolic mail member"));

    let manifest = fixture.0.join(folder).join(marker);
    fs::remove_file(&manifest).unwrap();
    symlink(fixture.0.join(folder).join(eml), manifest).unwrap();
    assert!(codes()[0].contains("regular file"));
}

#[test]
fn strict_v11_source_rejects_loose_content_after_a_valid_bubble() {
    let fixture = Fixture::new("v11-source-tail");
    let package = "topics/dossier/raw/20261111-source";
    let path = format!("{package}/20261111-source.md");
    let good = format!(
        "---\ntype: reference\ncategory: source\nwiki_status: pending\ncssclasses: [vault-navigation, vault-dashboard, vault-event, vault-event-v1-1]\n---\n\n# Source\n\n> [!navigation|shortcuts]\n> - [[topics/dossier/raw/raw|Back]]\n> - [Finder](file://{}/{package})\n\n> [!wiki-facts|dossier]\n> - Dossier **[[topics/dossier/dossier|Dossier]]**\n\n```vault-event-detail\n```\n\n> [!event-context] Context\n> Historical source.\n\n## Activity\n\n> [!activity-day|2026-11-11] 11 november 2026\n\n> [!activity-message|codex] Codex\n> Source inspected.\n>\n> > [!info]- Meer\n> > Complete provenance.\n>\n> *Codex*\n",
        fixture.0.display()
    );
    fixture.write(&path, &good);
    assert!(fixture.strict(package).status.success());
    for tail in [
        "## Herkomst\n\nLegacy path.",
        "### Verwerking in Wiki",
        "Loose source text.",
        "> [!info]- Archive\n> Legacy content.",
    ] {
        fixture.write(&path, format!("{good}\n{tail}\n"));
        let result = fixture.strict(package);
        assert_eq!(result.status.code(), Some(1));
        let report: Value = serde_json::from_slice(&result.stdout).unwrap();
        assert!(
            report["findings"]
                .as_array()
                .unwrap()
                .iter()
                .any(|finding| finding["code"] == "strict-event-header")
        );
    }
}

#[test]
fn immutable_export_payloads_require_typed_bytes_and_never_exempt_the_event() {
    let fixture = Fixture::new("immutable-export");
    let package = "topics/dossier/raw/20261111-export";
    let note = format!("{package}/20261111-export.md");
    let payload = "---\ntype: migration-collection\nmapping_version: fixture-v1\nsource_set: fictional\n---\n# Export\n\n## sample.1\n\nBron-ID: `sample.1` · accepted ^record-0123456789abcdef\n\n### Geïmporteerde velden\n\n```yaml\nid: sample.1\n```\n";
    let head = format!(
        "---\ntype: reference\ncategory: source\nwiki_status: pending\nexport_artifacts: source-export.json\ncssclasses: [vault-navigation, vault-dashboard, vault-event, vault-event-v1-1]\n---\n\n# Export\n\n> [!navigation|shortcuts]\n> - [[topics/dossier/raw/raw|Back]]\n> - [Finder](file://{}/{package})\n\n> [!wiki-facts|dossier]\n> - Dossier **[[topics/dossier/dossier|Dossier]]**\n\n```vault-event-detail\n```\n\n> [!event-context] Context\n> Immutable historical export; no current factual claim.\n\n## Activity\n\n> [!activity-day|2026-11-11] 11 november 2026\n\n> [!activity-message|codex] Codex\n> Source reviewed.\n>\n> > [!info]- Meer\n> > Consumer data remains unchanged.\n>\n> *Codex*\n",
        fixture.0.display()
    );
    fixture.write(&note, &head);
    fixture.write(&format!("{package}/data.md"), payload);
    fixture.write(&format!("{package}/source-export.json"),json!({"schema":"viggobsidian.source-export.v1","package_note":"20261111-export.md","consumer":"fictional-importer","artifacts":[{"path":"data.md","format":"migration-records-v1","bytes":payload.len(),"sha256":format!("{:x}",Sha256::digest(payload.as_bytes()))}]}).to_string());
    let good = fixture.strict(package);
    assert!(
        good.status.success(),
        "{}",
        String::from_utf8_lossy(&good.stdout)
    );
    fixture.write(&note, head.replace("## Activity", "## Missing activity"));
    let failed: Value = serde_json::from_slice(&fixture.strict(package).stdout).unwrap();
    assert!(
        failed["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["code"] == "strict-event-header")
    );
    fixture.write(&note, &head);
    fixture.write(&format!("{package}/ordinary.md"), "# Ordinary note\n");
    assert_eq!(fixture.strict(package).status.code(), Some(1));
    fs::remove_file(fixture.0.join(format!("{package}/ordinary.md"))).unwrap();
    fixture.write(&format!("{package}/data.md"), format!("{payload}\n"));
    let failed: Value = serde_json::from_slice(&fixture.strict(package).stdout).unwrap();
    assert!(
        failed["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["code"] == "strict-export-artifacts")
    );
    assert!(
        failed["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["code"] == "strict-raw-markdown")
    );
}

#[test]
fn frozen_reading_capture_is_payload_only_after_complete_native_validation() {
    let fixture = Fixture::new("native-source-archive");
    let package = "topics/dossier/raw/20261111-source";
    let archive = format!("{package}/_source-assets/original-package");
    let head = format!(
        "---\ntype: reference\ncategory: source\nwiki_status: pending\nsource_archives: source-archives.json\ncssclasses: [vault-navigation, vault-dashboard, vault-event, vault-event-v1-1]\n---\n\n# Historical source review\n\n> [!navigation|shortcuts]\n> - [[topics/dossier/raw/raw|Back]]\n> - [Finder](file://{}/{package})\n\n> [!wiki-facts|dossier]\n> - Dossier **[[topics/dossier/dossier|Dossier]]**\n\n```vault-event-detail\n```\n\n> [!event-context] Context\n> Frozen source capture; no clinical claim.\n\n## Activity\n\n> [!activity-day|2026-11-11] 11 november 2026\n\n> [!activity-message|codex] Codex\n> Source reviewed.\n>\n> > [!info]- Meer\n> > Original producer bytes remain immutable evidence.\n>\n> *Codex*\n",
        fixture.0.display()
    );
    fixture.write(&format!("{package}/20261111-source.md"), &head);
    let original = "---\ntype: bookmark\ncapture: safari-reading-list\nurl: https://example.invalid/page\ncaptured: 2020-01-01T12:00:00Z\n---\n# Original page capture\n";
    let mut contents = BTreeMap::from([
        ("capture.md", original.to_string()),
        ("entry.plist", "Fictional original plist entry".into()),
        (
            "source.json",
            r#"{"url":"https://example.invalid/page"}"#.into(),
        ),
        (
            "extracted.md",
            "# Original page text\n\nFictional article.\n".into(),
        ),
    ]);
    let pinned = contents
        .iter()
        .filter(|(n, _)| **n != "capture.md")
        .map(|(n, s)| (*n, format!("{:x}", Sha256::digest(s.as_bytes()))))
        .collect::<BTreeMap<_, _>>();
    contents.insert(".reading-list-source.json",json!({"schema":1,"identity":"a".repeat(64),"url":"https://example.invalid/page","files":pinned}).to_string());
    let mut files = BTreeMap::new();
    for (n, s) in &contents {
        fixture.write(&format!("{archive}/{n}"), s);
        files.insert(
            *n,
            json!({"sha256":format!("{:x}",Sha256::digest(s.as_bytes())),"bytes":s.len()}),
        );
    }
    fixture.write(&format!("{package}/source-archives.json"),json!({"schema":"viggobsidian.native-source-archives.v1","package_note":"20261111-source.md","archives":[{"directory":"_source-assets/original-package","producer":"safari-reading-list","capture_note":"capture.md","files":files,"directories":[]}]}).to_string());
    let good = fixture.strict(package);
    assert!(
        good.status.success(),
        "{}",
        String::from_utf8_lossy(&good.stdout)
    );
    fixture.write(
        &format!("{package}/ordinary.md"),
        "# A separate ordinary note\n",
    );
    assert_eq!(fixture.strict(package).status.code(), Some(1));
    fs::remove_file(fixture.0.join(format!("{package}/ordinary.md"))).unwrap();
    fixture.write(
        &format!("{package}/20261111-source.md"),
        head.replace("## Activity", "## Missing activity"),
    );
    let bad: Value = serde_json::from_slice(&fixture.strict(package).stdout).unwrap();
    assert!(
        bad["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["code"] == "strict-event-header")
    );
    fixture.write(&format!("{package}/20261111-source.md"), &head);
    fixture.write(&format!("{archive}/extracted.md"), "Changed bytes");
    let bad: Value = serde_json::from_slice(&fixture.strict(package).stdout).unwrap();
    assert!(
        bad["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["code"] == "strict-native-source-archives")
    );
    assert!(
        bad["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["code"] == "strict-raw-markdown")
    );
}

#[test]
fn deleted_skill_routes_are_unknown_and_effects_require_explicit_selection() {
    let fixture = procedure_fixture("retired-routes");
    for name in [
        "vault-read",
        "vault-log",
        "vault-process",
        "vault-task",
        "inbox",
        "obsidian",
        "contacts",
    ] {
        let output = skill_check(&fixture, &["--skill-route", name]).0;
        assert_eq!(output.status.code(), Some(2), "{name}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("unknown skill route"));
    }
    assert_eq!(
        skill_check(&fixture, &["--skill-capability", "capture"])
            .0
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        skill_check(
            &fixture,
            &["--skill-route", "vault", "--skill-capability", "unknown"]
        )
        .0
        .status
        .code(),
        Some(2)
    );
    assert_eq!(
        skill_check(
            &fixture,
            &[
                "--skill-route",
                "vault",
                "--skill-capability",
                "capture",
                "--skill-source",
                "imessage"
            ]
        )
        .0
        .status
        .code(),
        Some(1)
    );
}
