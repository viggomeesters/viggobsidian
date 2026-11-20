//! Explicit installer-authenticated local provider sources, separate from package bindings.
use super::{Result, bytes, execute_snapshot, hash, regular, text};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    io::Write,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Component, Path},
};

fn relative(value: &str) -> Result<&Path> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('\\')
        || !path.components().all(|p| matches!(p, Component::Normal(_)))
    {
        return Err("provider source path must be relative and canonical".into());
    }
    Ok(path)
}
fn collect(
    root: &Path,
    tree: &Path,
    files: &mut BTreeMap<String, String>,
    total: &mut usize,
    visited: &mut usize,
) -> Result<()> {
    *visited = visited
        .checked_add(1)
        .ok_or("provider source entry overflow")?;
    if *visited > 65_536 {
        return Err("provider source entry limit".into());
    }
    if tree.components().count() > 64 {
        return Err("provider source depth limit".into());
    }
    let path = root.join(tree);
    let mut cursor = root.to_owned();
    for component in tree.components() {
        cursor.push(component);
        let meta = fs::symlink_metadata(&cursor)?;
        if meta.file_type().is_symlink() {
            return Err("provider source symlink ancestor refused".into());
        }
        if cursor != path && !meta.is_dir() {
            return Err("provider source ancestor must be directory".into());
        }
    }
    let meta = fs::symlink_metadata(&path)?;
    if meta.file_type().is_symlink() {
        return Err("provider source symlink refused".into());
    }
    if meta.is_dir() {
        for entry in fs::read_dir(path)? {
            collect(root, &tree.join(entry?.file_name()), files, total, visited)?;
        }
    } else if meta.is_file() {
        if files.len() >= 65_536 {
            return Err("provider source file limit".into());
        }
        let data = bytes(&path)?;
        *total = total
            .checked_add(data.len())
            .ok_or("provider source size overflow")?;
        if *total > 128 * 1024 * 1024 {
            return Err("provider source size limit".into());
        }
        files.insert(
            tree.to_str()
                .ok_or("provider source path is not UTF-8")?
                .into(),
            hash(&data),
        );
    } else {
        return Err("provider source special file refused".into());
    }
    Ok(())
}
fn sources(manifest: &Path, digest: &str) -> Result<Value> {
    let data = bytes(manifest)?;
    if hash(&data) != digest {
        return Err("provider source manifest changed".into());
    }
    let value: Value = serde_json::from_slice(&data)?;
    if value["schema"] != "viggobsidian.provider-source.v1" {
        return Err("unknown provider source schema".into());
    }
    for field in ["tool", "version", "revision"] {
        let v = text(&value, field)?;
        if v.is_empty() || v.len() > 256 {
            return Err("provider identity/version/revision required".into());
        }
    }
    let root = Path::new(text(&value, "root")?);
    if !root.is_absolute() || root.canonicalize()? != root || !root.is_dir() {
        return Err("provider root must be canonical directory".into());
    }
    let tree = relative(text(&value, "tree")?)?;
    regular(&root.join(tree).join("Cargo.toml"))?;
    regular(&root.join(tree).join("Cargo.lock"))?;
    let mut actual = BTreeMap::new();
    let mut total = 0;
    let mut visited = 0;
    collect(root, tree, &mut actual, &mut total, &mut visited)?;
    if let Some(extras) = value.get("extra_sources") {
        let extras = extras.as_array().ok_or("extra_sources must be an array")?;
        if extras.len() > 64 {
            return Err("provider extra source root limit".into());
        }
        let mut roots = vec![tree.to_owned()];
        for extra in extras {
            let extra = relative(extra.as_str().ok_or("extra source path must be text")?)?;
            if roots
                .iter()
                .any(|p| p.starts_with(extra) || extra.starts_with(p))
            {
                return Err("overlapping provider source roots".into());
            }
            roots.push(extra.to_owned());
            collect(root, extra, &mut actual, &mut total, &mut visited)?;
        }
    }
    if !actual.keys().any(|p| p.ends_with(".rs")) {
        return Err("provider Rust sources missing".into());
    }
    if serde_json::to_value(actual)? != value["files"] {
        return Err("provider complete source tree changed".into());
    }
    Ok(value)
}
fn binding(manifest: &Path, manifest_sha: &str, binary: &Path, binary_sha: &str) -> Result<Value> {
    let source = sources(manifest, manifest_sha)?;
    let root = Path::new(text(&source, "root")?);
    if manifest.starts_with(root) || binary.starts_with(root) {
        return Err("provider manifest and executable must be outside source root".into());
    }
    let data = bytes(binary)?;
    if hash(&data) != binary_sha || fs::metadata(binary)?.permissions().mode() & 0o111 == 0 {
        return Err("provider executable changed or not executable".into());
    }
    Ok(
        json!({"schema":"viggobsidian.provider-binding.v1","source_manifest":manifest,"source_manifest_sha256":manifest_sha,
        "source":source,"binary":binary,"binary_sha256":binary_sha,"host":format!("{}-{}",std::env::consts::ARCH,std::env::consts::OS)}),
    )
}
fn execute(value: &Value, args: &[OsString]) -> Result<i32> {
    if value["schema"] != "viggobsidian.provider-binding.v1" {
        return Err("unknown provider binding schema".into());
    }
    if binding(
        Path::new(text(value, "source_manifest")?),
        text(value, "source_manifest_sha256")?,
        Path::new(text(value, "binary")?),
        text(value, "binary_sha256")?,
    )? != *value
    {
        return Err("provider binding changed".into());
    }
    execute_snapshot(
        Path::new(text(&value["source"], "root")?),
        Path::new(text(value, "binary")?),
        text(value, "binary_sha256")?,
        args,
    )
}
pub(super) fn run_args(args: &[OsString]) -> Result<i32> {
    match args.first().and_then(|s|s.to_str()){
        Some("bind-provider") if args.len()==6=>{
            let value=binding(Path::new(&args[1]),args[2].to_str().ok_or("digest")?,Path::new(&args[3]),args[4].to_str().ok_or("digest")?)?;
            let output=Path::new(&args[5]);let parent=output.parent().ok_or("output parent")?;
            if !output.is_absolute()||parent.canonicalize()?!=parent||output.starts_with(Path::new(text(&value["source"],"root")?)){return Err("provider binding output must be canonical and outside source root".into());}
            let data=serde_json::to_vec_pretty(&value)?;
            let mut file=fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(output)?;file.write_all(&data)?;file.sync_all()?;println!("{}",hash(&data));Ok(0)
        }
        Some("run-provider") if args.len()>=4&&args[3]=="--"=>{
            let data=bytes(Path::new(&args[1]))?;if Some(hash(&data).as_str())!=args[2].to_str(){return Err("provider binding digest changed".into());}execute(&serde_json::from_slice(&data)?,&args[4..])
        }
        _=>Err("usage: vault-validator bind-provider SOURCE_MANIFEST SOURCE_MANIFEST_SHA BINARY BINARY_SHA OUTPUT | run-provider BINDING BINDING_SHA -- ARGS...".into())
    }
}

#[cfg(test)]
mod tests {
    use super::super::Snapshot;
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    #[test]
    fn global_entry_budget_counts_empty_directories_before_collecting_children() {
        let f = Fixture::new();
        fs::create_dir_all(f.root.join("empty/child")).unwrap();
        let mut files = BTreeMap::new();
        let mut total = 0;
        let mut visited = 65_535;
        assert!(
            collect(
                &f.root,
                Path::new("empty"),
                &mut files,
                &mut total,
                &mut visited
            )
            .unwrap_err()
            .to_string()
            .contains("entry limit")
        );
        assert!(files.is_empty());
        assert_eq!(total, 0);
    }
    #[test]
    fn explicitly_declared_shared_source_is_authenticated() {
        let f = Fixture::new();
        let shared = "shared/common.rs";
        fs::create_dir(f.root.join("shared")).unwrap();
        fs::write(f.root.join(shared), b"shared dependency").unwrap();
        let mut manifest: Value = serde_json::from_slice(&fs::read(&f.manifest).unwrap()).unwrap();
        manifest["extra_sources"] = json!([shared]);
        manifest["files"][shared] = json!(hash(b"shared dependency"));
        fs::write(&f.manifest, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let b = f.binding();
        fs::write(f.root.join(shared), b"changed shared dependency").unwrap();
        assert!(
            execute(&b, &[])
                .unwrap_err()
                .to_string()
                .contains("source tree changed")
        );
    }
    #[test]
    fn empty_extra_directory_with_symlink_ancestor_is_refused() {
        let f = Fixture::new();
        fs::create_dir_all(f.root.join("real/empty")).unwrap();
        std::os::unix::fs::symlink("real", f.root.join("alias")).unwrap();
        let mut manifest: Value = serde_json::from_slice(&fs::read(&f.manifest).unwrap()).unwrap();
        manifest["extra_sources"] = json!(["alias/empty"]);
        fs::write(&f.manifest, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let error = binding(
            &f.manifest,
            &hash(&bytes(&f.manifest).unwrap()),
            &f.binary,
            &hash(&bytes(&f.binary).unwrap()),
        )
        .unwrap_err();
        assert!(error.to_string().contains("symlink ancestor"));
    }
    struct Fixture {
        _folder: Snapshot,
        manifest: std::path::PathBuf,
        binary: std::path::PathBuf,
        root: std::path::PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let folder = std::env::temp_dir().canonicalize().unwrap().join(format!(
                "provider-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&folder).unwrap();
            let root = folder.join("fictional-root");
            let tree = root.join("providers/example");
            fs::create_dir_all(&tree).unwrap();
            let mut files = BTreeMap::new();
            for n in ["Cargo.toml", "Cargo.lock", "main.rs"] {
                fs::write(tree.join(n), b"fictional").unwrap();
                files.insert(format!("providers/example/{n}"), hash(b"fictional"));
            }
            let manifest = folder.join("source.json");
            fs::write(&manifest,serde_json::to_vec(&json!({"schema":"viggobsidian.provider-source.v1","root":root,"tree":"providers/example","tool":"example","version":"0.1.0","revision":"fictional","files":files})).unwrap()).unwrap();
            let binary = folder.join("binary");
            fs::copy(std::env::current_exe().unwrap(), &binary).unwrap();
            Self {
                _folder: Snapshot(folder),
                manifest,
                binary,
                root,
            }
        }
        fn binding(&self) -> Value {
            binding(
                &self.manifest,
                &hash(&bytes(&self.manifest).unwrap()),
                &self.binary,
                &hash(&bytes(&self.binary).unwrap()),
            )
            .unwrap()
        }
    }
    #[test]
    fn provider_executes_without_cargo_or_vault_tmp() {
        let f = Fixture::new();
        assert_eq!(execute(&f.binding(), &["--list".into()]).unwrap(), 0);
        assert!(!f.root.join(".tmp").exists());
    }
    #[test]
    fn extra_or_modified_source_refuses() {
        let f = Fixture::new();
        let b = f.binding();
        fs::write(f.root.join("providers/example/extra.rs"), b"injected").unwrap();
        assert!(
            execute(&b, &[])
                .unwrap_err()
                .to_string()
                .contains("source tree changed")
        );
    }
    #[test]
    fn manifest_mutation_refuses() {
        let f = Fixture::new();
        let b = f.binding();
        fs::write(&f.manifest, b"{}").unwrap();
        assert!(
            execute(&b, &[])
                .unwrap_err()
                .to_string()
                .contains("manifest changed")
        );
    }
    #[test]
    fn binary_and_host_mutation_refuse() {
        let f = Fixture::new();
        let mut b = f.binding();
        b["host"] = json!("other-host");
        assert!(execute(&b, &[]).is_err());
        fs::write(&f.binary, b"changed").unwrap();
        assert!(execute(&b, &[]).is_err());
    }
    #[test]
    fn source_symlink_refuses() {
        let f = Fixture::new();
        let b = f.binding();
        std::os::unix::fs::symlink("main.rs", f.root.join("providers/example/linked.rs")).unwrap();
        assert!(
            execute(&b, &[])
                .unwrap_err()
                .to_string()
                .contains("symlink")
        );
    }
}
