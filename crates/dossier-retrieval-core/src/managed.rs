//! Managed, on-demand cache around the read-only dossier retrieval PoC.
//!
//! The cache is deliberately external to the selected vault.  It is a private,
//! disposable convenience layer: every warm query still delegates to
//! `query_index`, which reconstructs the complete live inventory before it
//! returns evidence.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const ASK_SCHEMA: &str = "viggobsidian.dossier-ask.v1";
const CACHE_FORMAT: &str = "viggobsidian.dossier-managed-cache.v1";
const CACHE_DIR: &str = "dossier-retrieval-managed-v1";
const PRIVATE_DIR_MODE: u32 = 0o700;
const PRIVATE_FILE_MODE: u32 = 0o600;

fn fail<T>(message: impl Into<String>) -> super::Result<T> {
    Err(message.into())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn response(status: &str, query: &str, selection: Value, fields: Value) -> Value {
    let mut output = json!({
        "schema": ASK_SCHEMA,
        "status": status,
        "query": query,
        "selection": selection,
    });
    output
        .as_object_mut()
        .expect("object literal")
        .extend(fields.as_object().cloned().unwrap_or_default());
    output
}

fn absolute(path: &Path) -> super::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        fail("cache_root must be an absolute path")
    }
}

fn reject_parent_components(path: &Path) -> super::Result<()> {
    if path
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return fail("parent component refused in cache path");
    }
    Ok(())
}

fn inspect_no_symlink_components(path: &Path, allow_missing_last: bool) -> super::Result<()> {
    let path = absolute(path)?;
    reject_parent_components(&path)?;
    let parts: Vec<_> = path.components().collect();
    let mut cursor = PathBuf::new();
    for (index, part) in parts.iter().enumerate() {
        cursor.push(part.as_os_str());
        match fs::symlink_metadata(&cursor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return fail(format!("symlink refused: {}", cursor.display()));
            }
            Ok(_) => {}
            Err(error)
                if allow_missing_last
                    && index + 1 == parts.len()
                    && error.kind() == ErrorKind::NotFound => {}
            Err(error) => return fail(format!("cannot inspect {}: {error}", cursor.display())),
        }
    }
    Ok(())
}

#[cfg(unix)]
fn private_mode(path: &Path, directory: bool) -> super::Result<()> {
    use std::os::unix::fs::MetadataExt;
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return fail(format!("symlink refused: {}", path.display()));
    }
    if directory != metadata.is_dir() {
        return fail(format!("cache path has wrong kind: {}", path.display()));
    }
    let expected = if directory {
        PRIVATE_DIR_MODE
    } else {
        PRIVATE_FILE_MODE
    };
    if metadata.mode() & 0o777 != expected {
        return fail(format!("insecure cache permissions: {}", path.display()));
    }
    Ok(())
}

#[cfg(not(unix))]
fn private_mode(path: &Path, directory: bool) -> super::Result<()> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || metadata.is_dir() != directory {
        return fail(format!("unsafe cache path: {}", path.display()));
    }
    Ok(())
}

#[cfg(unix)]
fn create_private_dir(path: &Path) -> super::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new()
        .mode(PRIVATE_DIR_MODE)
        .create(path)
        .map_err(|error| format!("cannot create cache directory {}: {error}", path.display()))?;
    private_mode(path, true)
}

#[cfg(not(unix))]
fn create_private_dir(path: &Path) -> super::Result<()> {
    fs::create_dir(path)
        .map_err(|error| format!("cannot create cache directory {}: {error}", path.display()))?;
    private_mode(path, true)
}

fn repository_root() -> super::Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "repository root unavailable".to_owned())?
        .canonicalize()
        .map_err(|error| format!("cannot canonicalize repository root: {error}"))
}

/// Create only absent descendants with private permissions. Existing ancestors
/// (such as `$HOME` and `$XDG_CACHE_HOME`) are inspected for symlinks but their
/// permissions are deliberately not changed.
fn ensure_private_cache_directory(path: &Path) -> super::Result<()> {
    let path = absolute(path)?;
    reject_parent_components(&path)?;
    let parts: Vec<_> = path.components().collect();
    let mut cursor = PathBuf::new();
    for (index, part) in parts.iter().enumerate() {
        cursor.push(part.as_os_str());
        match fs::symlink_metadata(&cursor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return fail(format!("symlink refused: {}", cursor.display()));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return fail(format!("cache path has wrong kind: {}", cursor.display()));
            }
            Ok(_) if index + 1 == parts.len() => private_mode(&cursor, true)?,
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {
                // Every missing component is below an already inspected normal
                // directory, so this never adjusts a broad pre-existing parent.
                match create_private_dir(&cursor) {
                    Ok(()) => {}
                    Err(_) => {
                        // A concurrent creator is acceptable only if it left the
                        // exact private directory we require.
                        private_mode(&cursor, true)?;
                    }
                }
            }
            Err(error) => return fail(format!("cannot inspect {}: {error}", cursor.display())),
        }
    }
    Ok(())
}

fn cache_directory(cache_root: &Path, vault: &Path) -> super::Result<PathBuf> {
    let cache_root = absolute(cache_root)?;
    reject_parent_components(&cache_root)?;
    // This lexical check prevents creating a new descendant in either protected
    // tree. A canonical check follows creation to protect the eventual target.
    if cache_root.starts_with(vault) || cache_root.starts_with(repository_root()?) {
        return fail("cache_root must be outside the selected vault and repository");
    }
    ensure_private_cache_directory(&cache_root)?;
    let external_root = cache_root
        .canonicalize()
        .map_err(|error| format!("cannot canonicalize cache_root: {error}"))?;
    if external_root.starts_with(vault) || external_root.starts_with(repository_root()?) {
        return fail("cache_root must be outside the selected vault and repository");
    }
    let managed = cache_root.join(CACHE_DIR);
    ensure_private_cache_directory(&managed)?;
    Ok(managed)
}

fn canonical_vault(vault: &Path) -> super::Result<PathBuf> {
    inspect_no_symlink_components(vault, false)?;
    let canonical = vault
        .canonicalize()
        .map_err(|error| format!("cannot canonicalize vault {}: {error}", vault.display()))?;
    if !canonical.is_dir() {
        return fail("vault is not a directory");
    }
    Ok(canonical)
}

fn selected_dossier(selection: &Value) -> super::Result<&str> {
    selection
        .get("dossier")
        .and_then(Value::as_str)
        .filter(|dossier| !dossier.is_empty())
        .ok_or_else(|| "selection returned selected without a dossier".to_owned())
}

fn cache_identity(vault: &Path, dossier: &str) -> String {
    let engine = sha256(include_bytes!("lib.rs"));
    sha256(
        format!(
            "cache_format={CACHE_FORMAT}\nvault={}\ndossier={dossier}\nengine={engine}\n",
            vault.display()
        )
        .as_bytes(),
    )
}

fn generated_paths(cache_dir: &Path, identity: &str) -> (PathBuf, PathBuf) {
    let stem = format!("dossier-{identity}");
    (
        cache_dir.join(format!("{stem}.json")),
        cache_dir.join(format!("{stem}.lock")),
    )
}

struct Lock {
    path: PathBuf,
    token: String,
}

impl Lock {
    fn acquire(path: PathBuf) -> super::Result<Option<Self>> {
        if path.exists() {
            private_mode(&path, false)?;
        }
        let token = format!(
            "{}:{}:{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_nanos(),
            sha256(path.as_os_str().as_encoded_bytes()),
        );
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(PRIVATE_FILE_MODE);
        }
        match options.open(&path) {
            Ok(mut file) => {
                file.write_all(token.as_bytes())
                    .map_err(|error| error.to_string())?;
                file.sync_all().map_err(|error| error.to_string())?;
                private_mode(&path, false)?;
                Ok(Some(Self { path, token }))
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                private_mode(&path, false)?;
                Ok(None)
            }
            Err(error) => fail(format!(
                "cannot create cache lock {}: {error}",
                path.display()
            )),
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let mut contents = String::new();
        let removable = File::open(&self.path)
            .and_then(|mut file| file.read_to_string(&mut contents))
            .is_ok()
            && contents == self.token
            && fs::symlink_metadata(&self.path)
                .map(|metadata| !metadata.file_type().is_symlink() && metadata.is_file())
                .unwrap_or(false);
        if removable {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn temporary_index(index: &Path) -> super::Result<PathBuf> {
    let parent = index.parent().ok_or("cache index has no parent")?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    Ok(parent.join(format!(
        "{}-build-{}-{nonce}.tmp",
        index
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("cache index has invalid filename")?,
        std::process::id()
    )))
}

#[cfg(unix)]
fn make_owned_file_private(path: &Path) -> super::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .map_err(|error| format!("cannot inspect built cache {}: {error}", path.display()))?
        .permissions();
    permissions.set_mode(PRIVATE_FILE_MODE);
    fs::set_permissions(path, permissions)
        .map_err(|error| format!("cannot secure built cache {}: {error}", path.display()))?;
    private_mode(path, false)
}

#[cfg(not(unix))]
fn make_owned_file_private(path: &Path) -> super::Result<()> {
    private_mode(path, false)
}

fn remove_owned_temporary(path: &Path) {
    if let Ok(metadata) = fs::symlink_metadata(path)
        && metadata.is_file()
        && !metadata.file_type().is_symlink()
    {
        let _ = fs::remove_file(path);
    }
}

fn publish_rebuild(vault: &Path, dossier: &str, index: &Path) -> super::Result<Value> {
    let temporary = temporary_index(index)?;
    // The engine publishes atomically to our unique sibling.  Only after that
    // succeeds do we replace the managed generated filename, leaving an old
    // cache intact on every build failure.
    let built = match super::build_index(vault, dossier, &temporary) {
        Ok(value) => value,
        Err(error) => {
            remove_owned_temporary(&temporary);
            return Err(error);
        }
    };
    make_owned_file_private(&temporary)?;
    if index.exists() {
        private_mode(index, false)?;
    }
    fs::rename(&temporary, index).map_err(|error| {
        remove_owned_temporary(&temporary);
        format!("cannot publish managed cache {}: {error}", index.display())
    })?;
    private_mode(index, false)?;
    Ok(built)
}

fn cache_rebuild_error(error: &str) -> bool {
    [
        "index is not UTF-8",
        "index has duplicate keys or invalid JSON",
        "invalid index JSON",
        "index must be a JSON object",
        "unsupported or malformed index schema",
        "index vault missing",
        "index vault must be absolute",
        "index dossier missing",
        "index documents missing",
        "index document count invalid",
        "invalid index document",
        "invalid index document identity",
        "invalid indexed section",
        "invalid indexed source artifact",
        "index hash missing",
        "index integrity hash mismatch",
        "index does not belong to this --vault and --dossier",
        "source inventory changed; rebuild the index",
        "source drift detected:",
        "indexed document does not match re-read source:",
    ]
    .iter()
    .any(|allowed| error.starts_with(allowed))
}

fn query_with_cache(
    vault: &Path,
    dossier: &str,
    index: &Path,
    query: &str,
    limit: usize,
) -> super::Result<(Value, &'static str, &'static str)> {
    let initial = match fs::symlink_metadata(index) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return fail("managed cache index is not a regular file");
            }
            private_mode(index, false)?;
            match super::query_index(vault, dossier, index, query, limit) {
                Ok(result) => return Ok((result, "reused", "validated existing managed cache")),
                Err(error) if cache_rebuild_error(&error) => error,
                Err(error) => return Err(error),
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let _ = publish_rebuild(vault, dossier, index)?;
            let result = super::query_index(vault, dossier, index, query, limit)?;
            return Ok((result, "built", "managed cache was absent"));
        }
        Err(error) => {
            return fail(format!(
                "cannot inspect managed cache {}: {error}",
                index.display()
            ));
        }
    };
    let _ = initial;
    let _ = publish_rebuild(vault, dossier, index)?;
    let result = super::query_index(vault, dossier, index, query, limit)?;
    Ok((
        result,
        "rebuilt",
        "cached index failed bounded integrity validation",
    ))
}

/// Select a dossier, maintain one private external cache, and return retrieval evidence.
///
/// This does not formulate an answer.  It returns only the lexical retrieval object from
/// the underlying PoC after a cold build, warm validation, or one bounded rebuild.
pub fn ask_dossier(
    vault: &Path,
    explicit: Option<&str>,
    context: Option<&str>,
    topic: Option<&str>,
    cache_root: &Path,
    query: &str,
    limit: usize,
) -> super::Result<Value> {
    let selection = super::selection::select_dossier(vault, explicit, context, topic, query)?;
    let selection_status = selection
        .get("status")
        .and_then(Value::as_str)
        .ok_or("selection status missing")?;
    if selection_status != "selected" {
        if !matches!(selection_status, "needs_selection" | "not_applicable") {
            return fail("selection returned an unsupported status");
        }
        return Ok(response(
            selection_status,
            query,
            selection.clone(),
            json!({"candidates": selection.get("candidates").cloned().unwrap_or_else(|| json!([]))}),
        ));
    }
    let dossier = selected_dossier(&selection)?.to_owned();
    let vault = canonical_vault(vault)?;
    let cache_dir = cache_directory(cache_root, &vault)?;
    let identity = cache_identity(&vault, &dossier);
    let (index, lock) = generated_paths(&cache_dir, &identity);
    let Some(_lock) = Lock::acquire(lock)? else {
        return Ok(response(
            "busy",
            query,
            selection.clone(),
            json!({
                "vault": vault,
                "dossier": dossier,
                "cache": {"outcome":"busy", "reason":"a managed build or rebuild holds this cache identity; retry"},
            }),
        ));
    };
    let (retrieval, outcome, reason) = query_with_cache(&vault, &dossier, &index, query, limit)?;
    let status = match retrieval.get("status").and_then(Value::as_str) {
        Some("refused") => "refused",
        Some("complete") if retrieval["results"].as_array().is_some_and(Vec::is_empty) => {
            "no_evidence"
        }
        Some("complete") => "complete",
        _ => return fail("retrieval returned an unsupported status"),
    };
    Ok(response(
        status,
        query,
        selection,
        json!({
            "vault": vault,
            "dossier": dossier,
            "cache": {"outcome":outcome,"reason":reason,"format":CACHE_FORMAT,"identity":identity},
            "retrieval": retrieval,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn root() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "managed-dossier-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&root).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&root).unwrap().permissions();
            permissions.set_mode(PRIVATE_DIR_MODE);
            fs::set_permissions(&root, permissions).unwrap();
        }
        root
    }

    fn fixture() -> (PathBuf, PathBuf, String, PathBuf) {
        let root = root();
        let vault = root.join("vault");
        let dossier = "30-topics/fictief-dossier".to_owned();
        fs::create_dir_all(vault.join(&dossier).join("wiki")).unwrap();
        fs::create_dir_all(vault.join(&dossier).join("raw")).unwrap();
        fs::write(
            vault.join(&dossier).join("fictief-dossier.md"),
            "# Fictief dossier\n",
        )
        .unwrap();
        fs::write(
            vault.join(&dossier).join("wiki/evidence.md"),
            "# Fictief\n\n## Garantie\n\nDe fictieve garantie is twee jaar; de ingangsdatum is onzeker.\n",
        )
        .unwrap();
        let cache = root.join("cache");
        (root, vault, dossier, cache)
    }

    #[test]
    fn private_cache_cold_warm_and_source_changes_are_managed() {
        let (root, vault, dossier, cache) = fixture();
        let cold = ask_dossier(&vault, Some(&dossier), None, None, &cache, "garantie", 5).unwrap();
        assert_eq!(cold["status"], "complete");
        assert_eq!(cold["cache"]["outcome"], "built");
        let warm = ask_dossier(&vault, Some(&dossier), None, None, &cache, "garantie", 5).unwrap();
        assert_eq!(warm["cache"]["outcome"], "reused");
        fs::write(
            vault.join(&dossier).join("wiki/evidence.md"),
            "# Fictief\n\n## Garantie\n\nDe fictieve garantie is drie jaar.\n",
        )
        .unwrap();
        let changed =
            ask_dossier(&vault, Some(&dossier), None, None, &cache, "garantie", 5).unwrap();
        assert_eq!(changed["cache"]["outcome"], "rebuilt");
        fs::write(
            vault.join(&dossier).join("wiki/new.md"),
            "# Extra\n\n## Garantie\n\nFictief aanvullend bewijs.\n",
        )
        .unwrap();
        assert_eq!(
            ask_dossier(&vault, Some(&dossier), None, None, &cache, "garantie", 5).unwrap()["cache"]
                ["outcome"],
            "rebuilt"
        );
        fs::remove_file(vault.join(&dossier).join("wiki/new.md")).unwrap();
        assert_eq!(
            ask_dossier(&vault, Some(&dossier), None, None, &cache, "garantie", 5).unwrap()["cache"]
                ["outcome"],
            "rebuilt"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corrupt_cache_rebuilds_but_locks_and_insecure_cache_do_not_fallback() {
        let (root, vault, dossier, cache) = fixture();
        let first = ask_dossier(&vault, Some(&dossier), None, None, &cache, "garantie", 5).unwrap();
        let identity = first["cache"]["identity"].as_str().unwrap();
        let cache_dir = cache.join(CACHE_DIR);
        let index = cache_dir.join(format!("dossier-{identity}.json"));
        fs::write(&index, b"not json").unwrap();
        #[cfg(unix)]
        fs::set_permissions(&index, fs::Permissions::from_mode(PRIVATE_FILE_MODE)).unwrap();
        assert_eq!(
            ask_dossier(&vault, Some(&dossier), None, None, &cache, "garantie", 5).unwrap()["cache"]
                ["outcome"],
            "rebuilt"
        );
        let lock = cache_dir.join(format!("dossier-{identity}.lock"));
        let held = Lock::acquire(lock).unwrap().unwrap();
        assert_eq!(
            ask_dossier(&vault, Some(&dossier), None, None, &cache, "garantie", 5).unwrap()["status"],
            "busy"
        );
        drop(held);
        let inside_vault = vault.join("private-cache");
        assert!(
            ask_dossier(
                &vault,
                Some(&dossier),
                None,
                None,
                &inside_vault,
                "garantie",
                5,
            )
            .is_err()
        );
        assert!(!inside_vault.exists());
        #[cfg(unix)]
        {
            fs::set_permissions(&cache_dir, fs::Permissions::from_mode(0o755)).unwrap();
            assert!(
                ask_dossier(&vault, Some(&dossier), None, None, &cache, "garantie", 5).is_err()
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn creates_missing_xdg_style_app_parents_privately_without_chmodding_xdg_root() {
        let (root, vault, dossier, _cache) = fixture();
        let xdg_root = root.join("xdg-cache");
        fs::create_dir(&xdg_root).unwrap();
        fs::set_permissions(&xdg_root, fs::Permissions::from_mode(0o755)).unwrap();
        let cache = xdg_root.join("viggobsidian/dossier-retrieval");

        let result =
            ask_dossier(&vault, Some(&dossier), None, None, &cache, "garantie", 5).unwrap();
        assert_eq!(result["cache"]["outcome"], "built");
        assert_eq!(fs::metadata(&xdg_root).unwrap().mode() & 0o777, 0o755);
        for directory in [
            xdg_root.join("viggobsidian"),
            cache.clone(),
            cache.join(CACHE_DIR),
        ] {
            assert_eq!(
                fs::metadata(&directory).unwrap().mode() & 0o777,
                PRIVATE_DIR_MODE,
                "directory: {}",
                directory.display()
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn selection_nonresults_never_create_cache() {
        let (root, vault, _dossier, cache) = fixture();
        let result = ask_dossier(&vault, None, None, None, &cache, "garantie", 5).unwrap();
        assert!(matches!(
            result["status"].as_str(),
            Some("needs_selection" | "not_applicable")
        ));
        assert!(!cache.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
