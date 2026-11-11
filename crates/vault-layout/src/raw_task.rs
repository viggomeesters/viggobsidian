use crate::{Scan, iso_date, package_name, read_metadata, text};
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

fn required<'a>(data: &'a Value, key: &str) -> io::Result<&'a str> {
    data[key]
        .as_str()
        .ok_or_else(|| io::Error::other(format!("missing Raw task input: {key}")))
}

fn write_note(path: &Path, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()
}

/// The intake adapter holds the shared lock and publishes only a validated stage.
pub fn stage(scan: &mut Scan<'_>, raw: &Path, input: Value) -> io::Result<()> {
    let rel = raw.strip_prefix(scan.vault).map_err(io::Error::other)?;
    let parts: Vec<_> = rel.iter().collect();
    if parts.len() != 4
        || !(matches!(parts[0].to_str(), Some("15-projects" | "20-projects"))
            || parts[0].to_str().is_some_and(|root| {
                crate::strings(&scan.schema["work_project_code_roots"]).any(|item| item == root)
            }))
        || parts[3] != "raw"
    {
        return Err(io::Error::other("project_task_requires_component_raw"));
    }
    let component = raw.parent().expect("component parent");
    let project = component.parent().expect("project parent");
    let project_name = parts[1]
        .to_str()
        .ok_or_else(|| io::Error::other("invalid project name"))?;
    let component_name = parts[2]
        .to_str()
        .ok_or_else(|| io::Error::other("invalid component name"))?;
    let owner = read_metadata(&project.join(format!("{project_name}.md")))?;
    if !owner.is_some_and(|(meta, _)| {
        text(&meta, "type") == Some("project")
            && text(&meta, "task_layout") == Some("component-raw")
    }) {
        return Err(io::Error::other("project_task_requires_component_raw"));
    }
    let stage = Path::new(required(&input, "stage")?);
    if stage.parent() != Some(raw)
        || !stage
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(".vault-task-stage-"))
        || !fs::symlink_metadata(stage)?.is_dir()
        || fs::read_dir(stage)?.next().is_some()
    {
        return Err(io::Error::other("invalid Raw task stage"));
    }
    let name = required(&input, "name")?;
    let day = required(&input, "day")?;
    if !package_name(name) || !iso_date(day) || !name.starts_with(&day.replace('-', "")) {
        return Err(io::Error::other("invalid Raw task package name or date"));
    }
    scan.folder(component)?;
    if !scan.findings.is_empty() {
        return Ok(());
    }

    let (component_data, _) = read_metadata(&component.join(format!("{component_name}.md")))?
        .ok_or_else(|| io::Error::other("missing component note"))?;
    let code = required(&component_data, "code")?;
    let prefix = format!("{code}-T");
    let mut maximum = 0_u64;
    for entry in fs::read_dir(raw)? {
        let entry = entry?;
        let package_name = entry.file_name();
        let Some(package_name) = package_name.to_str() else {
            continue;
        };
        if package_name.starts_with('.') || !entry.file_type()?.is_dir() {
            continue;
        }
        if let Some((data, _)) = read_metadata(&entry.path().join(format!("{package_name}.md")))?
            && text(&data, "type") == Some("task")
        {
            let number = required(&data, "task-code")?
                .strip_prefix(&prefix)
                .and_then(|suffix| suffix.parse::<u64>().ok())
                .ok_or_else(|| io::Error::other("invalid_component_task_code"))?;
            maximum = maximum.max(number);
        }
    }
    let number = maximum
        .checked_add(1)
        .ok_or_else(|| io::Error::other("component_task_code_exhausted"))?;
    let task_code = format!("{prefix}{number:02}");
    let title = required(&input, "title")?;
    let request = required(&input, "request")?;
    let mut fields = vec![
        ("type", json!("task")),
        ("category", json!("project-task")),
        ("task-id", json!(required(&input, "task_id")?)),
        ("status", json!("🔴 to-do")),
        ("name", json!(format!("{task_code} · {title}"))),
        ("created", json!(day)),
        ("updated", json!(day)),
        ("request_sha256", json!(required(&input, "request_sha256")?)),
        ("task-code", json!(task_code)),
        ("event_date", json!(day)),
        ("wiki_status", json!("pending")),
    ];
    for key in [
        "uitvoerder",
        "source_identity",
        "source_note",
        "source_path",
        "source_observed_sha256",
    ] {
        if let Some(value) = text(&input, key) {
            fields.push((key, json!(value)));
        }
    }
    let complex = input["complex"] == true;
    if complex {
        fields.push(("work_dossier", json!(format!("{name}-werkdossier.md"))));
    }
    fields.push(("project", json!(format!("[[{project_name}]]"))));
    let mut note = String::from("---\n");
    for (key, value) in fields {
        note.push_str(&format!("{key}: {value}\n"));
    }
    note.push_str("cssclasses: [folder-note]\n---\n\n");
    let package = stage.join(name);
    fs::create_dir(&package)?;
    if complex {
        let purpose = title
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace('|', "—");
        let folder_url = required(&input, "folder_url")?;
        let note_url = required(&input, "note_url")?;
        note.push_str(&format!(
            "> [!abstract] About\n> {purpose}\n\n<!-- vault-task-hub:start -->\n## Laatste status\n\nOpdracht vastgelegd; nog niet uitgevoerd.\n\n## Navigation\n\n| Link | Description |\n|---|---|\n| [{name}-werkdossier.md]({name}-werkdossier.md) | Opdracht en uitvoering |\n| [Folder]({folder_url}) | This task package |\n| [Obsidian]({note_url}) | Open this task note |\n\n<!-- vault-task-hub:end -->\n"
        ));
        let owner = json!(format!("{name}.md"));
        let mut dossier = format!(
            "---\ntype: reference\ncategory: task-workdossier\ntask_owner: {owner}\n---\n\n# Werkdossier\n\n[Taak en actuele status]({name}.md)\n\n# 1 Opdracht\n\n## Oorspronkelijk verzoek\n\n"
        );
        for line in request.split('\n') {
            dossier.push_str(&format!("> {line}\n"));
        }
        write_note(&package.join(format!("{name}-werkdossier.md")), &dossier)?;
    } else {
        note.push_str(request);
        if let Some(source) = text(&input, "source_note") {
            if !request.is_empty() {
                note.push_str("\n\n");
            }
            note.push_str(source);
            note.push('\n');
        }
    }
    write_note(&package.join(format!("{name}.md")), &note)?;
    scan.folder_with_raw_parent(&package, true)?;
    scan.check_component_tasks(raw, &[package])?;
    Ok(())
}
