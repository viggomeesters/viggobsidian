//! A bounded handoff: Obsidian moves content, this engine rebinds adoption.
use crate::adoption::{self, hash};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const META: &str = ".viggobsidian";
const JOURNAL: &str = "project-relocation.pending.json";
const INTEGRATION: &str = ".viggobsidian/integration.json";
const DEST: &str = "20-projects";
const PLAN: &str = "viggobsidian.project-relocation-plan.v1";

fn read_regular(path: &Path) -> Result<Option<File>> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() => Ok(Some(File::open(path)?)),
        Ok(_) => Err(format!("expected regular file: {}", path.display()).into()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

// Missing parents are valid for historical proofs. Existing symlinks are not.
fn guarded(vault: &Path, relative: &str) -> Result<PathBuf> {
    if relative.contains('\\')
        || relative
            .split('/')
            .any(|p| p.is_empty() || matches!(p, "." | ".."))
    {
        return Err(format!("unsafe relocation member: {relative}").into());
    }
    let mut candidate = vault.to_path_buf();
    for part in relative.split('/') {
        candidate.push(part);
        match fs::symlink_metadata(&candidate) {
            Ok(m) if m.is_symlink() => {
                return Err(format!("symlink relocation member: {relative}").into());
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(candidate)
}

fn read_json(path: &Path) -> Result<(Value, String)> {
    let mut file = read_regular(path)?.ok_or_else(|| format!("missing {}", path.display()))?;
    let mut bytes = Vec::new();
    io::Read::read_to_end(&mut file, &mut bytes)?;
    let opts = serde_saphyr::options! { duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error };
    let value: Value = serde_saphyr::from_str_with_options(std::str::from_utf8(&bytes)?, opts)?;
    Ok((value, hash(&bytes)))
}

fn fingerprint(path: &Path) -> Result<Value> {
    let Some(mut file) = read_regular(path)? else {
        return Ok(Value::Null);
    };
    let mut digest = Sha256::new();
    let bytes = io::copy(&mut file, &mut digest)?;
    Ok(json!({"bytes":bytes,"sha256":format!("{:x}", digest.finalize())}))
}

fn present(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

fn metadata(vault: &Path) -> Result<PathBuf> {
    let dir = guarded(vault, META)?;
    if !dir.is_dir() {
        return Err("relocation requires an existing local .viggobsidian directory".into());
    }
    Ok(dir)
}

fn acquire(vault: &Path, exclusive: bool) -> Result<File> {
    if !cfg!(unix) {
        return Err("project relocation requires Unix directory locks".into());
    }
    let file = File::open(metadata(vault)?)?;
    if exclusive {
        file.try_lock()?;
    } else {
        file.try_lock_shared()?;
    }
    Ok(file)
}

// Holds a shared guard without creating files in ordinary write-set preflight.
pub(crate) fn ordinary(vault: &Path) -> Result<Option<File>> {
    if !cfg!(unix) || !present(&vault.join(META))? {
        return Ok(None);
    }
    let gate = acquire(vault, false)?;
    if present(&vault.join(META).join(JOURNAL))? {
        return Err("project relocation pending; use the original plan to recover".into());
    }
    Ok(Some(gate))
}

fn source(input: &Value) -> Result<&str> {
    let root = input["source"]
        .as_str()
        .ok_or("missing relocation source")?;
    // The existing project-code resolver recognizes this legacy root. Do not
    // treat arbitrary configured collections or program roots as equivalent.
    if root != "15-projects" {
        return Err("project relocation only supports 15-projects → 20-projects".into());
    }
    Ok(root)
}

fn tree(root: &Path, dir: &Path, members: &mut BTreeMap<String, Value>) -> Result<()> {
    let mut names = std::collections::BTreeSet::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let relative = path
            .strip_prefix(root)?
            .to_str()
            .ok_or("non-UTF8 tree member")?;
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .ok_or("non-UTF8 name")?;
        if name.contains('\\') || !names.insert(name.to_lowercase()) {
            return Err(format!("unsafe or case-colliding project member: {relative}").into());
        }
        let meta = fs::symlink_metadata(&path)?;
        if meta.is_dir() {
            members.insert(relative.to_owned(), json!("directory"));
            tree(root, &path, members)?;
        } else if meta.is_file() {
            members.insert(relative.to_owned(), fingerprint(&path)?);
        } else {
            return Err(format!("non-regular project member: {relative}").into());
        }
    }
    Ok(())
}

fn inventory(vault: &Path, name: &str) -> Result<BTreeMap<String, Value>> {
    let mut matches = 0;
    for row in fs::read_dir(vault)? {
        let leaf = row?.file_name();
        if leaf.to_string_lossy().eq_ignore_ascii_case(name) {
            if leaf != name {
                return Err(
                    format!("case-equivalent project root: {}", leaf.to_string_lossy()).into(),
                );
            }
            matches += 1;
        }
    }
    let root = guarded(vault, name)?;
    if matches != 1 || !root.is_dir() {
        return Err(format!("project root is not a directory: {name}").into());
    }
    let mut members = BTreeMap::new();
    tree(&root, &root, &mut members)?;
    Ok(members)
}

fn appears(vault: &Path, name: &str) -> Result<bool> {
    for row in fs::read_dir(vault)? {
        if row?
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(name)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn mapped(raw: &str, old: &str) -> Result<String> {
    if raw == format!("{old}/{old}.md") {
        return Ok(format!("{DEST}/{DEST}.md"));
    }
    if raw
        .split('/')
        .next()
        .is_some_and(|part| part.eq_ignore_ascii_case(DEST))
    {
        return Err(format!("existing adoption binding at destination: {raw}").into());
    }
    if raw
        .split('/')
        .next()
        .is_some_and(|part| part != old && part.eq_ignore_ascii_case(old))
    {
        return Err(format!("case-equivalent adoption binding at source: {raw}").into());
    }
    if let Ok(rest) = Path::new(raw).strip_prefix(old) {
        let Some(rest) = rest.to_str() else {
            return Err("non-UTF8 adoption suffix".into());
        };
        return Ok(if rest.is_empty() {
            DEST.to_owned()
        } else {
            format!("{DEST}/{rest}")
        });
    }
    Ok(raw.to_owned())
}

fn move_adoption(vault: &Path, current: &Value, old: &str) -> Result<(Value, Value)> {
    let mut owners = Map::new();
    let mut protected = Map::new();
    let mut moved = false;
    for (owner, entry) in current["owners"]
        .as_object()
        .ok_or("missing adoption owners")?
    {
        let mut next = entry.clone();
        let mut files = Map::new();
        for (path, proof) in entry["files"].as_object().ok_or("missing file proofs")? {
            let target = mapped(path, old)?;
            if target == *path && entry["state"] == "legacy" {
                protected.insert(path.clone(), fingerprint(&guarded(vault, path)?)?);
            }
            if files.insert(target, proof.clone()).is_some() {
                return Err(format!("adoption file collision: {path}").into());
            }
        }
        next["files"] = Value::Object(files);
        let target = mapped(owner, old)?;
        moved |= target != *owner;
        if owners.insert(target, next).is_some() {
            return Err(format!("adoption owner collision: {owner}").into());
        }
    }
    if !moved {
        return Err("no existing adoption owners under the declared legacy root".into());
    }
    Ok((
        json!({"schema":"viggobsidian.adoption.v1","owners":owners}),
        json!(protected),
    ))
}

fn integration(vault: &Path, old: &str) -> Result<(Value, String)> {
    let (mut state, before) = read_json(&guarded(vault, INTEGRATION)?)?;
    if state["schema"] != "viggobsidian.integration.v1" || state["profile"] != "numbered-v1" {
        return Err("relocation requires the existing numbered-v1 profile".into());
    }
    let legacy = state["adoption"]["legacy_project_roots"]
        .as_array_mut()
        .ok_or("legacy project roots were not declared")?;
    if legacy.iter().filter(|name| **name == old).count() != 1
        || legacy
            .iter()
            .any(|name| name == DEST || name.as_str().is_none())
    {
        return Err(
            "expected exactly one declaration for the source and none for the destination".into(),
        );
    }
    legacy.retain(|name| name != old);
    Ok((state, before))
}

const WATCH: &[&str] = &[
    ".viggobsidian/appearance.json",
    ".viggobsidian/installed.json",
    crate::SCHEMA,
    crate::TILE_CSS,
];
fn watched(vault: &Path) -> Result<Value> {
    let mut resources = Map::new();
    for &path in WATCH {
        resources.insert(path.to_owned(), fingerprint(&guarded(vault, path)?)?);
    }
    Ok(Value::Object(resources))
}

fn bytes(value: &Value) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec_pretty(value)?)
}

fn sha(state: &Value) -> Result<String> {
    Ok(hash(&bytes(state)?))
}

fn plan(vault: &Path, old: &str) -> Result<Value> {
    if appears(vault, DEST)? {
        return Err("destination root exists (including case-equivalent names)".into());
    }
    let contents = inventory(vault, old)?;
    let root_note = format!("{old}.md");
    let new_note = format!("{DEST}.md");
    if !contents.get(&root_note).is_some_and(Value::is_object)
        || contents
            .keys()
            .any(|path| path.eq_ignore_ascii_case(&new_note))
    {
        return Err("expected a regular source root note and no destination root note".into());
    }
    let (integration, integration_before) = integration(vault, old)?;
    let adoption = adoption::load(vault)?;
    let adoption_before = adoption["sha256"]
        .as_str()
        .ok_or("adoption preimage required")?;
    let (adoption, protected_external) = move_adoption(vault, &adoption, old)?;
    Ok(json!({
        "schema":PLAN,"vault":vault,"source":old,"destination":DEST,
        "tree":contents,"protected_external":protected_external,"watched":watched(vault)?,
        "project_routes":crate::project_codes::relative_routes(vault, Path::new(old))?,
        "adoption":{"before":adoption_before,"after":adoption,"mode":file_mode(&vault.join(adoption::STATE))?},
        "integration":{"before":integration_before,"after":integration,"mode":file_mode(&vault.join(INTEGRATION))?},
    }))
}

fn ensure_other_operators(vault: &Path) -> Result<()> {
    let dir = metadata(vault)?;
    for name in [
        "adoption.lock",
        "adoption.pending",
        "update.lock",
        "pending.json",
        "rollback.json",
    ] {
        if present(&dir.join(name))? {
            return Err(format!("conflicting vault maintenance: {name}").into());
        }
    }
    Ok(())
}

fn stored_sha(path: &Path) -> Result<Option<String>> {
    Ok(fingerprint(path)?["sha256"].as_str().map(str::to_owned))
}

fn recorded(vault: &Path, plan: &Value) -> Result<(bool, bool)> {
    let mut post = [false; 2];
    for (index, (key, path)) in [("adoption", adoption::STATE), ("integration", INTEGRATION)]
        .into_iter()
        .enumerate()
    {
        let path = guarded(vault, path)?;
        let got = stored_sha(&path)?;
        if plan[key]["mode"] != file_mode(&path)? {
            return Err(format!("{key} mode changed; retain maintenance hold").into());
        }
        post[index] = got.as_deref() == Some(sha(&plan[key]["after"])?.as_str());
        if !post[index] && got.as_deref() != plan[key]["before"].as_str() {
            return Err(format!("{key} pre/postimage drift; retain maintenance hold").into());
        }
    }
    if post[1] && !post[0] {
        return Err("config updated without the corresponding adoption state".into());
    }
    if watched(vault)? != plan["watched"] {
        return Err(
            "project schema, appearance or installation changed while relocation was pending"
                .into(),
        );
    }
    for (path, expected) in plan["protected_external"]
        .as_object()
        .ok_or("missing protected files")?
    {
        if fingerprint(&guarded(vault, path)?)? != *expected {
            return Err(format!("protected external legacy file changed: {path}").into());
        }
    }
    Ok((post[0], post[1]))
}

fn check_tree(vault: &Path, plan: &Value, moved: bool) -> Result<()> {
    let old = source(plan)?;
    let (existing, absent) = if moved { (DEST, old) } else { (old, DEST) };
    if appears(vault, absent)? {
        return Err(format!("conflicting/partial move: {absent} still exists").into());
    }
    let mut expected: BTreeMap<String, Value> = serde_json::from_value(plan["tree"].clone())?;
    if moved {
        let root_note = expected
            .remove(&format!("{old}.md"))
            .ok_or("missing original root note")?;
        expected.insert(format!("{DEST}.md"), root_note);
    }
    if inventory(vault, existing)? != expected {
        return Err(
            "project tree differs from reviewed inventory (including root note or link edits)"
                .into(),
        );
    }
    if crate::project_codes::relative_routes(vault, Path::new(existing))? != plan["project_routes"]
    {
        return Err("project codes or stable project/component IDs changed".into());
    }
    Ok(())
}

// Publish only fully synced JSON; the durable pending file determines recovery.
#[cfg(unix)]
fn file_mode(path: &Path) -> Result<u32> {
    use std::os::unix::fs::PermissionsExt;
    let meta = fs::symlink_metadata(path)?;
    let mode = meta.permissions().mode();
    if !meta.is_file() || mode & 0o7000 != 0 {
        return Err("expected regular metadata file without special permissions".into());
    }
    Ok(mode & 0o777)
}

#[cfg(not(unix))]
fn file_mode(_path: &Path) -> Result<u32> {
    Err("project relocation requires Unix file modes".into())
}

fn write_atomic(dir: &Path, dest: &Path, value: &Value, mode: u32) -> Result<()> {
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let staging = dir.join(format!(
        ".relocation-{}-{nonce}-{serial}.tmp",
        std::process::id()
    ));
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&staging)?;
    let result = (|| -> Result<()> {
        file.write_all(&bytes(value)?)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(mode))?;
        }
        #[cfg(not(unix))]
        let _ = mode;
        file.sync_all()?;
        fs::rename(&staging, dest)?;
        File::open(dir)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(staging);
    }
    result
}

fn report(status: &str, id: &str) -> Value {
    json!({"status":status,"plan_sha256":id})
}

pub(crate) fn execute(vault: &Path, mode: &str, input: &Value) -> Result<Value> {
    let dir = metadata(vault)?;
    let _gate = acquire(vault, mode != "plan")?;
    ensure_other_operators(vault)?;
    let pending = dir.join(JOURNAL);
    if mode == "plan" {
        if input["schema"] != "viggobsidian.project-relocation-request.v1"
            || input.as_object().is_none_or(|obj| obj.len() != 2)
            || present(&pending)?
        {
            return Err("expected an exact relocation request and no pending operation".into());
        }
        return plan(vault, source(input)?);
    }

    if input["schema"] != PLAN || input["vault"] != json!(vault) || input["destination"] != DEST {
        return Err("invalid plan for this vault".into());
    }
    let id = sha(input)?;
    let receipt = dir.join(format!("project-relocation-{id}.json"));
    if mode == "begin" {
        if present(&pending)? || present(&receipt)? || *input != plan(vault, source(input)?)? {
            return Err(
                "relocation already started, completed or changed; do not start it again".into(),
            );
        }
        write_atomic(&dir, &pending, input, 0o600)?;
        return Ok(report("prepared-for-obsidian", &id));
    }
    if !present(&pending)? {
        if matches!(mode, "recover" | "apply")
            && present(&receipt)?
            && stored_sha(&receipt)?.as_deref() == Some(id.as_str())
            && recorded(vault, input)? == (true, true)
        {
            check_tree(vault, input, true)?;
            return Ok(report("already-applied", &id));
        }
        return Err("relocation has no matching pending journal".into());
    }
    if stored_sha(&pending)?.as_deref() != Some(id.as_str()) || present(&receipt)? {
        return Err("pending relocation does not match this plan".into());
    }

    let (adopted, integrated) = recorded(vault, input)?;
    if mode == "abort" {
        if adopted || integrated {
            return Err("state already changed; finish with recover, not abort".into());
        }
        check_tree(vault, input, false)?;
        fs::remove_file(&pending)?;
        File::open(&dir)?.sync_all()?;
        return Ok(report("aborted-unmoved", &id));
    }
    check_tree(vault, input, true)?;
    if mode == "validate" {
        return Ok(report("verified-move-inventory", &id));
    }
    if !matches!(mode, "apply" | "recover") {
        return Err("unknown project relocation mode".into());
    }
    for (already_post, key, path) in [
        (adopted, "adoption", adoption::STATE),
        (integrated, "integration", INTEGRATION),
    ] {
        if !already_post {
            let path = vault.join(path);
            let mode = input[key]["mode"].as_u64().ok_or("missing file mode")? as u32;
            write_atomic(&dir, &path, &input[key]["after"], mode)?;
        }
    }
    if recorded(vault, input)? != (true, true) {
        return Err("relocation postimage missing; recover from the original plan".into());
    }
    check_tree(vault, input, true)?;
    fs::rename(&pending, receipt)?;
    File::open(&dir)?.sync_all()?;
    Ok(report("adoption-and-integration-rebound", &id))
}
