use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn fixture() -> (PathBuf, PathBuf, String, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "viggobsidian-dossier-ask-test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    let vault = root.join("vault");
    let dossier = "30-topics/fictief-dossier".to_owned();
    let folder = vault.join(&dossier);
    fs::create_dir_all(folder.join("wiki")).unwrap();
    fs::create_dir_all(folder.join("raw")).unwrap();
    fs::write(folder.join("fictief-dossier.md"), "# Fictief dossier\n").unwrap();
    fs::write(
        folder.join("wiki/evidence.md"),
        "# Fictieve Wiki\n\n## Garantie\n\nDe fictieve garantie is twee jaar.\n",
    )
    .unwrap();
    (root.clone(), vault, dossier, root.join("cache"))
}

fn ask(vault: &Path, dossier: &str, cache: &Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_viggobsidian"))
        .args([
            "dossier-ask",
            "--vault",
            vault.to_str().unwrap(),
            "--dossier",
            dossier,
            "--cache-dir",
            cache.to_str().unwrap(),
            "--query",
            "fictieve garantie",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn canonical_dossier_ask_uses_the_shared_route_without_vault_writes() {
    let (root, vault, dossier, cache) = fixture();
    let evidence = vault.join(&dossier).join("wiki/evidence.md");
    let original = fs::read(&evidence).unwrap();

    let cold = ask(&vault, &dossier, &cache);
    assert_eq!(cold["schema"], "viggobsidian.dossier-ask.v1");
    assert_eq!(cold["status"], "complete");
    assert_eq!(cold["cache"]["outcome"], "built");
    assert_eq!(fs::read(&evidence).unwrap(), original);

    let warm = ask(&vault, &dossier, &cache);
    assert_eq!(warm["cache"]["outcome"], "reused");
    assert_eq!(fs::read(&evidence).unwrap(), original);

    fs::write(
        &evidence,
        "# Fictieve Wiki\n\n## Garantie\n\nDe fictieve garantie is drie jaar.\n",
    )
    .unwrap();
    let changed = ask(&vault, &dossier, &cache);
    assert_eq!(changed["cache"]["outcome"], "rebuilt");
    assert_eq!(
        fs::read_to_string(&evidence).unwrap(),
        "# Fictieve Wiki\n\n## Garantie\n\nDe fictieve garantie is drie jaar.\n"
    );

    fs::remove_dir_all(root).unwrap();
}
