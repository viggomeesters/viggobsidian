//! Read-only Work acceptance: visit every collection even after a finding.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs,
    path::{Component, Path},
    process::Command,
};

fn relative_file(root: &Path, name: &str) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let rel = Path::new(name);
    if rel.as_os_str().is_empty() || !rel.components().all(|p| matches!(p, Component::Normal(_))) {
        return Err(format!("invalid package path: {name}").into());
    }
    let mut path = root.to_path_buf();
    for part in rel.components() {
        path.push(part);
        if fs::symlink_metadata(&path)
            .map_err(|error| format!("package file {name}: {error}"))?
            .is_symlink()
        {
            return Err(format!("package symlink refused: {name}").into());
        }
    }
    if !path.is_file() {
        return Err(format!("missing package file: {name}").into());
    }
    Ok(path)
}

fn verify_package(root: &Path) -> Result<(), Box<dyn Error>> {
    let manifest: Value = serde_json::from_slice(&fs::read(
        root.join("99-system/contracts/work-package.json"),
    )?)?;
    if manifest["version"] != 1 {
        return Err("unsupported Work package manifest".into());
    }
    let enabled: Vec<String> =
        serde_json::from_slice(&fs::read(root.join(".obsidian/community-plugins.json"))?)?;
    let required = manifest["required_files"]
        .as_array()
        .ok_or("missing required_files")?;
    if required.is_empty() {
        return Err("empty Work package".into());
    }
    for entry in required {
        relative_file(root, entry.as_str().ok_or("invalid required file")?)?;
    }
    let pins = manifest["plugin_files"]
        .as_array()
        .ok_or("missing plugin_files")?;
    if pins.is_empty() {
        return Err("empty plugin inventory".into());
    }
    for pin in pins {
        let name = pin["path"].as_str().ok_or("missing plugin path")?;
        let bytes = fs::read(relative_file(root, name)?)?;
        let actual = format!("{:x}", Sha256::digest(&bytes));
        if pin["sha256"].as_str() != Some(&actual) {
            return Err(format!("plugin drift: {name}").into());
        }
    }
    for id in [
        "recent-files-obsidian",
        "newest-files",
        "outlook-drop-importer",
    ] {
        if !enabled.iter().any(|value| value == id) {
            return Err(format!("plugin disabled: {id}").into());
        }
        let meta: Value = serde_json::from_slice(&fs::read(relative_file(
            root,
            &format!(".obsidian/plugins/{id}/manifest.json"),
        )?)?)?;
        if meta["id"] != id {
            return Err(format!("plugin identity mismatch: {id}").into());
        }
    }
    println!("Work package: files, plugin hashes and enabled configuration OK");
    Ok(())
}

pub(crate) fn check(root: &Path) -> Result<u8, Box<dyn Error>> {
    check_all(root, true, true)
}

pub(crate) fn check_all(root: &Path, package: bool, links: bool) -> Result<u8, Box<dyn Error>> {
    let exe = std::env::current_exe()?;
    let mut failed = false;
    if package && let Err(error) = verify_package(root) {
        eprintln!("Work package: {error}");
        failed = true;
    }
    let mut roots = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
    roots.sort_by_key(|entry| entry.file_name());
    let mut entrances = vec!["HOME.md".to_owned()];
    for entry in roots {
        let name = entry.file_name().to_string_lossy().into_owned();
        let b = name.as_bytes();
        if b.len() < 4 || !b[0].is_ascii_digit() || !b[1].is_ascii_digit() || b[2] != b'-' {
            continue;
        }
        if !entry.file_type()?.is_dir() {
            return Err(format!("collection is not a directory: {name}").into());
        }
        println!("Checking {name}");
        failed |= !Command::new(&exe)
            .args(["--vault", &root.to_string_lossy(), "--strict", &name])
            .status()?
            .success();
        entrances.push(format!("{name}/{name}.md"));
    }
    failed |= !Command::new(&exe)
        .arg("--vault")
        .arg(root)
        .arg("--tiles-only")
        .status()?
        .success();
    // Explicitly include human system guides and actual templates, not only root entrances.
    for folder in ["99-system/vault", "90-templates"] {
        for entry in fs::read_dir(root.join(folder))? {
            let path = entry?.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                entrances.push(path.strip_prefix(root)?.to_string_lossy().into_owned());
            }
        }
    }
    entrances.sort();
    entrances.dedup();
    failed |= !Command::new(&exe)
        .arg("--vault")
        .arg(root)
        .args(["--indexes", "--paths"])
        .args(entrances)
        .status()?
        .success();
    if links {
        failed |= crate::link_style::run(root, false, false)? != 0;
    }
    Ok(u8::from(failed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../.tmp")
                .join(format!(
                    "work-package-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
            fs::create_dir_all(root.join("99-system/contracts")).unwrap();
            let ids = [
                "recent-files-obsidian",
                "newest-files",
                "outlook-drop-importer",
            ];
            fs::create_dir(root.join(".obsidian")).unwrap();
            fs::write(
                root.join(".obsidian/community-plugins.json"),
                serde_json::to_vec(&ids).unwrap(),
            )
            .unwrap();
            let mut pins = Vec::new();
            for id in ids {
                let path = format!(".obsidian/plugins/{id}/manifest.json");
                fs::create_dir_all(root.join(&path).parent().unwrap()).unwrap();
                let bytes = serde_json::to_vec(&json!({"id": id})).unwrap();
                fs::write(root.join(&path), &bytes).unwrap();
                pins.push(json!({"path":path,"sha256":format!("{:x}",Sha256::digest(&bytes))}));
            }
            fs::write(root.join("guide.md"), "Work rules").unwrap();
            fs::write(
                root.join("99-system/contracts/work-package.json"),
                serde_json::to_vec(&json!({
                    "version":1,"required_files":["guide.md"],"plugin_files":pins
                }))
                .unwrap(),
            )
            .unwrap();
            Self(root)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn package_checks_content_presence_and_enabled_state() {
        let fixture = Fixture::new();
        assert!(verify_package(&fixture.0).is_ok());
        fs::remove_file(fixture.0.join("guide.md")).unwrap();
        assert!(
            verify_package(&fixture.0)
                .unwrap_err()
                .to_string()
                .contains("guide.md")
        );
        fs::write(fixture.0.join("guide.md"), "Work rules").unwrap();
        let path = fixture
            .0
            .join(".obsidian/plugins/newest-files/manifest.json");
        let original = fs::read(&path).unwrap();
        fs::write(&path, "tampered").unwrap();
        assert!(
            verify_package(&fixture.0)
                .unwrap_err()
                .to_string()
                .contains("plugin drift")
        );
        fs::write(&path, original).unwrap();
        fs::write(fixture.0.join(".obsidian/community-plugins.json"), "[]").unwrap();
        assert!(
            verify_package(&fixture.0)
                .unwrap_err()
                .to_string()
                .contains("plugin disabled")
        );
    }
    #[test]
    fn package_cannot_escape_vault() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(relative_file(root, "../Cargo.toml").is_err());
        assert!(relative_file(root, "/etc/passwd").is_err());
        assert!(relative_file(root, "Cargo.toml").is_ok());
    }
}
