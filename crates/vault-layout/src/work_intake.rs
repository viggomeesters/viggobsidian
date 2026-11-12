//! Bounded, create-only local Raw-task intake. No provider/registry or status updates.
use crate::{activity_layout, event_header, iso_date, read_metadata, strings, text};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path},
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

pub(crate) fn create(root: &Path, raw: &Path, input: Value) -> Result<Value> {
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
    let prefix = format!("{code}-T");
    let mut maximum = 0_u64;
    let mut seen = std::collections::BTreeSet::new();
    for entry in fs::read_dir(&raw_abs)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        if entry.file_type()?.is_symlink() {
            return Err("symlink in component Raw".into());
        }
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path().join(format!("{name}.md"));
        if let Some((data, _)) = read_metadata(&path)?
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
    let n = maximum.checked_add(1).ok_or("task-code exhausted")?;
    let task_code = format!("{prefix}{n:02}");
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
        "# {task_code} · {title}\n\n> [!navigation|shortcuts]\n> - [[{}/raw|Back]]\n> - [Finder](file://{folder_url})\n\n> [!wiki-facts|dossier]\n> - Dossier **{breadcrumb}**\n\n```vault-event-detail\n```\n\n> [!event-context] Context\n> Oorspronkelijke opdracht: {title}\n\n## Activity\n\n> [!activity-day|{day}] {day}\n\n> [!activity-message|human] {actor} <span class=\"activity-meta\">— · Work intake</span>\n",
        raw.display()
    );
    for line in request.split('\n') {
        body.push_str(&format!("> {line}\n"));
    }
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
