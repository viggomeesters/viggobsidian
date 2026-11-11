use super::*;

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
