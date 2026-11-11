use crate::{frontmatter, iso_date, text};
use serde_json::Value;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub struct Entry {
    pub id: String,
    pub key: String,
    pub path: PathBuf,
}

pub fn resolve(vault: &Path, project: Option<&Path>, query: &str) -> Result<Entry, String> {
    let query = query.to_ascii_uppercase();
    let (month, code) = match query.split_once('/') {
        Some((month, code)) if iso_date(&format!("{month}-01")) => (Some(month), code),
        Some(_) => return Err("expected YYYY-MM/CODE".into()),
        None => (None, query.as_str()),
    };
    if !valid_code(code) {
        return Err("expected a project or component code".into());
    }

    let mut roots = vec![vault.join("15-projects"), vault.join("20-projects")];
    if let Ok(schema) = fs::read_to_string(vault.join(crate::SCHEMA)) {
        let schema: Value = serde_saphyr::from_str(&schema).map_err(|error| error.to_string())?;
        for root in crate::strings(&schema["work_project_code_roots"]) {
            let path = Path::new(root);
            if !matches!(
                path.components().collect::<Vec<_>>().as_slice(),
                [std::path::Component::Normal(_)]
            ) {
                return Err(format!("invalid project-code root: {root}"));
            }
            let path = vault.join(path);
            if !roots.contains(&path) {
                roots.push(path);
            }
        }
    }
    let mut entries = Vec::new();
    if let Some(project) = project {
        let project = vault
            .join(project)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !roots
            .iter()
            .any(|root| project != *root && project.starts_with(root))
        {
            return Err("choose a project folder inside the vault".into());
        }
        if !index_project(vault, &project, &mut entries)? {
            return Err("project does not declare a project-month".into());
        }
    } else {
        for root in roots {
            if root.is_dir() && !root.is_symlink() {
                scan(vault, &root, &mut entries)?;
            }
        }
    }

    let mut matches: Vec<_> = entries
        .iter()
        .filter(|entry| {
            let (entry_month, entry_code) = entry.key.split_once('/').expect("qualified key");
            entry_code == code && month.is_none_or(|month| month == entry_month)
        })
        .collect();
    matches.sort_by(|a, b| (&a.key, &a.path).cmp(&(&b.key, &b.path)));
    match matches.as_slice() {
        [] => Err(format!("unknown code {query}")),
        [entry] if entries.iter().filter(|other| other.id == entry.id).count() == 1 => Ok(Entry {
            id: entry.id.clone(),
            key: entry.key.clone(),
            path: entry.path.clone(),
        }),
        [_] => Err(format!("duplicate stable ID for {query}")),
        _ => {
            let options: Vec<_> = matches
                .iter()
                .map(|entry| format!("{} ({})", entry.key, entry.path.display()))
                .collect();
            Err(format!("ambiguous code {query}: {}", options.join(", ")))
        }
    }
}

fn valid_code(code: &str) -> bool {
    !code.is_empty()
        && code
            .split('-')
            .all(|part| !part.is_empty() && part.bytes().all(|ch| ch.is_ascii_alphanumeric()))
}

pub fn validate_layout(folder: &Path, project: &Value) -> Result<(), String> {
    let slug = folder
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let month = text(project, "project-month")
        .filter(|month| iso_date(&format!("{month}-01")) && slug.starts_with(&format!("{month}-")))
        .ok_or("project-month must match the YYYY-MM project folder")?;
    let code = text(project, "project-code")
        .filter(|code| valid_code(code) && *code == code.to_ascii_uppercase())
        .ok_or("expected an uppercase project-code")?;
    if text(project, "project_id").is_none() {
        return Err(format!("{month}/{code} is missing project_id"));
    }
    let mut numbers = std::collections::BTreeSet::new();
    let mut ids = std::collections::BTreeSet::new();
    for child in directories(folder).map_err(|error| error.to_string())? {
        let name = child
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let expected = format!("{}-", code.to_ascii_lowercase());
        let (number, label) = name
            .strip_prefix(&expected)
            .and_then(|rest| rest.split_once('-'))
            .filter(|(number, label)| {
                number.len() == 2
                    && number.bytes().all(|byte| byte.is_ascii_digit())
                    && valid_code(label)
                    && *label == label.to_ascii_lowercase()
            })
            .ok_or_else(|| format!("{name}: expected {expected}NN-name in lowercase"))?;
        if (number == "00") != (label == "project-wiki") || !numbers.insert(number.to_owned()) {
            return Err(format!(
                "{name}: use a unique number and reserve 00 for Project Wiki"
            ));
        }
        let (_, meta) = folder_note(&child, "code:")?
            .ok_or_else(|| format!("{name}: missing component code"))?;
        let component_code = format!("{code}-{number}");
        if text(&meta, "category") != Some("project-component")
            || text(&meta, "code") != Some(&component_code)
            || !text(&meta, "component-id").is_some_and(|id| ids.insert(id.to_owned()))
        {
            return Err(format!(
                "{name}: expected code {component_code} and a unique component-id"
            ));
        }
    }
    if !numbers.contains("00") {
        return Err("missing 00 Project Wiki".into());
    }
    Ok(())
}

pub fn validate_registry(vault: &Path, roots: &Value) -> Result<(), String> {
    let mut entries = Vec::new();
    for root in crate::strings(roots) {
        let root = vault.join(root);
        if root.is_dir() && !root.is_symlink() {
            scan(vault, &root, &mut entries)?;
        }
    }
    let mut keys = std::collections::BTreeMap::new();
    let mut ids = std::collections::BTreeMap::new();
    for entry in &entries {
        for (value, previous) in [
            (&entry.key, keys.insert(&entry.key, &entry.path)),
            (&entry.id, ids.insert(&entry.id, &entry.path)),
        ] {
            if let Some(previous) = previous {
                return Err(format!(
                    "duplicate {value}: {} and {}",
                    previous.display(),
                    entry.path.display()
                ));
            }
        }
    }
    Ok(())
}

fn scan(vault: &Path, folder: &Path, entries: &mut Vec<Entry>) -> Result<(), String> {
    for child in directories(folder).map_err(|e| e.to_string())? {
        if !index_project(vault, &child, entries)? {
            scan(vault, &child, entries)?;
        }
    }
    Ok(())
}

fn directories(folder: &Path) -> io::Result<Vec<PathBuf>> {
    let mut children = Vec::new();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() && !entry.file_name().to_string_lossy().starts_with('.') {
            children.push(entry.path());
        }
    }
    Ok(children)
}

fn index_project(vault: &Path, folder: &Path, entries: &mut Vec<Entry>) -> Result<bool, String> {
    let Some((path, meta)) = folder_note(folder, "project-month:")? else {
        return Ok(false);
    };
    if text(&meta, "type") != Some("project") {
        return Ok(false);
    }
    let Some(month) = text(&meta, "project-month") else {
        return Ok(false);
    };
    if !iso_date(&format!("{month}-01")) {
        return Err(format!("invalid project-month in {}", path.display()));
    }
    let code = add(
        vault,
        &path,
        &meta,
        month,
        "project-code",
        "project_id",
        entries,
    )?;
    for child in directories(folder).map_err(|e| e.to_string())? {
        if let Some((path, meta)) = folder_note(&child, "category:")?
            && text(&meta, "category") == Some("project-component")
        {
            let component = add(vault, &path, &meta, month, "code", "component-id", entries)?;
            let number = component.strip_prefix(&format!("{code}-"));
            if !number.is_some_and(|n| n.len() == 2 && n.bytes().all(|b| b.is_ascii_digit())) {
                return Err(format!("invalid component code in {}", path.display()));
            }
        }
    }
    Ok(true)
}

fn folder_note(folder: &Path, field: &str) -> Result<Option<(PathBuf, Value)>, String> {
    let Some(name) = folder.file_name().and_then(|n| n.to_str()) else {
        return Ok(None);
    };
    let path = folder.join(format!("{name}.md"));
    if !path.is_file() || path.is_symlink() {
        return Ok(None);
    }
    let note = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let Some((yaml, _)) = frontmatter(&note) else {
        return Ok(None);
    };
    if !yaml.lines().any(|line| line.starts_with(field)) {
        return Ok(None);
    }
    let meta = serde_saphyr::from_str(&yaml).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Some((path, meta)))
}

fn add(
    vault: &Path,
    path: &Path,
    meta: &Value,
    month: &str,
    code_field: &str,
    id_field: &str,
    entries: &mut Vec<Entry>,
) -> Result<String, String> {
    let code = text(meta, code_field)
        .filter(|code| valid_code(code))
        .ok_or_else(|| format!("missing or invalid {code_field} in {}", path.display()))?
        .to_ascii_uppercase();
    let id =
        text(meta, id_field).ok_or_else(|| format!("missing {id_field} in {}", path.display()))?;
    entries.push(Entry {
        id: id.to_owned(),
        key: format!("{month}/{code}"),
        path: path
            .strip_prefix(vault)
            .expect("folder note inside vault")
            .to_owned(),
    });
    Ok(code)
}
