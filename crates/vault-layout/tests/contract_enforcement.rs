use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

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
    let mut child = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
        .arg("--vault")
        .arg(&fixture.0)
        .args(["--new-work-task", raw])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(input).unwrap())
        .unwrap();
    child.wait_with_output().unwrap()
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
