//! Operator-reviewed adoption state. Content writers can read, never edit it.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) const STATE: &str = ".viggobsidian/adoption.json";
const SCHEMA: &str = "viggobsidian.adoption.v1";

pub(crate) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn safe(vault: &Path, raw: &str) -> Result<PathBuf> {
    if raw.split('/').any(|p| p.is_empty() || p.starts_with('.')) || raw.contains('\\') {
        return Err("invalid adoption path".into());
    }
    let path = vault.join(raw);
    for part in Path::new(raw)
        .ancestors()
        .filter(|p| !p.as_os_str().is_empty())
    {
        if fs::symlink_metadata(vault.join(part)).is_ok_and(|m| m.is_symlink()) {
            return Err("symlink in adoption path".into());
        }
    }
    Ok(path)
}

pub(crate) fn load(vault: &Path) -> Result<Value> {
    let dir = vault.join(".viggobsidian");
    if fs::symlink_metadata(&dir).is_ok_and(|m| m.is_symlink())
        || fs::symlink_metadata(dir.join("adoption.json"))
            .is_ok_and(|m| !m.is_file() || m.is_symlink())
    {
        return Err("adoption state must be a regular local file".into());
    }
    let bytes = match fs::read(vault.join(STATE)) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(json!({"sha256":null,"owners":{}}));
        }
        Err(e) => return Err(e.into()),
    };
    let options =
        serde_saphyr::options! { duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error };
    let value: Value = serde_saphyr::from_str_with_options(std::str::from_utf8(&bytes)?, options)?;
    if value["schema"] != SCHEMA || value.as_object().is_none_or(|o| o.len() != 2) {
        return Err("invalid adoption state schema".into());
    }
    let owners = value["owners"]
        .as_object()
        .ok_or("adoption owners missing")?;
    for (owner, entry) in owners {
        safe(vault, owner)?;
        if !matches!(entry["state"].as_str(), Some("legacy" | "strict"))
            || entry.as_object().is_none_or(|o| o.len() != 2)
        {
            return Err("invalid adoption owner state".into());
        }
        for (path, proof) in entry["files"]
            .as_object()
            .ok_or("adoption file proofs missing")?
        {
            safe(vault, path)?;
            if !Path::new(path).starts_with(owner)
                || proof.as_object().is_none_or(|p| p.len() != 2)
                || proof["bytes"].as_u64().is_none()
                || proof["sha256"]
                    .as_str()
                    .is_none_or(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
            {
                return Err("invalid adoption file proof".into());
            }
        }
    }
    Ok(json!({"sha256":hash(&bytes),"owners":owners}))
}

pub(crate) fn owner<'a>(state: &'a Value, path: &str) -> Option<(&'a str, &'a Value)> {
    state["owners"]
        .as_object()?
        .iter()
        .filter(|(p, _)| Path::new(path).starts_with(p))
        .max_by_key(|(p, _)| p.len())
        .map(|(p, v)| (p.as_str(), v))
}

// Intentionally finite: integrity, source inventory, manifests and identity never
// qualify. New checker rules must be reviewed before being grandfathered.
pub(crate) fn formatting(vault: &Path, row: &Value) -> Result<bool> {
    let code = row["code"].as_str().ok_or("finding code missing")?;
    let message = row["message"].as_str().ok_or("finding message missing")?;
    if code == "strict-event-header" {
        return Ok(message
            == "after Activity use only dated message bubbles; put provenance, source content and archives inside bubbles"
            || message == "use the vault-navigation, vault-dashboard and vault-event classes"
            || message == "start the event with its title"
            || message == "use one shortcut row containing Back and Finder below the title"
            || message == "use the Event Note v1.1 cards below the dossier path"
            || message == "put Activity immediately after the Event Note v1.1 header");
    }
    if code == "strict-activity-provenance" {
        if !message.contains(": start the bubble with ")
            && !message.contains(": end the bubble with ")
        {
            return Ok(false);
        }
        let path = row["path"].as_str().ok_or("finding path missing")?;
        let body = fs::read_to_string(safe(vault, path)?)?;
        let lines: Vec<_> = body.lines().collect();
        // Missing historical stamps are presentation debt; a supplied false
        // author/time is never grandfathered under the same broad error code.
        for (i, line) in lines.iter().enumerate() {
            if let Some((_, author)) = crate::activity_layout::message_heading(line)
                && crate::activity_delivery::header_stamp(line, author) == Some(false)
            {
                return Ok(false);
            }
            if let Some((_, author)) = crate::activity_layout::message_heading(line)
                && let Some(stamp) = lines
                    .get(i + 1)
                    .and_then(|l| l.strip_prefix("> `"))
                    .and_then(|s| s.strip_suffix('`'))
                && !crate::activity_delivery::valid_stamp(stamp, author)
            {
                return Ok(false);
            }
        }
        return Ok(true);
    }
    Ok(matches!(
        code,
        "folder-note"
            | "layer"
            | "missing-dashboard-class"
            | "loose-file"
            | "missing-tile"
            | "activity-message"
            | "strict-package-location"
            | "strict-dossier-dashboard"
            | "strict-dossier-shortcuts"
            | "strict-navigation-content"
            | "strict-navigation-shortcuts"
            | "raw-hub-content"
            | "raw-hub-table"
            | "raw-package-wrapper"
            | "wiki-preamble"
            | "wiki-style"
            | "table-wikilink-pipe"
    ))
}

/// Existing prefix is immutable; every accumulated addition is checked anew.
pub(crate) fn retained(vault: &Path, state: &Value, path: &str) -> Result<bool> {
    let Some((_, entry)) = owner(state, path) else {
        return Ok(false);
    };
    if entry["state"] != "legacy" {
        return Ok(false);
    }
    let proof = &entry["files"][path];
    let Some(length) = proof["bytes"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
    else {
        return Ok(false);
    };
    let bytes = fs::read(safe(vault, path)?)?;
    let Some(prefix) = bytes.get(..length) else {
        return Ok(false);
    };
    if proof["sha256"] != hash(prefix) {
        return Ok(false);
    }
    if bytes.len() == length {
        return Ok(true);
    }
    if !path.ends_with(".md") {
        return Ok(false);
    }
    let tail = std::str::from_utf8(&bytes[length..])?;
    if !(tail.starts_with("\n\n") || prefix.ends_with(b"\n") && tail.starts_with('\n')) {
        return Ok(false);
    }
    let original = std::str::from_utf8(prefix)?;
    // A legacy unclosed top-level fence must not swallow the new contribution.
    let mut fence = None;
    for line in original.lines() {
        let line = line.trim_start_matches(' ');
        let character = line.as_bytes().first().copied().unwrap_or(0);
        let width = line.bytes().take_while(|b| *b == character).count();
        match fence {
            Some((opening, minimum))
                if character == opening && width >= minimum && line[width..].trim().is_empty() =>
            {
                fence = None
            }
            None if matches!(character, b'`' | b'~') && width >= 3 => {
                fence = Some((character, width))
            }
            _ => {}
        }
    }
    if fence.is_some() {
        return Ok(false);
    }
    let day = |line: &str| {
        line.strip_prefix("> [!activity-day|")
            .and_then(|s| s.split_once("] "))
            .map(|(date, _)| date.to_owned())
    };
    let latest = original
        .lines()
        .filter_map(day)
        .filter(|d| crate::iso_date(d))
        .max();
    let first = tail.lines().find_map(day);
    if latest
        .zip(first)
        .is_some_and(|(before, after)| before > after)
    {
        return Ok(false);
    }
    for line in tail
        .lines()
        .filter(|l| l.starts_with("> <!-- viggobsidian:activity:v1 "))
    {
        if original.lines().any(|old| old == line) {
            return Ok(false);
        }
    }
    // A separate day prevents historical parser state from hiding a new bubble.
    if !tail.trim_start().starts_with("> [!activity-day|") {
        return Ok(false);
    }
    let body = format!("## Activity\n\n{tail}");
    Ok(
        crate::activity_delivery::provenance_findings(&body).is_empty()
            && crate::activity_delivery::findings(&body).is_empty()
            && crate::mail_draft::findings(&body).is_empty(),
    )
}

fn inventory(vault: &Path, dir: &Path, files: &mut BTreeMap<String, Value>) -> Result<()> {
    for item in fs::read_dir(dir)? {
        let path = item?.path();
        let raw = path
            .strip_prefix(vault)?
            .to_str()
            .ok_or("non-UTF8 adoption path")?;
        if path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with('.'))
        {
            continue;
        }
        safe(vault, raw)?;
        if path.is_dir() {
            inventory(vault, &path, files)?;
        } else if path.is_file() {
            let bytes = fs::read(&path)?;
            files.insert(
                raw.into(),
                json!({"bytes":bytes.len(),"sha256":hash(&bytes)}),
            );
        } else {
            return Err("non-regular adoption member".into());
        }
    }
    Ok(())
}

fn plan(vault: &Path, request: &Value) -> Result<Value> {
    if request["schema"] != "viggobsidian.adoption-request.v1"
        || request.as_object().is_none_or(|o| o.len() != 2)
    {
        return Err("invalid adoption request".into());
    }
    let previous = load(vault)?;
    let mut owners = previous["owners"].as_object().unwrap().clone();
    let mut scans = BTreeMap::new();
    for (path, choice) in request["owners"].as_object().ok_or("owners missing")? {
        let dir = safe(vault, path)?;
        if !dir.is_dir() {
            return Err("adoption owner is not a directory".into());
        }
        if !matches!(choice.as_str(), Some("legacy" | "strict")) {
            return Err("unknown adoption choice".into());
        }
        if owners.get(path).is_some_and(|v| v["state"] == "strict") && choice != "strict" {
            return Err("strict adoption cannot be downgraded".into());
        }
        let output = Command::new(std::env::current_exe()?)
            .arg("--vault")
            .arg(vault)
            .args(["--strict", path, "--json"])
            .output()?;
        if !matches!(output.status.code(), Some(0 | 1)) {
            return Err(format!(
                "strict adoption scan failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        let scan: Value = serde_json::from_slice(&output.stdout)?;
        if choice == "strict" && !output.status.success() {
            return Err(
                "strict promotion requires a clean complete scan and operator source acceptance"
                    .into(),
            );
        }
        let mut files = BTreeMap::new();
        inventory(vault, &dir, &mut files)?;
        if choice == "legacy" && owners.contains_key(path) {
            // Replanning must never re-baseline bytes already under a contract.
            return Err(
                "existing legacy owner must be preserved or promoted, not re-baselined".into(),
            );
        }
        // A child added later may tighten an existing parent, never loosen it.
        if choice == "legacy" && owner(&previous, path).is_some() {
            return Err("cannot add legacy beneath an adopted owner".into());
        }
        owners.insert(path.clone(), json!({"state":choice,"files":files}));
        scans.insert(path.clone(), scan);
    }
    for (path, entry) in &owners {
        if entry["state"] == "legacy"
            && owners.iter().any(|(parent, value)| {
                parent != path && Path::new(path).starts_with(parent) && value["state"] == "strict"
            })
        {
            return Err("legacy child cannot weaken a strict ancestor".into());
        }
    }
    Ok(
        json!({"schema":"viggobsidian.adoption-plan.v1","vault":vault,"request":request,"previous_sha256":previous["sha256"],"next":{"schema":SCHEMA,"owners":owners},"scans":scans}),
    )
}

pub(crate) fn run(vault: &Path, mode: &str, input: Value) -> Result<u8> {
    if mode == "plan" {
        println!("{}", serde_json::to_string_pretty(&plan(vault, &input)?)?);
        return Ok(0);
    }
    if input["schema"] != "viggobsidian.adoption-plan.v1" || input["vault"] != json!(vault) {
        return Err("invalid adoption plan".into());
    }
    let current = load(vault)?;
    let dir = vault.join(".viggobsidian");
    fs::create_dir_all(&dir)?;
    let lock = dir.join("adoption.lock");
    let staging = dir.join("adoption.pending");
    let bytes = serde_json::to_vec_pretty(&input["next"])?;
    let receipt = dir.join(format!("adoption-{}.json", hash(&bytes)));
    let identity = hash(&serde_json::to_vec(&input)?);
    let postimage = current["sha256"] == hash(&bytes);
    let receipt_value = if receipt.exists() {
        if fs::symlink_metadata(&receipt)?.is_symlink() {
            return Err("symlink adoption receipt".into());
        }
        let value: Value = serde_json::from_slice(&fs::read(&receipt)?)?;
        if value["plan"] != input {
            return Err("adoption receipt conflict".into());
        }
        Some(value)
    } else {
        None
    };
    if mode == "recover" {
        if receipt_value.is_none() || (!postimage && current["sha256"] != input["previous_sha256"])
        {
            return Err(
                "adoption recovery requires receipt and exact preimage or postimage".into(),
            );
        }
        for (path, expected) in [(&lock, identity.as_bytes()), (&staging, bytes.as_slice())] {
            if path.exists()
                && (fs::symlink_metadata(path)?.is_symlink() || fs::read(path)? != expected)
            {
                return Err("adoption recovery member changed".into());
            }
        }
        if !postimage && input != plan(vault, &input["request"])? {
            return Err("adoption recovery source drift".into());
        }
        // Only exact members of the reviewed interrupted transaction are removed.
        if staging.exists() {
            fs::remove_file(&staging)?;
        }
        if lock.exists() {
            fs::remove_file(&lock)?;
        }
    }
    if postimage {
        if receipt_value.is_none() || lock.exists() || staging.exists() {
            return Err("adoption recovery required".into());
        }
        println!("{}", json!({"status":"already-applied","state":current}));
        return Ok(0);
    }
    if input != plan(vault, &input["request"])? {
        return Err("adoption plan drift; review a new plan".into());
    }
    let mut guard = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)?;
    guard.write_all(identity.as_bytes())?;
    guard.sync_all()?;
    let result = (|| -> Result<()> {
        if input != plan(vault, &input["request"])? {
            return Err("adoption plan changed while locking".into());
        }
        if receipt_value.is_none() {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&receipt)?;
            file.write_all(&serde_json::to_vec_pretty(&json!({"plan":input,"previous":fs::read(vault.join(STATE)).ok().map(|b|String::from_utf8_lossy(&b).into_owned())}))?)?;
            file.sync_all()?;
        }
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(staging, vault.join(STATE))?;
        fs::File::open(&dir)?.sync_all()?;
        Ok(())
    })();
    fs::remove_file(lock)?;
    result?;
    println!("{}", json!({"status":"applied","state":load(vault)?}));
    Ok(0)
}
