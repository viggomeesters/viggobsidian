use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
mod css;
const STATE: &str = ".viggobsidian/installed.json";
const SKILLS: &[&str] = &[
    "vault",
    "vault-log",
    "vault-process",
    "vault-task",
    "vault-tackle",
    "vault-hub",
    "vault-brief",
];

fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(message.into().into())
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn field<'a>(v: &'a Value, name: &str) -> Result<&'a str> {
    v[name]
        .as_str()
        .ok_or_else(|| format!("missing string: {name}").into())
}
fn rows<'a>(v: &'a Value, name: &str) -> Result<&'a Vec<Value>> {
    v[name]
        .as_array()
        .ok_or_else(|| format!("missing array: {name}").into())
}
fn relative(path: &str) -> Result<&Path> {
    let p = Path::new(path);
    if path.is_empty()
        || p.components().any(|c| !matches!(c, Component::Normal(_)))
        || path.contains('\\')
    {
        return fail(format!("unsafe relative path: {path}"));
    }
    Ok(p)
}
fn checked(root: &Path, path: &str) -> Result<PathBuf> {
    let mut full = root.to_path_buf();
    for part in relative(path)?.components() {
        full.push(part.as_os_str());
        match fs::symlink_metadata(&full) {
            Ok(m) if m.file_type().is_symlink() => {
                return fail(format!("symlink refused: {}", full.display()));
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(full)
}
fn root(path: &Path) -> Result<PathBuf> {
    // Reject symlinks in the explicitly supplied path before canonicalization.
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut cursor = PathBuf::new();
    for part in absolute.components() {
        if matches!(part, Component::ParentDir) {
            return fail("parent component refused");
        }
        cursor.push(part.as_os_str());
        if fs::symlink_metadata(&cursor)?.file_type().is_symlink() {
            return fail("root contains a symlink");
        }
    }
    let canonical = absolute.canonicalize()?;
    if !canonical.is_dir() {
        return fail("root is not a directory");
    }
    Ok(canonical)
}
fn allowed(target: &str) -> bool {
    if relative(target).is_err() {
        return false;
    }
    if SKILLS
        .iter()
        .any(|s| target == format!(".agents/skills/{s}/SKILL.md"))
    {
        return true;
    }
    if [
        "viggobsidian.css",
        "folder-note.css",
        "agents-callout.css",
        "active-note-contrast.css",
    ]
    .iter()
    .any(|name| target == format!(".obsidian/snippets/{name}"))
    {
        return true;
    }
    if target == ".viggobsidian/core/folder-note.css" {
        return true;
    }
    if [
        "layout-contract.yaml",
        "vault-raw.base",
        "vault-dossier.base",
    ]
    .iter()
    .any(|name| target == format!(".agents/tools/vault-layout/fixtures/{name}"))
        || target == ".agents/tools/vault-layout/tests/contract_enforcement.rs"
    {
        return true;
    }
    if ["Cargo.toml", "Cargo.lock"]
        .iter()
        .any(|name| target == format!(".agents/tools/vault-layout/{name}"))
    {
        return true;
    }
    if [
        "main",
        "activity_layout",
        "dossier_tables",
        "event_header",
        "index_navigation",
        "link_style",
        "mail_manifest",
        "mail_draft",
        "project_codes",
        "raw_processing",
        "raw_task",
        "shortcut_links",
        "source_inventory",
        "wiki_layout",
        "work_intake",
        "work_package",
    ]
    .iter()
    .any(|name| target == format!(".agents/tools/vault-layout/src/{name}.rs"))
    {
        return true;
    }
    if [
        "policy.md",
        "mail-format.md",
        "mail-draft.md",
        "skills.md",
        "updates.md",
        "numbered-v1.yaml",
        "vault-schema.yaml",
        "vault.schema.json",
        "daily.md",
        "lucide-license.txt",
    ]
    .iter()
    .any(|s| target == format!(".viggobsidian/core/{s}"))
    {
        return true;
    }
    ["dossier", "inbox", "raw", "templates"].iter().any(|s| {
        target == format!("99-system/vault/vault-navigation/standaard/wiki/vault-{s}.base")
    })
}
fn bytes_at(root: &Path, path: &str) -> Result<Option<Vec<u8>>> {
    let path = checked(root, path)?;
    match fs::read(path) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
fn digest_at(root: &Path, path: &str) -> Result<Value> {
    Ok(bytes_at(root, path)?
        .map(|b| Value::String(hash(&b)))
        .unwrap_or(Value::Null))
}
fn atomic(root: &Path, target: &str, bytes: &[u8]) -> Result<()> {
    let dest = checked(root, target)?;
    let parent = dest.parent().ok_or("missing parent")?;
    fs::create_dir_all(parent)?;
    checked(root, target)?;
    let temporary = parent.join(format!(".viggobsidian-{}.tmp", stamp()?));
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    if let Err(e) = fs::rename(&temporary, dest) {
        let _ = fs::remove_file(&temporary);
        return Err(e.into());
    }
    Ok(())
}
fn stamp() -> Result<String> {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    Ok(format!(
        "{}-{}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ))
}
struct Lock(PathBuf);
impl Lock {
    fn acquire(vault: &Path) -> Result<Self> {
        let dir = checked(vault, ".viggobsidian")?;
        fs::create_dir_all(dir)?;
        let path = checked(vault, ".viggobsidian/update.lock")?;
        fs::create_dir(&path)
            .map_err(|e| format!("update lock unavailable (do not blindly remove it): {e}"))?;
        Ok(Self(path))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}
fn package(package: &Path) -> Result<(Value, String)> {
    let bytes = fs::read(checked(package, "package.json")?)?;
    let manifest: Value = serde_json::from_slice(&bytes)?;
    if manifest["schema"] != "viggobsidian.package.v1" {
        return fail("unknown package schema");
    }
    let revision = field(&manifest, "revision")?;
    if revision.len() != 40 || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return fail("package requires an exact Git commit");
    }
    if !field(&manifest, "version")?.starts_with("0.") {
        return fail("unsupported package major version");
    }
    let mut targets = BTreeSet::new();
    for item in rows(&manifest, "files")? {
        let target = field(item, "target")?;
        if !item["transform"].is_null()
            && (item["transform"] != "css-v1" || target != ".obsidian/snippets/folder-note.css")
        {
            return fail("unknown or misplaced package transform");
        }
        if !allowed(target) || !targets.insert(target) {
            return fail(format!("forbidden or duplicate target: {target}"));
        }
        let source = checked(package, field(item, "source")?)?;
        let bytes = fs::read(source)?;
        if bytes.len() > 16 * 1024 * 1024 || hash(&bytes) != field(item, "sha256")? {
            return fail(format!("package hash mismatch: {target}"));
        }
    }
    if targets.is_empty() {
        return fail("empty package");
    }
    Ok((manifest, hash(&bytes)))
}
fn materialize(
    vault: &Path,
    package_root: &Path,
    item: &Value,
) -> Result<(Vec<u8>, Option<Value>)> {
    let source = fs::read(checked(package_root, field(item, "source")?)?)?;
    if hash(&source) != field(item, "sha256")? {
        return fail("package source changed");
    }
    if item["transform"].is_null() {
        return Ok((source, None));
    }
    if item["transform"] != "css-v1" || item["target"] != ".obsidian/snippets/folder-note.css" {
        return fail("invalid transform");
    }
    let config_name = ".viggobsidian/appearance.json";
    let config = fs::read(checked(vault, config_name)?)?;
    let bytes = css::render(
        std::str::from_utf8(&source)?,
        &serde_json::from_slice(&config)?,
    )?;
    let recipe = json!({"kind":"css-v1","source_sha256":item["sha256"],"config":config_name,"config_sha256":hash(&config),"sha256":hash(&bytes)});
    Ok((bytes, Some(recipe)))
}
fn state(vault: &Path) -> Result<Value> {
    match bytes_at(vault, STATE)? {
        None => Ok(Value::Null),
        Some(bytes) => {
            let value: Value = serde_json::from_slice(&bytes)?;
            if value["schema"] != "viggobsidian.installed.v1" || !value["files"].is_object() {
                return fail("invalid installed state");
            }
            let manifest_bytes = field(&value, "manifest_bytes")?;
            let manifest: Value = serde_json::from_str(manifest_bytes)?;
            let mut files = BTreeMap::new();
            for item in rows(&manifest, "files")? {
                let target = field(item, "target")?;
                let expected = if item["transform"] == "css-v1"
                    && target == ".obsidian/snippets/folder-note.css"
                {
                    let derivation = &value["derived"][target];
                    if derivation["kind"] != "css-v1"
                        || derivation["source_sha256"] != item["sha256"]
                        || derivation["config"] != ".viggobsidian/appearance.json"
                    {
                        return fail("invalid installed CSS derivation");
                    }
                    field(derivation, "config_sha256")?;
                    field(derivation, "sha256")?;
                    derivation["sha256"].clone()
                } else if item["transform"].is_null() {
                    item["sha256"].clone()
                } else {
                    return fail("unsupported installed transform");
                };
                if !allowed(target) || files.insert(target.to_string(), expected).is_some() {
                    return fail("invalid installed manifest target");
                }
            }
            if hash(manifest_bytes.as_bytes()) != field(&value, "manifest_sha256")?
                || manifest["schema"] != "viggobsidian.package.v1"
                || manifest["revision"] != value["revision"]
                || manifest["version"] != value["version"]
                || serde_json::to_value(files)? != value["files"]
            {
                return fail("installed manifest integrity failure");
            }
            for (path, digest) in value["files"]
                .as_object()
                .ok_or("missing installed files")?
            {
                if !allowed(path) || digest.as_str().is_none() {
                    return fail("unsafe installed entry");
                }
            }
            Ok(value)
        }
    }
}
fn plan(vault: &Path, package_root: &Path, adoption: &Value) -> Result<Value> {
    if !checked(vault, "AGENTS.md")?.is_file() {
        return fail("target vault must have AGENTS.md");
    }
    if !adoption.is_object() {
        return fail("adoption must map exact target paths to preimage hashes");
    }
    let (manifest, manifest_sha256) = package(package_root)?;
    let installed = state(vault)?;
    if let Some(old) = installed["files"].as_object() {
        for (path, expected) in old {
            if digest_at(vault, path)? != *expected {
                return fail(format!("managed drift: {path}"));
            }
        }
    }
    let mut changes = Vec::new();
    let mut next = BTreeMap::new();
    let mut derived = BTreeMap::new();
    for item in rows(&manifest, "files")? {
        let target = field(item, "target")?;
        let preimage = digest_at(vault, target)?;
        let (bytes, recipe) = materialize(vault, package_root, item)?;
        let desired = &Value::String(hash(&bytes));
        if let Some(recipe) = recipe {
            derived.insert(target, recipe);
        }
        if !preimage.is_null()
            && installed["files"][target].is_null()
            && adoption[target] != preimage
        {
            return fail(format!(
                "unmanaged existing file requires reviewed adoption hash: {target}"
            ));
        }
        if &preimage != desired {
            changes.push(json!({"target":target,"before":preimage,"after":desired}));
        }
        next.insert(target.to_string(), desired.clone());
    }
    // A release cannot silently retire managed files. A later explicit migration can do so.
    if let Some(old) = installed["files"].as_object() {
        for target in old.keys() {
            if !next.contains_key(target) {
                return fail(format!(
                    "release retires a managed file; explicit migration required: {target}"
                ));
            }
        }
    }
    Ok(
        json!({"schema":"viggobsidian.plan.v1","vault":vault,"package":package_root,
        "version":manifest["version"],"revision":manifest["revision"],"manifest_sha256":manifest_sha256,
        "previous_state_sha256":digest_at(vault,STATE)?,"adoption":adoption,"changes":changes,
        "next_state":{"schema":"viggobsidian.installed.v1","version":manifest["version"],"revision":manifest["revision"],"manifest_sha256":manifest_sha256,"manifest_bytes":fs::read_to_string(checked(package_root,"package.json")?)?,"files":next,"derived":derived}}),
    )
}
fn apply(vault: &Path, proposal: &Value) -> Result<Value> {
    let _lock = Lock::acquire(vault)?;
    if proposal["schema"] != "viggobsidian.plan.v1"
        || proposal["vault"] != vault.to_string_lossy().as_ref()
    {
        return fail("plan target mismatch");
    }
    if checked(vault, ".viggobsidian/pending.json")?.exists()
        || checked(vault, ".viggobsidian/rollback.json")?.exists()
    {
        return fail(
            "unfinished transaction: rollback with .viggobsidian/pending.json or .viggobsidian/rollback.json before applying",
        );
    }
    let package_root = root(Path::new(field(proposal, "package")?))?;
    let fresh = plan(vault, &package_root, &proposal["adoption"])?;
    if fresh != *proposal {
        return fail("stale or altered plan; regenerate and review");
    }
    if rows(proposal, "changes")?.is_empty() && state(vault)? == proposal["next_state"] {
        return Ok(json!({"status":"unchanged"}));
    }
    let (manifest, _) = package(&package_root)?;
    let mut entries = Vec::new();
    for change in rows(proposal, "changes")? {
        let target = field(change, "target")?;
        let bytes = bytes_at(vault, target)?;
        entries.push(json!({"target":target,"before":bytes,"after":change["after"]}));
    }
    let old_state = bytes_at(vault, STATE)?;
    let receipt_path = format!(".viggobsidian/receipts/{}.json", stamp()?);
    let receipt = json!({"schema":"viggobsidian.receipt.v1","vault":vault,"receipt":receipt_path,
        "entries":entries,"previous_state":old_state,"next_state":proposal["next_state"]});
    atomic(vault, &receipt_path, &serde_json::to_vec_pretty(&receipt)?)?;
    atomic(
        vault,
        ".viggobsidian/pending.json",
        &serde_json::to_vec_pretty(&receipt)?,
    )?;
    for change in rows(proposal, "changes")? {
        let target = field(change, "target")?;
        if digest_at(vault, target)? != change["before"] {
            return fail(format!("concurrent edit; transaction retained: {target}"));
        }
        let item = rows(&manifest, "files")?
            .iter()
            .find(|i| i["target"] == target)
            .ok_or("missing package target")?;
        let (bytes, _) = materialize(vault, &package_root, item)?;
        if hash(&bytes) != field(change, "after")? {
            return fail("package or local CSS configuration changed during apply");
        }
        atomic(vault, target, &bytes)?;
    }
    atomic(
        vault,
        STATE,
        &serde_json::to_vec_pretty(&proposal["next_state"])?,
    )?;
    fs::remove_file(checked(vault, ".viggobsidian/pending.json")?)?;
    Ok(
        json!({"status":"applied","receipt":receipt_path,"version":proposal["version"],"changed":rows(proposal,"changes")?.len()}),
    )
}
fn rollback(vault: &Path, receipt: &Value) -> Result<Value> {
    let _lock = Lock::acquire(vault)?;
    if receipt["schema"] != "viggobsidian.receipt.v1"
        || receipt["vault"] != vault.to_string_lossy().as_ref()
    {
        return fail("receipt target mismatch");
    }
    let next = &receipt["next_state"];
    let installed = state(vault)?;
    let pending = bytes_at(vault, ".viggobsidian/pending.json")?;
    let undo = bytes_at(vault, ".viggobsidian/rollback.json")?;
    if let Some(bytes) = &undo {
        let active: Value = serde_json::from_slice(bytes)?;
        if active != *receipt {
            return fail("a different rollback is in progress");
        }
    }
    if let Some(bytes) = &pending {
        let active: Value = serde_json::from_slice(bytes)?;
        if active != *receipt {
            return fail("a different pending transaction exists");
        }
    } else if undo.is_none() && installed != *next {
        return fail("receipt is not current installation");
    }
    let mut restoring = BTreeSet::new();
    for item in rows(receipt, "entries")? {
        let target = field(item, "target")?;
        if !allowed(target) || !restoring.insert(target) || next["files"][target] != item["after"] {
            return fail("unsafe receipt target");
        }
        let current = digest_at(vault, target)?;
        let before: Option<Vec<u8>> = serde_json::from_value(item["before"].clone())?;
        let before_hash = before
            .as_ref()
            .map(|b| Value::String(hash(b)))
            .unwrap_or(Value::Null);
        if current != item["after"]
            && !((pending.is_some() || undo.is_some()) && current == before_hash)
        {
            return fail(format!("rollback conflict: {target}"));
        }
    }
    // Verify the whole managed postimage, including assets that this transaction didn't change.
    if pending.is_none() {
        for (target, expected) in next["files"].as_object().ok_or("missing next files")? {
            if !allowed(target)
                || (!restoring.contains(target.as_str()) && digest_at(vault, target)? != *expected)
            {
                return fail(format!("rollback managed drift: {target}"));
            }
        }
    }
    atomic(
        vault,
        ".viggobsidian/rollback.json",
        &serde_json::to_vec_pretty(receipt)?,
    )?;
    for item in rows(receipt, "entries")? {
        let target = field(item, "target")?;
        let before: Option<Vec<u8>> = serde_json::from_value(item["before"].clone())?;
        if let Some(bytes) = before {
            atomic(vault, target, &bytes)?;
        } else if checked(vault, target)?.exists() {
            fs::remove_file(checked(vault, target)?)?;
        }
    }
    let before: Option<Vec<u8>> = serde_json::from_value(receipt["previous_state"].clone())?;
    if let Some(bytes) = before {
        atomic(vault, STATE, &bytes)?;
    } else if checked(vault, STATE)?.exists() {
        fs::remove_file(checked(vault, STATE)?)?;
    }
    if pending.is_some() {
        fs::remove_file(checked(vault, ".viggobsidian/pending.json")?)?;
    }
    fs::remove_file(checked(vault, ".viggobsidian/rollback.json")?)?;
    Ok(json!({"status":"rolled-back","receipt":receipt["receipt"]}))
}
fn program(name: &str) -> bool {
    let b = name.as_bytes();
    b.len() > 3 && b[0] == b'2' && (b'1'..=b'9').contains(&b[1]) && b[2] == b'-' && slug(&name[3..])
}
fn slug(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn project_name(name: &str) -> bool {
    let b = name.as_bytes();
    name.is_ascii()
        && b.len() > 8
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'-'
        && matches!(
            &name[5..7],
            "01" | "02" | "03" | "04" | "05" | "06" | "07" | "08" | "09" | "10" | "11" | "12"
        )
        && b[7] == b'-'
        && slug(&name[8..])
}
fn dirs(path: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_symlink() {
            return fail(format!(
                "symlink inside checked scope: {}",
                entry.path().display()
            ));
        }
        if entry.file_type()?.is_dir() {
            out.push(entry.path());
        }
    }
    out.sort();
    Ok(out)
}
fn check(vault: &Path, structure: bool) -> Result<Value> {
    let installed = state(vault)?;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    if installed.is_null() {
        errors.push("package not installed".to_string());
    }
    if let Some(files) = installed["files"].as_object() {
        for (path, expected) in files {
            if digest_at(vault, path)? != *expected {
                errors.push(format!("managed drift: {path}"));
            }
        }
    }
    if let Some(derived) = installed["derived"].as_object() {
        for (target, recipe) in derived {
            let config = fs::read(checked(vault, field(recipe, "config")?)?)?;
            let shared = fs::read(checked(vault, ".viggobsidian/core/folder-note.css")?)?;
            if hash(&config) != field(recipe, "config_sha256")?
                || hash(&shared) != field(recipe, "source_sha256")?
                || hash(&css::render(
                    std::str::from_utf8(&shared)?,
                    &serde_json::from_slice(&config)?,
                )?) != field(recipe, "sha256")?
            {
                errors.push(format!(
                    "CSS inputs changed; preview/apply required: {target}"
                ));
            }
        }
    }
    let integration = read_json(&checked(vault, ".viggobsidian/integration.json")?)?;
    let valid_keys = ["schema", "profile", "adoption", "local_adapter", "layout"];
    if !integration.as_object().is_some_and(|o| {
        (o.len() == 4 || o.len() == 5) && o.keys().all(|k| valid_keys.contains(&k.as_str()))
    }) || !integration["adoption"]
        .as_object()
        .is_some_and(|o| o.keys().all(|k| k == "legacy_project_roots"))
        || (!integration["adoption"]["legacy_project_roots"].is_null()
            && !integration["adoption"]["legacy_project_roots"].is_array())
    {
        errors.push("integration does not match the published schema".into());
    }
    if integration["schema"] != "viggobsidian.integration.v1"
        || integration["profile"] != "numbered-v1"
        || integration["local_adapter"] != ".viggobsidian/local-adapter.md"
    {
        errors.push("invalid integration contract".into());
    }
    if integration.get("layout").is_some()
        && integration["layout"]
            != json!({"engine":".agents/tools/vault-layout/Cargo.toml","css_configuration":".viggobsidian/appearance.json","source":"viggobsidian-release"})
    {
        errors.push("invalid layout integration contract".into());
    }
    if !checked(vault, ".viggobsidian/local-adapter.md")?.is_file() {
        errors.push("missing local adapter".into());
    }
    let mut legacy = BTreeSet::new();
    if let Some(items) = integration["adoption"]["legacy_project_roots"].as_array() {
        for item in items {
            let name = item.as_str().ok_or("invalid legacy root")?;
            relative(name)?;
            if name.contains('/')
                || name.len() < 4
                || !name.as_bytes()[..2].iter().all(u8::is_ascii_digit)
                || name.as_bytes()[2] != b'-'
                || !slug(&name[3..])
            {
                return fail("invalid legacy root");
            }
            legacy.insert(name.to_string());
            warnings.push(format!(
                "explicit adoption debt: {name}; no content was moved"
            ));
        }
    }
    let mut projects = 0;
    let project_roots = if structure { dirs(vault)? } else { Vec::new() };
    for path in project_roots {
        let name = path
            .file_name()
            .ok_or("missing name")?
            .to_string_lossy()
            .to_string();
        if name == "20-projects" || program(&name) || legacy.contains(&name) {
            for project in dirs(&path)? {
                let pname = project
                    .file_name()
                    .ok_or("missing project name")?
                    .to_string_lossy()
                    .to_string();
                if !project_name(&pname) {
                    errors.push(format!("invalid project name: {name}/{pname}"));
                    continue;
                }
                projects += 1;
                let mut code: Option<String> = None;
                let mut numbers = BTreeSet::new();
                let mut zero = false;
                for component in dirs(&project)? {
                    let cname = component
                        .file_name()
                        .ok_or("missing component name")?
                        .to_string_lossy()
                        .to_string();
                    let bits: Vec<_> = cname.splitn(3, '-').collect();
                    if bits.len() != 3
                        || bits[0].len() < 2
                        || bits[0].len() > 12
                        || !bits[0].as_bytes()[0].is_ascii_lowercase()
                        || !bits[0]
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                        || bits[1].len() != 2
                        || !bits[1].bytes().all(|b| b.is_ascii_digit())
                        || !slug(bits[2])
                    {
                        errors.push(format!("invalid component: {name}/{pname}/{cname}"));
                        continue;
                    }
                    if code.as_ref().is_some_and(|c| c != bits[0]) {
                        errors.push(format!("mixed project codes: {name}/{pname}"));
                    }
                    code = Some(bits[0].into());
                    if !numbers.insert(bits[1].to_string()) {
                        errors.push(format!(
                            "duplicate component number: {name}/{pname}/{cname}"
                        ));
                    }
                    if bits[1] == "00" {
                        zero = bits[2] == "project-wiki";
                    }
                    for layer in ["wiki", "raw"] {
                        if !component.join(layer).is_dir() {
                            errors.push(format!("missing {layer}: {name}/{pname}/{cname}"));
                        }
                    }
                }
                if !zero {
                    errors.push(format!("missing code-00-project-wiki: {name}/{pname}"));
                }
            }
        }
    }
    Ok(
        json!({"schema":"viggobsidian.check.v1","version":installed["version"],"profile":"numbered-v1",
        "scope":if structure {"package-integrity-and-project-structure; local strict adapters remain required"}else{"package-integrity-and-integration; provider capability checks remain required"},"projects":projects,"warnings":warnings,"errors":errors}),
    )
}
fn pack(package_root: &Path, revision: &str) -> Result<Value> {
    let distribution = read_json(&checked(package_root, "distribution.json")?)?;
    let mut files = Vec::new();
    for item in rows(&distribution, "files")? {
        let target = field(item, "target")?;
        if !allowed(target) {
            return fail("forbidden package target");
        }
        let source = field(item, "source")?;
        let bytes = fs::read(checked(package_root, source)?)?;
        let mut entry = json!({"source":source,"target":target,"sha256":hash(&bytes)});
        if !item["transform"].is_null() {
            entry["transform"] = item["transform"].clone();
        }
        files.push(entry);
    }
    let manifest = json!({"schema":"viggobsidian.package.v1","version":env!("CARGO_PKG_VERSION"),"revision":revision,"files":files});
    atomic(
        package_root,
        "package.json",
        &serde_json::to_vec_pretty(&manifest)?,
    )?;
    package(package_root)?;
    Ok(manifest)
}
fn run() -> Result<bool> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let get = |name: &str| -> Result<&str> {
        let i = args
            .iter()
            .position(|a| a == name)
            .ok_or_else(|| format!("missing {name}"))?;
        args.get(i + 1)
            .map(String::as_str)
            .ok_or_else(|| format!("missing value: {name}").into())
    };
    let command = args.first().map(String::as_str).unwrap_or("help");
    let mut ok = true;
    let output = match command {
        "css-split" => {
            let (shared, routes) = css::split(&fs::read_to_string(get("--input")?)?)?;
            json!({"shared":shared,"appearance":{"schema":"viggobsidian.appearance.v1","routes":routes}})
        }
        "pack" => pack(&root(Path::new(get("--package")?))?, get("--revision")?)?,
        "plan" => {
            let adoption = if args.iter().any(|a| a == "--adopt") {
                read_json(Path::new(get("--adopt")?))?
            } else {
                json!({})
            };
            plan(
                &root(Path::new(get("--vault")?))?,
                &root(Path::new(get("--package")?))?,
                &adoption,
            )?
        }
        "apply" => apply(
            &root(Path::new(get("--vault")?))?,
            &read_json(Path::new(get("--plan")?))?,
        )?,
        "rollback" => rollback(
            &root(Path::new(get("--vault")?))?,
            &read_json(Path::new(get("--receipt")?))?,
        )?,
        "check" | "doctor" => {
            let out = check(&root(Path::new(get("--vault")?))?, command == "check")?;
            ok = rows(&out, "errors")?.is_empty();
            out
        }
        "help" | "--help" => {
            println!(
                "viggobsidian 0.2\npack --package DIR --revision COMMIT\nplan --vault DIR --package DIR [--adopt JSON]\napply --vault DIR --plan JSON\nrollback --vault DIR --receipt JSON\ncheck --vault DIR\nAll commands print JSON; redirect a plan to a file before apply. See docs/updates.md."
            );
            return Ok(true);
        }
        _ => return fail("unknown command"),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(ok)
}
fn main() {
    match run() {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests;
