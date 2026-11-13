use super::*;

#[test]
fn installed_activity_core_has_all_exact_path_dependency_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let distribution = read_json(&root.join("distribution.json")).unwrap();
    for name in [
        "Cargo.toml",
        "src/lib.rs",
        "src/snapshot.rs",
        "src/wasm.rs",
        "src/activity_layout.rs",
        "src/activity_delivery.rs",
        "src/mail_draft.rs",
        "fixtures/activity-legacy.json",
        "fixtures/activity-deliveries.md",
    ] {
        let source = format!("crates/activity-core/{name}");
        let target = format!(".agents/tools/activity-core/{name}");
        assert!(allowed(&target));
        assert!(root.join(&source).is_file());
        assert_eq!(
            rows(&distribution, "files")
                .unwrap()
                .iter()
                .filter(|r| r["source"] == source && r["target"] == target)
                .count(),
            1
        );
    }
    assert!(!allowed(".agents/tools/activity-core/src/unreviewed.rs"));
    assert!(!allowed(
        ".agents/tools/activity-core/../../records/private.jsonl"
    ));
}

#[test]
fn installed_wiki_core_has_every_path_dependency_source() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let distribution = read_json(&root.join("distribution.json")).unwrap();
    for name in [
        "Cargo.toml",
        "src/lib.rs",
        "src/headings.rs",
        "src/snapshot.rs",
        "src/wasm.rs",
        "fixtures/wiki-layout-legacy.json",
    ] {
        let source = format!("crates/wiki-layout-core/{name}");
        let target = format!(".agents/tools/wiki-layout-core/{name}");
        assert!(allowed(&target));
        assert!(root.join(&source).is_file());
        assert_eq!(
            rows(&distribution, "files")
                .unwrap()
                .iter()
                .filter(|row| row["source"] == source && row["target"] == target)
                .count(),
            1
        );
    }
    assert!(!allowed(".agents/tools/wiki-layout-core/src/unreviewed.rs"));
    assert!(!allowed(
        ".agents/tools/wiki-layout-core/../../records/private.jsonl"
    ));
}

#[test]
fn installed_link_style_core_has_every_path_dependency_source() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let distribution = read_json(&root.join("distribution.json")).unwrap();
    for name in ["Cargo.toml", "src/lib.rs", "src/snapshot.rs", "src/wasm.rs"] {
        let source = format!("crates/link-style-core/{name}");
        let target = format!(".agents/tools/link-style-core/{name}");
        assert!(allowed(&target));
        assert!(root.join(&source).is_file());
        assert_eq!(
            rows(&distribution, "files")
                .unwrap()
                .iter()
                .filter(|row| row["source"] == source && row["target"] == target)
                .count(),
            1
        );
    }
    assert!(
        fs::read_to_string(root.join("crates/vault-layout/Cargo.toml"))
            .unwrap()
            .contains("link-style-core = { path = \"../link-style-core\" }")
    );
    assert!(!allowed(".agents/tools/link-style-core/src/unreviewed.rs"));
    assert!(!allowed(
        ".agents/tools/link-style-core/../../records/private.jsonl"
    ));
}

#[test]
fn bundled_procedure_assets_match_the_complete_current_public_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = "crates/vault-layout/fixtures/procedure-assets.json";
    let target = ".agents/tools/vault-layout/fixtures/procedure-assets.json";
    let options = serde_saphyr::options! {duplicate_keys:serde_saphyr::DuplicateKeyPolicy::Error};
    let bundle: Value = serde_saphyr::from_str_with_options(
        &fs::read_to_string(root.join(source)).unwrap(),
        options,
    )
    .unwrap();
    assert_eq!(bundle["schema"], "viggobsidian.procedure-test-fixture.v1");
    let files = bundle["files"].as_object().unwrap();
    assert_eq!(
        files.keys().cloned().collect::<BTreeSet<_>>(),
        *procedure_surface::targets()
    );
    let distribution = read_json(&root.join("distribution.json")).unwrap();
    assert!(allowed(target));
    assert_eq!(
        rows(&distribution, "files")
            .unwrap()
            .iter()
            .filter(|r| r["target"] == target && r["source"] == source)
            .count(),
        1
    );
    for (target, file) in files {
        let entries: Vec<_> = rows(&distribution, "files")
            .unwrap()
            .iter()
            .filter(|r| r["target"] == *target)
            .collect();
        assert_eq!(entries.len(), 1, "{target}");
        assert_eq!(file["source"], entries[0]["source"], "{target}");
        let actual = fs::read_to_string(root.join(file["source"].as_str().unwrap())).unwrap();
        assert_eq!(
            file["content"], actual,
            "stale public procedure fixture: {target}"
        );
    }
}

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
        fs::read_to_string(root.join("crates/activity-core/src/activity_delivery.rs"))
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
    let target = ".agents/skills/vault/SKILL.md";
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
#[test]
fn pack_refuses_source_version_and_bad_revision_without_mutating_manifest() {
    let f = Fixture::new();
    fs::create_dir_all(f.package.join("src")).unwrap();
    fs::write(f.package.join("src/lib.rs"), "").unwrap();
    let original = fs::read(f.package.join("package.json")).unwrap();
    fs::write(
        f.package.join("distribution.json"),
        serde_json::to_vec(
            &json!({"files":[{"source":"policy.md","target":".viggobsidian/core/policy.md"}]}),
        )
        .unwrap(),
    )
    .unwrap();
    for declaration in [
        "version = \"0.0.0\"".to_owned(),
        "version.workspace = true".into(),
        format!(
            "\"version\" = \"0.0.0\"\ndescription = \"\"\"\n[package]\nversion = \"{}\"\n\"\"\"",
            env!("CARGO_PKG_VERSION")
        ),
        format!(
            "version = \"{}\"\nversion = \"{}\"",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_VERSION")
        ),
    ] {
        fs::write(
            f.package.join("Cargo.toml"),
            format!("[package]\nname = \"fixture\"\n{declaration}\n[workspace]\n"),
        )
        .unwrap();
        assert!(pack(&f.package, &"a".repeat(40)).is_err());
        assert_eq!(fs::read(f.package.join("package.json")).unwrap(), original);
    }
    fs::write(
        f.package.join("Cargo.toml"),
        format!(
            "[package]\nname = \"fixture\"\nversion = \"{}\"\n[workspace]\n",
            env!("CARGO_PKG_VERSION")
        ),
    )
    .unwrap();
    assert!(pack(&f.package, "abcdef").is_err());
    assert_eq!(fs::read(f.package.join("package.json")).unwrap(), original);
    let manifest = pack(&f.package, &"a".repeat(40)).unwrap();
    assert_eq!(manifest["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(manifest["revision"], "a".repeat(40));
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
fn procedure_fixture_update_rejects_drift_and_rolls_back_exact_bytes() {
    let f = Fixture::new();
    let target = ".agents/tools/vault-layout/fixtures/procedure-assets.json";
    let original = include_bytes!("../crates/vault-layout/fixtures/procedure-assets.json");
    fs::write(f.vault.join("provider.txt"), "Unmanaged provider").unwrap();
    let publish = |content: &[u8]| {
        fs::write(f.package.join("fixture.json"), content).unwrap();
        let manifest = json!({"schema":"viggobsidian.package.v1","version":"0.14.2","revision":"a".repeat(40),
            "files":[{"source":"fixture.json","target":target,"sha256":hash(content)}]});
        fs::write(f.package.join("package.json"), manifest.to_string()).unwrap();
    };
    publish(original);
    let preview = f.plan();
    assert!(!f.vault.join(target).exists());
    let initial = apply(&f.vault, &preview).unwrap();
    assert_eq!(fs::read(f.vault.join(target)).unwrap(), original);
    publish(b"{\"fictional\":\"updated fixture\"}");
    let update = f.plan();
    fs::write(f.vault.join(target), "Local drift").unwrap();
    assert!(apply(&f.vault, &update).is_err());
    assert_eq!(
        fs::read_to_string(f.vault.join(target)).unwrap(),
        "Local drift"
    );
    fs::write(f.vault.join(target), original).unwrap();
    let updated = apply(&f.vault, &f.plan()).unwrap();
    rollback(
        &f.vault,
        &read_json(&f.vault.join(updated["receipt"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    assert_eq!(fs::read(f.vault.join(target)).unwrap(), original);
    rollback(
        &f.vault,
        &read_json(&f.vault.join(initial["receipt"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    assert!(!f.vault.join(target).exists());
    assert_eq!(
        fs::read_to_string(f.vault.join("provider.txt")).unwrap(),
        "Unmanaged provider"
    );
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

fn github_rename_fixture() -> Fixture {
    let f = Fixture::new();
    // Model a genuine old installation; never bypass the new package gate in production.
    let mut old = read_json(&f.package.join("package.json")).unwrap();
    let mut files = serde_json::Map::new();
    atomic(&f.vault, ".viggobsidian/core/policy.md", b"one").unwrap();
    files.insert(".viggobsidian/core/policy.md".into(), json!(hash(b"one")));
    for target in RETIRED_GO {
        atomic(&f.vault, target, b"old owned instruction").unwrap();
        old["files"].as_array_mut().unwrap().push(
            json!({"source":"old.md","target":target,"sha256":hash(b"old owned instruction")}),
        );
        files.insert((*target).into(), json!(hash(b"old owned instruction")));
    }
    let bytes = serde_json::to_string(&old).unwrap();
    atomic(&f.vault, STATE, &serde_json::to_vec(&json!({"schema":"viggobsidian.installed.v1","version":old["version"],"revision":old["revision"],"manifest_sha256":hash(bytes.as_bytes()),"manifest_bytes":bytes,"files":files,"derived":{}})).unwrap()).unwrap();
    let mut next = read_json(&f.package.join("package.json")).unwrap();
    for target in SKILLS
        .iter()
        .map(|name| format!(".agents/skills/{name}/SKILL.md"))
    {
        next["files"]
            .as_array_mut()
            .unwrap()
            .push(json!({"source":"policy.md","target":target,"sha256":hash(b"one")}));
    }
    fs::write(
        f.package.join("package.json"),
        serde_json::to_vec(&next).unwrap(),
    )
    .unwrap();
    f
}
#[test]
fn finite_github_rename_restores_all_nine_old_assets_and_state() {
    let f = github_rename_fixture();
    let before = fs::read(f.vault.join(STATE)).unwrap();
    let p = f.plan();
    assert_eq!(
        rows(&p, "changes")
            .unwrap()
            .iter()
            .filter(|c| c["after"].is_null())
            .count(),
        9
    );
    let result = apply(&f.vault, &p).unwrap();
    assert!(RETIRED_GO.iter().all(|p| !f.vault.join(p).exists()));
    assert!(state(&f.vault).unwrap()["files"][RETIRED_GO[0]].is_null());
    assert!(rows(&f.plan(), "changes").unwrap().is_empty());
    let receipt = read_json(&f.vault.join(field(&result, "receipt").unwrap())).unwrap();
    rollback(&f.vault, &receipt).unwrap();
    assert_eq!(fs::read(f.vault.join(STATE)).unwrap(), before);
    for p in RETIRED_GO {
        assert_eq!(fs::read(f.vault.join(p)).unwrap(), b"old owned instruction");
    }
    assert!(!f.vault.join(".agents/skills/github-work/SKILL.md").exists());
}
#[test]
fn retirement_refuses_missing_successor_drift_symlink_and_old_package_reintroduction() {
    let f = github_rename_fixture();
    let p = f.plan();
    atomic(&f.vault, RETIRED_GO[0], b"human edit").unwrap();
    assert!(apply(&f.vault, &p).is_err());
    atomic(&f.vault, RETIRED_GO[0], b"old owned instruction").unwrap();
    fs::remove_file(f.vault.join(RETIRED_GO[0])).unwrap();
    std::os::unix::fs::symlink(f.vault.join("AGENTS.md"), f.vault.join(RETIRED_GO[0])).unwrap();
    assert!(plan(&f.vault, &f.package, &json!({})).is_err());
    fs::remove_file(f.vault.join(RETIRED_GO[0])).unwrap();
    atomic(&f.vault, RETIRED_GO[0], b"old owned instruction").unwrap();
    let mut m = read_json(&f.package.join("package.json")).unwrap();
    m["files"].as_array_mut().unwrap().pop();
    fs::write(
        f.package.join("package.json"),
        serde_json::to_vec(&m).unwrap(),
    )
    .unwrap();
    assert!(plan(&f.vault, &f.package, &json!({})).is_err());
    m["files"]
        .as_array_mut()
        .unwrap()
        .push(json!({"source":"policy.md","target":RETIRED_GO[0],"sha256":hash(b"one")}));
    fs::write(
        f.package.join("package.json"),
        serde_json::to_vec(&m).unwrap(),
    )
    .unwrap();
    assert!(package(&f.package).is_err());
    assert!(!RETIRED_GO.contains(&".agents/skills/go-extra/PROCEDURE.md"));
    assert!(!allowed(".agents/skills/go-extra/PROCEDURE.md"));
}
#[test]
fn interrupted_retirement_rolls_back_and_recreated_file_blocks_undo() {
    let f = github_rename_fixture();
    let before = fs::read(f.vault.join(STATE)).unwrap();
    let result = apply(&f.vault, &f.plan()).unwrap();
    let receipt = read_json(&f.vault.join(field(&result, "receipt").unwrap())).unwrap();
    atomic(&f.vault, RETIRED_GO[0], b"new user file").unwrap();
    assert!(rollback(&f.vault, &receipt).is_err());
    fs::remove_file(f.vault.join(RETIRED_GO[0])).unwrap();
    rollback(&f.vault, &receipt).unwrap();
    // Reconstruct the interruption immediately after deleting the first old file.
    let pending = receipt;
    atomic(
        &f.vault,
        ".viggobsidian/pending.json",
        &serde_json::to_vec(&pending).unwrap(),
    )
    .unwrap();
    for target in SKILLS
        .iter()
        .map(|name| format!(".agents/skills/{name}/SKILL.md"))
    {
        atomic(&f.vault, &target, b"one").unwrap();
    }
    fs::remove_file(f.vault.join(RETIRED_GO[0])).unwrap();
    rollback(&f.vault, &pending).unwrap();
    for p in RETIRED_GO {
        assert_eq!(fs::read(f.vault.join(p)).unwrap(), b"old owned instruction");
    }
    assert_eq!(fs::read(f.vault.join(STATE)).unwrap(), before);
    assert!(!f.vault.join(".agents/skills/github-work/SKILL.md").exists());
}

#[test]
fn finite_rename_does_not_remove_unmanaged_old_paths() {
    let f = github_rename_fixture();
    rollback(
        &f.vault,
        &read_json(
            &f.vault
                .join(field(&apply(&f.vault, &f.plan()).unwrap(), "receipt").unwrap()),
        )
        .unwrap(),
    )
    .unwrap();
    fs::remove_file(f.vault.join(STATE)).unwrap();
    let mut adoption = serde_json::Map::new();
    adoption.insert(".viggobsidian/core/policy.md".into(), json!(hash(b"one")));
    let p = plan(&f.vault, &f.package, &Value::Object(adoption)).unwrap();
    assert!(
        rows(&p, "changes")
            .unwrap()
            .iter()
            .all(|c| !c["after"].is_null())
    );
    apply(&f.vault, &p).unwrap();
    for target in RETIRED_GO {
        assert_eq!(
            fs::read(f.vault.join(target)).unwrap(),
            b"old owned instruction"
        );
    }
}

#[test]
fn ten_core_surface_has_no_procedure_or_module_backdoor() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let names: BTreeSet<_> = fs::read_dir(root.join("skills"))
        .unwrap()
        .map(|p| p.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(names, SKILLS.iter().map(|n| n.to_string()).collect());
    for row in rows(
        &read_json(&root.join("schemas/skill-library.json")).unwrap(),
        "files",
    )
    .unwrap()
    {
        let target = field(row, "target").unwrap();
        assert!(!target.ends_with("PROCEDURE.md"));
        assert!(!target.contains("/_bundle/") && !target.contains("/modules/"));
        if let Some(path) = target.strip_prefix(".agents/skills/") {
            assert!(SKILLS.contains(&path.split('/').next().unwrap()));
        }
    }
}

#[test]
fn full_reduction_retires_only_exact_managed_preimages_and_restores_them() {
    let f = github_rename_fixture();
    let mut previous = state(&f.vault).unwrap();
    let mut manifest: Value =
        serde_json::from_str(previous["manifest_bytes"].as_str().unwrap()).unwrap();
    let historical: Vec<String> =
        serde_json::from_str(include_str!("../schemas/retired-skill-targets.json")).unwrap();
    let retired: Vec<_> = historical
        .iter()
        .filter(|target| !allowed(target))
        .collect();
    assert!(retired.len() > 200);
    for target in &retired {
        atomic(&f.vault, target, b"original managed content").unwrap();
        previous["files"][*target] = json!(hash(b"original managed content"));
        manifest["files"].as_array_mut().unwrap().push(
            json!({"source":"old.md","target":target,"sha256":hash(b"original managed content")}),
        );
    }
    let raw = serde_json::to_string(&manifest).unwrap();
    previous["manifest_bytes"] = json!(raw);
    previous["manifest_sha256"] = json!(hash(raw.as_bytes()));
    atomic(&f.vault, STATE, &serde_json::to_vec(&previous).unwrap()).unwrap();
    atomic(
        &f.vault,
        ".agents/skills/vault-read/scripts/provider.py",
        b"local exclusive provider",
    )
    .unwrap();
    atomic(
        &f.vault,
        ".agents/skills/unknown/PROCEDURE.md",
        b"unmanaged owner",
    )
    .unwrap();
    let before = fs::read(f.vault.join(STATE)).unwrap();
    let plan = f.plan();
    assert_eq!(
        rows(&plan, "changes")
            .unwrap()
            .iter()
            .filter(|c| c["after"].is_null())
            .count(),
        retired.len() + RETIRED_GO.len()
    );
    let applied = apply(&f.vault, &plan).unwrap();
    assert!(retired.iter().all(|p| !f.vault.join(p).exists()));
    assert_eq!(
        fs::read(
            f.vault
                .join(".agents/skills/vault-read/scripts/provider.py")
        )
        .unwrap(),
        b"local exclusive provider"
    );
    assert_eq!(
        fs::read(f.vault.join(".agents/skills/unknown/PROCEDURE.md")).unwrap(),
        b"unmanaged owner"
    );
    let receipt = read_json(&f.vault.join(field(&applied, "receipt").unwrap())).unwrap();
    rollback(&f.vault, &receipt).unwrap();
    assert_eq!(fs::read(f.vault.join(STATE)).unwrap(), before);
    for target in retired {
        assert_eq!(
            fs::read(f.vault.join(target)).unwrap(),
            b"original managed content"
        );
    }
}

#[test]
fn complete_distribution_is_inside_current_package_write_allowlist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for row in rows(
        &read_json(&root.join("distribution.json")).unwrap(),
        "files",
    )
    .unwrap()
    {
        assert!(allowed(field(row, "target").unwrap()), "{}", row["target"]);
    }
}

#[test]
fn installed_skills_reference_has_its_exact_plugin_check_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let distribution = read_json(&root.join("distribution.json")).unwrap();
    let source = "docs/plugin-check.md";
    let target = ".viggobsidian/core/plugin-check.md";
    assert!(allowed(target));
    assert!(!allowed(".viggobsidian/core/unreviewed-plugin-contract.md"));
    assert!(root.join(source).is_file());
    assert_eq!(
        rows(&distribution, "files")
            .unwrap()
            .iter()
            .filter(|row| row["source"] == source && row["target"] == target)
            .count(),
        1
    );
    assert!(
        fs::read_to_string(root.join("docs/skills.md"))
            .unwrap()
            .contains("[plugin-check](plugin-check.md)")
    );
}
