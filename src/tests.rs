use super::*;

#[test]
fn installed_activity_tests_bind_a_distributed_crate_local_fixture() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = ".agents/tools/vault-layout/fixtures/activity-deliveries.md";
    assert!(allowed(target));
    let distribution = read_json(&root.join("distribution.json")).unwrap();
    assert!(
        rows(&distribution, "files")
            .unwrap()
            .iter()
            .any(|r| r["source"] == "fixtures/activity-deliveries.md" && r["target"] == target)
    );
    assert_eq!(
        fs::read(root.join("crates/vault-layout/fixtures/activity-deliveries.md")).unwrap(),
        fs::read(root.join("fixtures/activity-deliveries.md")).unwrap()
    );
    assert!(
        fs::read_to_string(root.join("crates/vault-layout/src/activity_delivery.rs"))
            .unwrap()
            .contains("include_str!(\"../fixtures/activity-deliveries.md\")")
    );
}

#[test]
fn reviewed_skill_library_has_an_exact_surface_and_no_local_adapters() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let spec = read_json(&root.join("schemas/skill-library.json")).unwrap();
    let distribution = read_json(&root.join("distribution.json")).unwrap();
    let mut seen = BTreeSet::new();
    for row in rows(&spec, "files").unwrap() {
        let target = field(row, "target").unwrap();
        let source = field(row, "source").unwrap();
        assert!(seen.insert(target));
        assert!(allowed(target));
        assert!(root.join(relative(source).unwrap()).is_file());
        assert!(
            rows(&distribution, "files")
                .unwrap()
                .iter()
                .any(|r| r["target"] == target && r["source"] == source)
        );
        assert!(!target.starts_with(".agents/skill-adapters/"));
        assert!(!target.contains("/scripts/"));
    }
    assert!(!allowed(".agents/skill-adapters/draft.md"));
    assert!(!allowed(".agents/skills/work-client/SKILL.md"));
    assert!(!allowed(".viggobsidian/skill-adapters.json"));
}

#[test]
fn public_skill_sources_do_not_embed_host_home_paths() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let spec = read_json(&root.join("schemas/skill-library.json")).unwrap();
    for row in rows(&spec, "files").unwrap() {
        let source = field(row, "source").unwrap();
        let body = fs::read_to_string(root.join(source)).unwrap();
        for forbidden in ["/Users/", "/home/"] {
            assert!(
                !body.contains(forbidden),
                "host/private source in {source}: {forbidden}"
            );
        }
    }
}

#[test]
fn library_markdown_links_resolve_inside_the_distribution() {
    let mut missing = Vec::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let distribution = read_json(&root.join("distribution.json")).unwrap();
    let targets: BTreeSet<_> = rows(&distribution, "files")
        .unwrap()
        .iter()
        .map(|r| r["target"].as_str().unwrap().to_owned())
        .collect();
    let spec = read_json(&root.join("schemas/skill-library.json")).unwrap();
    for row in rows(&spec, "files").unwrap() {
        let source = field(row, "source").unwrap();
        if !source.ends_with(".md") {
            continue;
        }
        let body = fs::read_to_string(root.join(source)).unwrap();
        let target = field(row, "target").unwrap();
        for part in body.split("](").skip(1) {
            let Some(link) = part.split(')').next() else {
                continue;
            };
            let link = link.split('#').next().unwrap().trim_matches(['<', '>']);
            if link.is_empty()
                || link.contains("://")
                || link.starts_with("mailto:")
                || link.starts_with("tel:")
            {
                continue;
            }
            let mut normalized: Vec<_> = target.split('/').collect();
            normalized.pop();
            for component in Path::new(link).components() {
                match component {
                    Component::Normal(s) => normalized.push(s.to_str().unwrap()),
                    Component::ParentDir => {
                        normalized.pop().expect("link remains within install root");
                    }
                    Component::CurDir => {}
                    _ => panic!("absolute skill link {source}: {link}"),
                }
            }
            let resolved = normalized.join("/");
            if !targets.contains(&resolved) {
                missing.push(format!("{source}: {link} -> {resolved}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "missing shared targets:\n{}",
        missing.join("\n")
    );
}

#[test]
fn all_declared_procedure_assets_are_distributed_with_a_bounded_allowlist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let distribution = read_json(&root.join("distribution.json")).unwrap();
    let targets: BTreeSet<_> = rows(&distribution, "files")
        .unwrap()
        .iter()
        .map(|r| r["target"].as_str().unwrap().to_owned())
        .collect();
    for target in procedure_surface::targets() {
        assert!(
            targets.contains(target),
            "missing distribution for {target}"
        );
        assert!(allowed(target));
    }
    assert!(!allowed(".agents/skills/vault-task/scripts/create_note.py"));
    assert!(!allowed(".viggobsidian/providers.json"));
    assert!(!allowed(".agents/skills/unreviewed/PROCEDURE.md"));
}

#[test]
fn procedure_adoption_preserves_the_existing_provider_and_restores_original_text() {
    let f = Fixture::new();
    let target = ".agents/skills/vault-read/PROCEDURE.md";
    atomic(&f.vault, target, b"Original local procedure").unwrap();
    atomic(
        &f.vault,
        ".agents/skills/vault-read/scripts/provider.py",
        b"Original provider bytes",
    )
    .unwrap();
    fs::write(f.package.join("read.md"), b"Reviewed shared procedure").unwrap();
    let mut manifest = read_json(&f.package.join("package.json")).unwrap();
    manifest["files"].as_array_mut().unwrap().push(
        json!({"source":"read.md","target":target,"sha256":hash(b"Reviewed shared procedure")}),
    );
    fs::write(
        f.package.join("package.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    assert!(plan(&f.vault, &f.package, &json!({})).is_err());
    let proposal = plan(
        &f.vault,
        &f.package,
        &json!({target:hash(b"Original local procedure")}),
    )
    .unwrap();
    let receipt = apply(&f.vault, &proposal).unwrap();
    assert_eq!(
        fs::read(
            f.vault
                .join(".agents/skills/vault-read/scripts/provider.py")
        )
        .unwrap(),
        b"Original provider bytes"
    );
    let receipt = read_json(&f.vault.join(receipt["receipt"].as_str().unwrap())).unwrap();
    rollback(&f.vault, &receipt).unwrap();
    assert_eq!(
        fs::read(f.vault.join(target)).unwrap(),
        b"Original local procedure"
    );
    assert_eq!(
        fs::read(
            f.vault
                .join(".agents/skills/vault-read/scripts/provider.py")
        )
        .unwrap(),
        b"Original provider bytes"
    );
}

#[test]
fn derived_css_binds_local_configuration_rejects_stale_plans_and_rolls_back() {
    let f = Fixture::new();
    let shared = ".vault-dashboard { display: grid; }";
    fs::write(f.package.join("layout.css"), shared).unwrap();
    let mut manifest = read_json(&f.package.join("package.json")).unwrap();
    manifest["files"].as_array_mut().unwrap().extend([
        json!({"source":"layout.css","target":".viggobsidian/core/folder-note.css","sha256":hash(shared.as_bytes())}),
        json!({"source":"layout.css","target":".obsidian/snippets/folder-note.css","sha256":hash(shared.as_bytes()),"transform":"css-v1"})]);
    fs::write(
        f.package.join("package.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let config = json!({"schema":"viggobsidian.appearance.v1","routes":[{"selectors":".vault-dashboard a[data-href=\"fictional\"]","declarations":"--dashboard-accent: red;"}]});
    atomic(
        &f.vault,
        ".viggobsidian/appearance.json",
        &serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    let old_plan = f.plan();
    let changed = config.to_string().replace("red", "blue");
    atomic(
        &f.vault,
        ".viggobsidian/appearance.json",
        changed.as_bytes(),
    )
    .unwrap();
    assert!(apply(&f.vault, &old_plan).is_err());
    let result = apply(&f.vault, &f.plan()).unwrap();
    let css = fs::read_to_string(f.vault.join(".obsidian/snippets/folder-note.css")).unwrap();
    assert!(css.contains("blue"));
    assert!(!css.contains("red;"));
    assert!(state(&f.vault).is_ok());
    assert_eq!(apply(&f.vault, &f.plan()).unwrap()["status"], "unchanged");
    let receipt = read_json(&f.vault.join(result["receipt"].as_str().unwrap())).unwrap();
    rollback(&f.vault, &receipt).unwrap();
    assert!(!f.vault.join(".obsidian/snippets/folder-note.css").exists());
    assert_eq!(
        fs::read_to_string(f.vault.join(".viggobsidian/appearance.json")).unwrap(),
        changed
    );
}

#[test]
fn doctor_reports_integrity_separately_from_strict_adoption() {
    let f = Fixture::new();
    apply(&f.vault, &f.plan()).unwrap();
    atomic(
        &f.vault,
        ".viggobsidian/local-adapter.md",
        b"# Explicit test providers",
    )
    .unwrap();
    atomic(&f.vault,".viggobsidian/integration.json",&serde_json::to_vec(&json!({
        "schema":"viggobsidian.integration.v1","profile":"numbered-v1",
        "adoption":{"legacy_project_roots":["15-projects"]},"local_adapter":".viggobsidian/local-adapter.md"
    })).unwrap()).unwrap();
    fs::create_dir_all(f.vault.join("15-projects/2026-11-test/00-project-wiki")).unwrap();
    assert!(
        rows(&check(&f.vault, false).unwrap(), "errors")
            .unwrap()
            .is_empty()
    );
    assert!(
        !rows(&check(&f.vault, true).unwrap(), "errors")
            .unwrap()
            .is_empty()
    );
    atomic(&f.vault, ".viggobsidian/integration.json", b"{}").unwrap();
    assert!(
        !rows(&check(&f.vault, false).unwrap(), "errors")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn rollback_resumes_after_first_of_multiple_assets() {
    let f = Fixture::new();
    let mut manifest = read_json(&f.package.join("package.json")).unwrap();
    manifest["files"].as_array_mut().unwrap().push(
        json!({"source":"policy.md","target":".viggobsidian/core/skills.md","sha256":hash(b"one")}),
    );
    fs::write(
        f.package.join("package.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let result = apply(&f.vault, &f.plan()).unwrap();
    let receipt = read_json(&f.vault.join(result["receipt"].as_str().unwrap())).unwrap();
    atomic(
        &f.vault,
        ".viggobsidian/rollback.json",
        &serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    fs::remove_file(f.vault.join(".viggobsidian/core/policy.md")).unwrap();
    assert!(plan(&f.vault, &f.package, &json!({})).is_err());
    rollback(&f.vault, &receipt).unwrap();
    assert!(!f.vault.join(".viggobsidian/core/skills.md").exists());
    assert!(!f.vault.join(".viggobsidian/rollback.json").exists());
    assert!(!f.vault.join(STATE).exists());
}

struct Fixture {
    home: PathBuf,
    vault: PathBuf,
    package: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let home = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(stamp().unwrap());
        let vault = home.join("vault");
        let package = home.join("package");
        fs::create_dir_all(&vault).unwrap();
        fs::create_dir_all(&package).unwrap();
        fs::write(vault.join("AGENTS.md"), "# Test vault\n").unwrap();
        let f = Self {
            home,
            vault,
            package,
        };
        f.asset("one");
        f
    }
    fn asset(&self, content: &str) {
        fs::write(self.package.join("policy.md"), content).unwrap();
        fs::write(self.package.join("package.json"),serde_json::to_vec(&json!({
            "schema":"viggobsidian.package.v1","version":"0.1.0","revision":"a".repeat(40),
            "files":[{"source":"policy.md","target":".viggobsidian/core/policy.md","sha256":hash(content.as_bytes())}]
        })).unwrap()).unwrap();
    }
    fn plan(&self) -> Value {
        plan(&self.vault, &self.package, &json!({})).unwrap()
    }
    fn read(&self) -> String {
        fs::read_to_string(self.vault.join(".viggobsidian/core/policy.md")).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.home);
    }
}

#[test]
fn install_idempotence_update_and_rollback() {
    let f = Fixture::new();
    let p = f.plan();
    assert!(
        !f.vault.join(".viggobsidian").exists(),
        "preview must not write"
    );
    apply(&f.vault, &p).unwrap();
    assert_eq!(f.read(), "one");
    assert_eq!(apply(&f.vault, &f.plan()).unwrap()["status"], "unchanged");
    f.asset("two");
    let result = apply(&f.vault, &f.plan()).unwrap();
    assert_eq!(f.read(), "two");
    let receipt = read_json(&f.vault.join(result["receipt"].as_str().unwrap())).unwrap();
    rollback(&f.vault, &receipt).unwrap();
    assert_eq!(f.read(), "one");
    assert!(
        rollback(&f.vault, &receipt).is_err(),
        "old receipt is not repeatable"
    );
}
#[test]
fn initial_rollback_removes_new_assets() {
    let f = Fixture::new();
    let r = apply(&f.vault, &f.plan()).unwrap();
    rollback(
        &f.vault,
        &read_json(&f.vault.join(r["receipt"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    assert!(!f.vault.join(STATE).exists());
    assert!(!f.vault.join(".viggobsidian/core/policy.md").exists());
}
#[test]
fn unmanaged_adoption_requires_exact_preimage_and_restores_it() {
    let f = Fixture::new();
    atomic(&f.vault, ".viggobsidian/core/policy.md", b"local").unwrap();
    assert!(plan(&f.vault, &f.package, &json!({})).is_err());
    assert!(
        plan(
            &f.vault,
            &f.package,
            &json!({".viggobsidian/core/policy.md":"wrong"})
        )
        .is_err()
    );
    let p = plan(
        &f.vault,
        &f.package,
        &json!({".viggobsidian/core/policy.md":hash(b"local")}),
    )
    .unwrap();
    let r = apply(&f.vault, &p).unwrap();
    rollback(
        &f.vault,
        &read_json(&f.vault.join(r["receipt"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    assert_eq!(f.read(), "local");
}
#[test]
fn edited_managed_file_is_a_conflict_even_when_release_unchanged() {
    let f = Fixture::new();
    apply(&f.vault, &f.plan()).unwrap();
    atomic(&f.vault, ".viggobsidian/core/policy.md", b"edited").unwrap();
    assert!(plan(&f.vault, &f.package, &json!({})).is_err());
    assert_eq!(f.read(), "edited");
}
#[test]
fn stale_plan_and_changed_package_are_refused() {
    let f = Fixture::new();
    let p = f.plan();
    f.asset("changed");
    assert!(apply(&f.vault, &p).is_err());
    assert!(!f.vault.join(STATE).exists());
    fs::write(f.package.join("policy.md"), "unmanifested").unwrap();
    assert!(plan(&f.vault, &f.package, &json!({})).is_err());
}
#[test]
fn tampered_plan_and_wrong_target_are_refused() {
    let f = Fixture::new();
    let mut p = f.plan();
    p["changes"][0]["target"] = json!("AGENTS.md");
    assert!(apply(&f.vault, &p).is_err());
    let mut p = f.plan();
    p["vault"] = json!("/wrong");
    assert!(apply(&f.vault, &p).is_err());
    assert_eq!(
        fs::read_to_string(f.vault.join("AGENTS.md")).unwrap(),
        "# Test vault\n"
    );
}
#[test]
fn rollback_rejects_edited_postimage() {
    let f = Fixture::new();
    let r = apply(&f.vault, &f.plan()).unwrap();
    atomic(&f.vault, ".viggobsidian/core/policy.md", b"human").unwrap();
    assert!(
        rollback(
            &f.vault,
            &read_json(&f.vault.join(r["receipt"].as_str().unwrap())).unwrap()
        )
        .is_err()
    );
    assert_eq!(f.read(), "human");
}
#[test]
fn path_traversal_and_content_targets_are_forbidden() {
    for path in [
        "../x",
        "/absolute",
        "a/../b",
        "a\\b",
        ".git/config",
        "AGENTS.md",
        "20-projects/task.md",
        ".agents/bin/vs",
    ] {
        assert!(!allowed(path), "{path}");
    }
    let f = Fixture::new();
    let mut manifest = read_json(&f.package.join("package.json")).unwrap();
    manifest["files"][0]["target"] = json!("20-projects/task.md");
    fs::write(
        f.package.join("package.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    assert!(package(&f.package).is_err());
}
#[cfg(unix)]
#[test]
fn symlink_root_source_and_destination_are_refused() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    symlink(&f.vault, f.home.join("alias")).unwrap();
    assert!(root(&f.home.join("alias")).is_err());
    symlink(&f.package, f.vault.join(".viggobsidian")).unwrap();
    assert!(plan(&f.vault, &f.package, &json!({})).is_err());
    fs::remove_file(f.vault.join(".viggobsidian")).unwrap();
    fs::remove_file(f.package.join("policy.md")).unwrap();
    symlink(f.vault.join("AGENTS.md"), f.package.join("policy.md")).unwrap();
    assert!(package(&f.package).is_err());
}
#[test]
fn lock_and_unfinished_transaction_block_apply() {
    let f = Fixture::new();
    let p = f.plan();
    let lock = Lock::acquire(&f.vault).unwrap();
    assert!(apply(&f.vault, &p).is_err());
    drop(lock);
    atomic(&f.vault, ".viggobsidian/pending.json", b"{}").unwrap();
    assert!(apply(&f.vault, &p).is_err());
}
#[test]
fn partial_apply_can_recover_original_bytes() {
    let f = Fixture::new();
    let p = f.plan();
    let receipt = json!({"schema":"viggobsidian.receipt.v1","vault":f.vault,"receipt":"test",
        "entries":[{"target":".viggobsidian/core/policy.md","before":null,"after":hash(b"one")}],
        "previous_state":null,"next_state":p["next_state"]});
    atomic(
        &f.vault,
        ".viggobsidian/pending.json",
        &serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    atomic(&f.vault, ".viggobsidian/core/policy.md", b"one").unwrap();
    rollback(&f.vault, &receipt).unwrap();
    assert!(!f.vault.join(".viggobsidian/pending.json").exists());
    assert!(!f.vault.join(".viggobsidian/core/policy.md").exists());
}
#[test]
fn changed_unmanaged_content_and_ids_are_never_touched() {
    let f = Fixture::new();
    fs::create_dir_all(f.vault.join("20-projects")).unwrap();
    let task = f.vault.join("20-projects/task.md");
    fs::write(
        &task,
        "task_id: TEST-042\nsource_id: source-7\nregistration: kept\n",
    )
    .unwrap();
    let before = fs::read(&task).unwrap();
    apply(&f.vault, &f.plan()).unwrap();
    assert_eq!(fs::read(task).unwrap(), before);
}
#[test]
fn profile_project_and_component_names() {
    assert!(!project_name("2026-aé-project"));
    assert!(program("21-programma"));
    assert!(program("29-programma"));
    assert!(!program("20-projects"));
    assert!(!program("30-programma"));
    assert!(project_name("2026-11-project"));
    assert!(!project_name("2026-13-project"));
    assert!(!project_name("project"));
    let profile: Value =
        serde_saphyr::from_str(include_str!("../profiles/numbered-v1.yaml")).unwrap();
    assert_eq!(profile["id"], "numbered-v1");
    assert_eq!(profile["roots"]["projects"], "20-projects");
    let policy: Value =
        serde_saphyr::from_str(include_str!("../schemas/vault-schema.yaml")).unwrap();
    assert_eq!(policy["profile"], "numbered-v1");
}
#[test]
fn duplicate_package_targets_and_retirement_are_refused() {
    let f = Fixture::new();
    apply(&f.vault, &f.plan()).unwrap();
    let mut manifest = read_json(&f.package.join("package.json")).unwrap();
    let duplicate = manifest["files"][0].clone();
    manifest["files"].as_array_mut().unwrap().push(duplicate);
    fs::write(
        f.package.join("package.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    assert!(package(&f.package).is_err());
    f.asset("one");
    let mut manifest = read_json(&f.package.join("package.json")).unwrap();
    manifest["files"][0]["target"] = json!(".viggobsidian/core/skills.md");
    fs::write(
        f.package.join("package.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    assert!(plan(&f.vault, &f.package, &json!({})).is_err());
}
