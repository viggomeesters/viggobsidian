//! Run an explicitly installed validator without invoking a producer build.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const LIMIT: u64 = 128 * 1024 * 1024;
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn regular(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err("absolute path required".into());
    }
    let mut cursor = PathBuf::new();
    for part in path.components() {
        if matches!(part, Component::ParentDir | Component::CurDir) {
            return Err("noncanonical path".into());
        }
        cursor.push(part);
        if fs::symlink_metadata(&cursor)?.file_type().is_symlink() {
            return Err("symlink refused".into());
        }
    }
    if !fs::metadata(path)?.is_file() {
        return Err("regular file required".into());
    }
    Ok(())
}
fn bytes(path: &Path) -> Result<Vec<u8>> {
    regular(path)?;
    let mut data = Vec::new();
    fs::File::open(path)?
        .take(LIMIT + 1)
        .read_to_end(&mut data)?;
    if data.len() as u64 > LIMIT {
        return Err("file exceeds validator limit".into());
    }
    Ok(data)
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("missing {key}").into())
}
fn sources(vault: &Path, installed_sha: &str) -> Result<Value> {
    let installed = bytes(&vault.join(".viggobsidian/installed.json"))?;
    if hash(&installed) != installed_sha {
        return Err("installed package changed".into());
    }
    let state: Value = serde_json::from_slice(&installed)?;
    let mut sources = BTreeMap::new();
    for (path, expected) in state["files"]
        .as_object()
        .ok_or("missing installed files")?
    {
        if !path.starts_with(".agents/tools/") {
            continue;
        }
        if Path::new(path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
            || path.contains('\\')
        {
            return Err("unsafe source path".into());
        }
        let actual = hash(&bytes(&vault.join(path))?);
        if expected.as_str() != Some(&actual) {
            return Err(format!("installed source drift: {path}").into());
        }
        sources.insert(path, actual);
    }
    if !sources.contains_key(&".agents/tools/vault-layout/Cargo.toml".to_string())
        || !sources.contains_key(&".agents/tools/vault-layout/Cargo.lock".to_string())
        || !sources
            .keys()
            .any(|p| p.starts_with(".agents/tools/vault-layout/src/") && p.ends_with(".rs"))
    {
        return Err("installed validator sources missing".into());
    }
    Ok(serde_json::to_value(sources)?)
}
fn binding(vault: &Path, binary: &Path, installed_sha: &str, binary_sha: &str) -> Result<Value> {
    let actual = hash(&bytes(binary)?);
    if actual != binary_sha {
        return Err("validator binary changed".into());
    }
    if fs::metadata(binary)?.permissions().mode() & 0o111 == 0 {
        return Err("validator is not executable".into());
    }
    Ok(
        json!({"schema":"viggobsidian.validator-binding.v1", "vault":vault, "binary":binary,
        "binary_sha256":actual, "installed_sha256":installed_sha,
        "host":format!("{}-{}",std::env::consts::ARCH,std::env::consts::OS),
        "sources":sources(vault, installed_sha)?}),
    )
}
struct Snapshot(PathBuf);
impl Drop for Snapshot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn execute(value: &Value, args: &[std::ffi::OsString]) -> Result<i32> {
    let vault = Path::new(text(value, "vault")?);
    let binary = Path::new(text(value, "binary")?);
    if binding(
        vault,
        binary,
        text(value, "installed_sha256")?,
        text(value, "binary_sha256")?,
    )? != *value
    {
        return Err("validator binding changed".into());
    }
    // Execute a private, verified copy: changing the installed path after validation
    // must not substitute the bytes executed by this invocation.
    let parent = std::env::temp_dir().canonicalize()?;
    if parent.starts_with(vault) {
        return Err("runtime temporary directory is inside vault".into());
    }
    let token = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let folder = parent.join(format!("vault-validator-{}-{token}", std::process::id()));
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new().mode(0o700).create(&folder)?;
    let snapshot = Snapshot(folder);
    let data = bytes(binary)?;
    if hash(&data) != text(value, "binary_sha256")? {
        return Err("validator changed before snapshot".into());
    }
    let executable = snapshot.0.join("validator");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o700)
        .open(&executable)?;
    file.write_all(&data)?;
    file.sync_all()?;
    drop(file);
    let status = Command::new(&executable).args(args).status()?;
    Ok(status.code().unwrap_or(1))
}
fn run() -> Result<i32> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    run_args(&args)
}
fn run_args(args: &[std::ffi::OsString]) -> Result<i32> {
    if args == ["--version"] {
        println!("vault-validator {}", env!("CARGO_PKG_VERSION"));
        return Ok(0);
    }
    match args.first().and_then(|s|s.to_str()) {
        Some("bind") if args.len()==6 => {
            let value = binding(Path::new(&args[1]),Path::new(&args[2]),args[3].to_str().ok_or("invalid digest")?,args[4].to_str().ok_or("invalid digest")?)?;
            let output = Path::new(&args[5]);
            if !output.is_absolute() { return Err("absolute output required".into()); }
            let parent = output.parent().ok_or("missing parent")?;
            if parent.canonicalize()? != parent || !parent.is_dir() { return Err("output parent must be a canonical directory".into()); }
            if output.starts_with(Path::new(&args[1])) { return Err("binding output must be outside vault".into()); }
            let data = serde_json::to_vec_pretty(&value)?;
            let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(output)?;
            file.write_all(&data)?; file.sync_all()?;
            println!("{}",hash(&data));
            Ok(0)
        }
        Some("run") if args.len()>=4 && args[3]=="--" => {
            let data = bytes(Path::new(&args[1]))?;
            if Some(hash(&data).as_str()) != args[2].to_str() { return Err("binding digest changed".into()); }
            execute(&serde_json::from_slice(&data)?,&args[4..])
        }
        _ => Err("usage: vault-validator bind VAULT BINARY INSTALLED_SHA256 BINARY_SHA256 OUTPUT | run BINDING BINDING_SHA256 -- ARGS...".into())
    }
}
fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("vault-validator: {error}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture {
        folder: Snapshot,
        vault: PathBuf,
        binary: PathBuf,
        installed_sha: String,
    }
    impl Fixture {
        fn new() -> Self {
            let folder = std::env::temp_dir().canonicalize().unwrap().join(format!(
                "validator-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&folder).unwrap();
            let vault = folder.join("vault");
            fs::create_dir_all(vault.join(".viggobsidian")).unwrap();
            let mut files = serde_json::Map::new();
            for source in ["Cargo.toml", "Cargo.lock", "src/main.rs"] {
                let path = format!(".agents/tools/vault-layout/{source}");
                let absolute = vault.join(&path);
                fs::create_dir_all(absolute.parent().unwrap()).unwrap();
                fs::write(absolute, b"fictional source").unwrap();
                files.insert(path, json!(hash(b"fictional source")));
            }
            let installed = serde_json::to_vec(&json!({"files":files})).unwrap();
            fs::write(vault.join(".viggobsidian/installed.json"), &installed).unwrap();
            let binary = folder.join("validator");
            fs::copy(std::env::current_exe().unwrap(), &binary).unwrap();
            Self {
                folder: Snapshot(folder),
                vault,
                binary,
                installed_sha: hash(&installed),
            }
        }
        fn binding(&self) -> Value {
            binding(
                &self.vault,
                &self.binary,
                &self.installed_sha,
                &hash(&bytes(&self.binary).unwrap()),
            )
            .unwrap()
        }
    }
    #[test]
    fn verified_binary_executes_without_build_output() {
        let fixture = Fixture::new();
        assert_eq!(execute(&fixture.binding(), &["--list".into()]).unwrap(), 0);
        assert!(!fixture.vault.join(".tmp").exists());
        assert!(fixture.folder.0.exists());
    }
    #[test]
    fn source_drift_refuses_before_execution() {
        let fixture = Fixture::new();
        let value = fixture.binding();
        fs::write(
            fixture.vault.join(".agents/tools/vault-layout/src/main.rs"),
            b"changed",
        )
        .unwrap();
        assert!(
            execute(&value, &[])
                .unwrap_err()
                .to_string()
                .contains("source drift")
        );
    }
    #[test]
    fn installed_manifest_drift_refuses() {
        let fixture = Fixture::new();
        let value = fixture.binding();
        fs::write(fixture.vault.join(".viggobsidian/installed.json"), b"{}").unwrap();
        assert!(
            execute(&value, &[])
                .unwrap_err()
                .to_string()
                .contains("package changed")
        );
    }
    #[test]
    fn binary_drift_and_host_binding_drift_refuse() {
        let fixture = Fixture::new();
        let mut value = fixture.binding();
        value["host"] = json!("different-host");
        assert!(
            execute(&value, &[])
                .unwrap_err()
                .to_string()
                .contains("binding changed")
        );
        let value = fixture.binding();
        fs::write(&fixture.binary, b"changed").unwrap();
        assert!(
            execute(&value, &[])
                .unwrap_err()
                .to_string()
                .contains("binary changed")
        );
    }
    #[test]
    fn source_symlink_is_refused() {
        let fixture = Fixture::new();
        let value = fixture.binding();
        let path = fixture.vault.join(".agents/tools/vault-layout/src/main.rs");
        fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink(&fixture.binary, path).unwrap();
        assert!(
            execute(&value, &[])
                .unwrap_err()
                .to_string()
                .contains("symlink refused")
        );
    }

    #[test]
    fn binding_digest_and_create_only_destination_are_enforced() {
        let fixture = Fixture::new();
        let output = fixture.folder.0.join("binding.json");
        let args = vec![
            "bind".into(),
            fixture.vault.as_os_str().into(),
            fixture.binary.as_os_str().into(),
            fixture.installed_sha.clone().into(),
            hash(&bytes(&fixture.binary).unwrap()).into(),
            output.as_os_str().into(),
        ];
        assert_eq!(run_args(&args).unwrap(), 0);
        let original = bytes(&output).unwrap();
        assert!(run_args(&args).is_err());
        assert_eq!(bytes(&output).unwrap(), original);
        let mut args = vec![
            "run".into(),
            output.as_os_str().into(),
            "wrong-digest".into(),
            "--".into(),
            "--list".into(),
        ];
        assert!(
            run_args(&args)
                .unwrap_err()
                .to_string()
                .contains("digest changed")
        );
        args[2] = hash(&original).into();
        assert_eq!(run_args(&args).unwrap(), 0);
        fs::write(output, b"changed binding").unwrap();
        assert!(
            run_args(&args)
                .unwrap_err()
                .to_string()
                .contains("digest changed")
        );
    }
}
