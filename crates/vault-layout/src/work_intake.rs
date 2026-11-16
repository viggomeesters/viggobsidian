//! Bounded, create-only local Raw-task intake. No provider/registry or status updates.
use crate::{activity_layout, event_header, iso_date, read_metadata, strings, text};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn field<'a>(data: &'a Value, key: &str) -> Result<&'a str> {
    text(data, key)
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| format!("missing {key}").into())
}
fn uuid() -> Result<String> {
    let mut bytes = [0_u8; 16];
    File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    bytes[6] = (bytes[6] & 15) | 64;
    bytes[8] = (bytes[8] & 63) | 128;
    let h = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &h[..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..]
    ))
}

fn context(root: &Path, raw: &Path) -> Result<(Value, Value, Value)> {
    if !raw.components().all(|p| matches!(p, Component::Normal(_))) {
        return Err("choose a vault-relative component Raw path".into());
    }
    let parts = raw.components().map(|p| p.as_os_str()).collect::<Vec<_>>();
    if parts.len() != 4 || parts[3] != "raw" {
        return Err("expected PROGRAM/PROJECT/COMPONENT/raw".into());
    }
    let schema: Value = serde_saphyr::from_str(&fs::read_to_string(root.join(crate::SCHEMA))?)?;
    if !strings(&schema["work_project_code_roots"]).any(|v| parts[0] == v) {
        return Err("Raw is outside configured Work project roots".into());
    }
    let mut checked = root.to_path_buf();
    for part in &parts {
        checked.push(part);
        if fs::symlink_metadata(&checked)?.is_symlink() {
            return Err("symlink in Raw path".into());
        }
    }
    let raw_abs = root.join(raw);
    let component = raw_abs.parent().ok_or("missing component")?;
    let project = component.parent().ok_or("missing project")?;
    let project_name = parts[1].to_str().ok_or("invalid project path")?;
    let component_name = parts[2].to_str().ok_or("invalid component path")?;
    let (owner, _) = read_metadata(&project.join(format!("{project_name}.md")))?
        .ok_or("missing project note")?;
    let (meta, _) = read_metadata(&component.join(format!("{component_name}.md")))?
        .ok_or("missing component note")?;
    if text(&owner, "type") != Some("project")
        || text(&meta, "category") != Some("project-component")
    {
        return Err("choose an existing component-raw Work project".into());
    }
    field(&owner, "project_id")?;
    field(&meta, "component-id")?;
    let code = field(&meta, "code")?;
    if !code.starts_with(&format!("{}-", field(&owner, "project-code")?))
        || code.contains(['\n', '\r'])
    {
        return Err("invalid component code".into());
    }
    Ok((owner, meta, schema))
}

fn inventory(root: &Path, raw: &Path, code: &str) -> Result<(String, Vec<Value>, Vec<String>)> {
    let prefix = format!("{code}-T");
    let mut maximum = 0_u64;
    let mut seen = std::collections::BTreeSet::new();
    let mut pins = Vec::new();
    let mut members = Vec::new();
    for entry in fs::read_dir(root.join(raw))? {
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "invalid Raw filename")?;
        if name.starts_with('.') {
            continue;
        }
        if entry.file_type()?.is_symlink() {
            return Err("symlink in component Raw".into());
        }
        members.push(name.clone());
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path().join(format!("{name}.md"));
        let bytes = match fs::symlink_metadata(&path) {
            Ok(m) if m.is_file() && !m.is_symlink() => Some(fs::read(&path)?),
            Ok(_) => return Err("invalid Raw primary note".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        pins.push(
            json!({"path":path.strip_prefix(root)?,"sha256":bytes.as_ref().map(|b|digest(b))}),
        );
        if let Some((data, _)) = bytes.as_deref().map(metadata).transpose()?.flatten()
            && text(&data, "type") == Some("task")
        {
            // Migrated Work tasks retain their original IDs without a numbered code.
            let Some(task_code) = text(&data, "task-code") else {
                continue;
            };
            let n = task_code
                .strip_prefix(&prefix)
                .and_then(|v| v.parse::<u64>().ok())
                .ok_or("invalid existing task-code")?;
            if n == 0 || task_code != format!("{prefix}{n:02}") || !seen.insert(n) {
                return Err("invalid or duplicate existing task-code".into());
            }
            maximum = maximum.max(n);
        }
    }
    pins.sort_by_key(|v| v["path"].as_str().unwrap().to_owned());
    members.sort();
    let n = maximum.checked_add(1).ok_or("task-code exhausted")?;
    Ok((format!("{prefix}{n:02}"), pins, members))
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn metadata(bytes: &[u8]) -> Result<Option<(Value, String)>> {
    let text = std::str::from_utf8(bytes)?;
    let Some((yaml, body)) = crate::frontmatter(text) else {
        return Ok(None);
    };
    Ok(serde_saphyr::from_str(&yaml).ok().map(|data| (data, body)))
}
fn regular(root: &Path, path: &Path) -> Result<Vec<u8>> {
    if !path.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err("expected a normalized vault-relative source path".into());
    }
    let mut current = root.to_path_buf();
    for part in path.components() {
        current.push(part);
        if fs::symlink_metadata(&current)?.is_symlink() {
            return Err("symlink in promotion source or owner".into());
        }
    }
    if !current.is_file() {
        return Err("promotion source is not a regular file".into());
    }
    Ok(fs::read(current)?)
}

/// Read-only allocation certificate. Caller owns the shared Raw lock and editor
/// transaction, rechecks every pin/membership and never treats this as a write.
pub(crate) fn promotion(root: &Path, input: Value) -> Result<Value> {
    let object = input.as_object().ok_or("expected promotion input")?;
    if object.len() != 4 || input["schema"] != "viggobsidian.work-task-promotion.v1" {
        return Err("invalid promotion schema or fields".into());
    }
    let source = PathBuf::from(field(&input, "path")?);
    let original = regular(root, &source)?;
    if input["source_sha256"] != digest(&original) {
        return Err("promotion source changed".into());
    }
    let id = field(&input, "request_id")?;
    if id.len() != 36
        || !id.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
    {
        return Err("promotion requires an invocation UUID".into());
    }
    let package = source.parent().ok_or("source package missing")?;
    let raw = package.parent().ok_or("Raw missing")?;
    let name = package
        .file_name()
        .ok_or("package name missing")?
        .to_str()
        .ok_or("invalid package encoding")?;
    if source.file_stem().and_then(|v| v.to_str()) != Some(name)
        || source.extension().and_then(|v| v.to_str()) != Some("md")
    {
        return Err("choose the primary Raw package note".into());
    }
    let (owner, meta, schema) = context(root, raw)?;
    let (mut data, body) = metadata(&original)?.ok_or("source metadata missing")?;
    if text(&data, "type") != Some("note")
        || text(&data, "wiki_status") == Some("processed")
        || matches!(text(&data, "layer"), Some("wiki" | "navigation"))
        || text(&data, "category") == Some("index")
        || strings(&data["cssclasses"]).any(|c| c == "folder-note" || c.starts_with("vault-wiki"))
        || !strings(&data["cssclasses"]).any(|c| c == "vault-event-v1-1")
    {
        return Err("choose an unprocessed own Event Note v1.1 source".into());
    }
    for key in [
        "source_identity",
        "source_inventory",
        "source_archives",
        "work_dossier",
        "task_owner",
        "registration_id",
        "claim_token",
        "generated",
        "generated_by",
        "generated_from",
        "compiled_from",
        "projection",
        "readonly",
    ] {
        if data.get(key).is_some() {
            return Err(format!("{key} remains with its bound source writer").into());
        }
    }
    if data.get("task-code").is_some() {
        return Err("a source with task-code retains its task writer".into());
    }
    event_header::check(root, &root.join(package), &data, &body)?;
    let (task_code, mut pins, members) = inventory(root, raw, field(&meta, "code")?)?;
    pins.retain(|p| p["path"].as_str() != source.to_str());
    let schema_bytes = regular(root, Path::new(crate::SCHEMA))?;
    let current_schema: Value = serde_saphyr::from_str(std::str::from_utf8(&schema_bytes)?)?;
    if current_schema != schema {
        return Err("promotion schema changed during planning".into());
    }
    pins.push(json!({"path":crate::SCHEMA,"sha256":digest(&schema_bytes)}));
    let component = raw.parent().ok_or("component missing")?;
    let project = component.parent().ok_or("project missing")?;
    for folder in [project, component] {
        let name = folder
            .file_name()
            .ok_or("owner name missing")?
            .to_str()
            .ok_or("owner encoding")?;
        let path = folder.join(format!("{name}.md"));
        let bytes = regular(root, &path)?;
        let (current, _) = metadata(&bytes)?.ok_or("owner metadata missing")?;
        if current
            != if folder == project {
                owner.clone()
            } else {
                meta.clone()
            }
        {
            return Err("promotion owner changed during planning".into());
        }
        pins.push(json!({"path":path,"sha256":digest(&bytes)}));
    }
    let task_id = match data.get("task-id") {
        Some(v) => v
            .as_str()
            .filter(|s| {
                s.starts_with("task:") && s.len() > 5 && !s.chars().any(char::is_whitespace)
            })
            .ok_or("invalid existing task identity")?
            .to_owned(),
        None => format!("task:{id}"),
    };
    for pin in &pins {
        if pin["sha256"].is_null() {
            continue;
        }
        let bytes = regular(root, Path::new(field(pin, "path")?))?;
        if pin["sha256"] != digest(&bytes) {
            return Err("promotion context changed during planning".into());
        }
        if let Some((other, _)) = metadata(&bytes)?
            && text(&other, "type") == Some("task")
            && text(&other, "task-id") == Some(task_id.as_str())
        {
            return Err("task identity already belongs to another Raw task".into());
        }
    }
    let title = text(&data, "name")
        .or_else(|| text(&data, "title"))
        .or_else(|| body.lines().find_map(|l| l.strip_prefix("# ")))
        .filter(|s| !s.trim().is_empty() && !s.contains(['\n', '\r']))
        .ok_or("source title missing")?;
    let task_name = format!("{task_code} · {title}");
    data["type"] = json!("task");
    data["status"] = json!("🔴 to-do");
    event_header::check(root, &root.join(package), &data, &body)?;
    Ok(
        json!({"schema":"viggobsidian.work-task-promotion-plan.v1","path":source,"raw":raw,"source_sha256":digest(&original),"task-id":task_id,"task-code":task_code,"name":task_name,"pins":pins,"members":members,"project_id":field(&owner,"project_id")?,"component-id":field(&meta,"component-id")?}),
    )
}

pub(crate) fn create(root: &Path, raw: &Path, input: Value) -> Result<Value> {
    let (_, meta, schema) = context(root, raw)?;
    let code = field(&meta, "code")?;
    let raw_abs = root.join(raw);
    let component = raw_abs.parent().ok_or("missing component")?;
    let day = field(&input, "date")?;
    let slug = field(&input, "slug")?;
    let title = field(&input, "title")?;
    let request = field(&input, "request")?;
    if !iso_date(day)
        || slug.starts_with('-')
        || slug.ends_with('-')
        || !slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        || title.contains(['\n', '\r'])
    {
        return Err("invalid date, kebab-case slug or single-line title".into());
    }
    if input
        .get("work_date")
        .is_some_and(|v| !v.as_str().is_some_and(iso_date))
    {
        return Err("work_date must be YYYY-MM-DD when supplied".into());
    }
    // Cooperating local producers share this lock. It is deliberately not a Syncthing lock.
    let lock_path = raw_abs.join(".work-intake.lock");
    if fs::symlink_metadata(&lock_path).is_ok_and(|m| !m.is_file() || m.is_symlink()) {
        return Err("invalid Work intake lock".into());
    }
    let lock = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)?;
    lock.try_lock()
        .map_err(|e| format!("Work intake busy: {e}"))?;
    let name = format!("{}-{slug}", day.replace('-', ""));
    let destination = raw_abs.join(&name);
    let note = destination.join(format!("{name}.md"));
    let digest = format!("{:x}", Sha256::digest(request.as_bytes()));
    if destination.exists() {
        if destination.is_symlink() || note.is_symlink() {
            return Err("existing task is a symlink".into());
        }
        let (old, _) = read_metadata(&note)?.ok_or("existing package has no task metadata")?;
        let old_code = field(&old, "task-code")?;
        if text(&old, "type") != Some("task")
            || text(&old, "request_sha256") != Some(&digest)
            || text(&old, "name") != Some(format!("{old_code} · {title}").as_str())
            || old.get("work_date") != input.get("work_date")
        {
            return Err("package exists with different input; nothing overwritten".into());
        }
        return Ok(
            json!({"path":note.strip_prefix(root)?,"task-id":field(&old,"task-id")?,"task-code":field(&old,"task-code")?,"reused":true}),
        );
    }
    let (task_code, _, _) = inventory(root, raw, code)?;
    let id = format!("task:{}", uuid()?);
    let mut data = json!({"type":"task","category":"project-task","task-id":id,"task-code":task_code,
        "name":format!("{task_code} · {title}"),"status":"🔴 to-do","created":day,"event_date":day,
        "wiki_status":"pending","request_sha256":digest,"activity_contract":"provenance-v1",
        "cssclasses":["vault-navigation","vault-dashboard","vault-event","vault-event-v1-1"]});
    if let Some(date) = input.get("work_date") {
        data["work_date"] = date.clone();
    }
    let breadcrumb = component
        .ancestors()
        .take_while(|p| *p != root)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|p| {
            let n = p.file_name().unwrap().to_string_lossy();
            format!("[[{}/{n}|{n}]]", p.strip_prefix(root).unwrap().display())
        })
        .collect::<Vec<_>>()
        .join(" / ");
    // Percent-encode URL delimiter bytes, leaving the actual local directory readable.
    let folder_url = destination
        .to_string_lossy()
        .replace('%', "%25")
        .replace(' ', "%20")
        .replace('#', "%23")
        .replace('?', "%3F");
    let actor = text(&schema, "activity_actor").unwrap_or("Human");
    if actor.trim().is_empty()
        || !actor
            .chars()
            .all(|c| c.is_alphabetic() || matches!(c, ' ' | '-' | '.'))
    {
        return Err("invalid local activity_actor".into());
    }
    let mut body = format!(
        "# {task_code} · {title}\n\n> [!navigation|shortcuts]\n> - [[{}/raw|Back]]\n> - [Finder](file://{folder_url})\n\n> [!wiki-facts|dossier]\n> - Dossier **{breadcrumb}**\n\n```vault-event-detail\n```\n\n> [!event-context] Context\n> Oorspronkelijke opdracht: {title}\n\n## Activity\n\n> [!activity-day|{day}] {day}\n\n> [!activity-message|human] {actor}\n",
        raw.display()
    );
    for line in request.split('\n') {
        body.push_str(&format!("> {line}\n"));
    }
    body.push_str(&format!(">\n> *— · {actor} · Work intake*\n"));
    event_header::check(root, &destination, &data, &body)?;
    activity_layout::check_task_event(&body)?;
    let mut content = String::from("---\n");
    for (k, v) in data.as_object().unwrap() {
        content.push_str(&format!("{k}: {v}\n"));
    }
    content.push_str("---\n\n");
    content.push_str(&body);
    // Stage complete bytes, then create the destination exclusively and publish via hard link.
    let stage = raw_abs.join(format!(".work-intake-stage-{}", uuid()?));
    fs::create_dir(&stage)?;
    let staged = stage.join("note.md");
    let mut file = File::create_new(&staged)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;
    fs::create_dir(&destination)?;
    if let Err(error) = fs::hard_link(&staged, &note) {
        let _ = fs::remove_dir(&destination);
        return Err(error.into());
    }
    File::open(&destination)?.sync_all()?;
    fs::remove_file(staged)?;
    fs::remove_dir(stage)?;
    Ok(json!({"path":note.strip_prefix(root)?,"task-id":id,"task-code":task_code,"reused":false}))
}
