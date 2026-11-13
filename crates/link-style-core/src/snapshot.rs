use crate::{Note, edits, visible_body};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

pub const MAX_FILES: usize = 10_000;
pub const MAX_NOTE_BYTES: usize = 1024 * 1024;
pub const MAX_TOTAL_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = 24 * 1024 * 1024;

// The snapshot is an attestation by the capture adapter, not proof about a disk.
fn relative_markdown(path: &str) -> bool {
    path.ends_with(".md")
        && !path.contains(['\\', ':', '\0'])
        && path
            .split('/')
            .all(|part| !part.is_empty() && !part.starts_with('.'))
}

fn validate(input: &Value) -> Result<Value, &'static str> {
    if input["schema"] != "viggobsidian.link-style-snapshot.v1" || input["complete"] != true {
        return Err("complete_supported_snapshot_required");
    }
    let inventory = input["inventory"].as_array().ok_or("inventory_required")?;
    let documents = input["notes"].as_array().ok_or("notes_required")?;
    if inventory.len() > MAX_FILES || documents.len() > MAX_FILES {
        return Err("file_limit");
    }
    let mut names = BTreeSet::new();
    for entry in inventory {
        let path = entry.as_str().ok_or("invalid_inventory_path")?;
        if !relative_markdown(path) || !names.insert(path) {
            return Err("unsafe_or_duplicate_inventory_path");
        }
    }
    let mut seen = BTreeSet::new();
    let mut notes = Vec::new();
    let mut sources = std::collections::BTreeMap::new();
    let mut total = 0;
    for doc in documents {
        let path = doc["path"].as_str().ok_or("note_path_required")?;
        let source = doc["markdown"].as_str().ok_or("note_markdown_required")?;
        if !names.contains(path) || !seen.insert(path) {
            return Err("unexpected_or_duplicate_note");
        }
        total += source.len();
        if source.len() > MAX_NOTE_BYTES || total > MAX_TOTAL_BYTES {
            return Err("content_limit");
        }
        let name = Path::new(path).file_stem().unwrap().to_str().unwrap();
        let title = visible_body(source)
            .lines()
            .find_map(|line| line.strip_prefix("# "))
            .filter(|s| !s.contains("[["))
            .unwrap_or(name)
            .to_owned();
        notes.push(Note {
            path: path.into(),
            name: name.into(),
            title,
        });
        sources.insert(path, source);
    }
    if seen != names {
        return Err("missing_note_content");
    }
    notes.sort_by(|a, b| a.path.cmp(&b.path));
    let targets = match input.get("targets") {
        None | Some(Value::Null) => None,
        Some(Value::Array(paths)) => {
            let mut targets = BTreeSet::new();
            for value in paths {
                let path = value.as_str().ok_or("invalid_target")?;
                if !names.contains(path) || !targets.insert(path) {
                    return Err("unknown_or_duplicate_target");
                }
            }
            if targets.is_empty() {
                return Err("empty_targets");
            }
            Some(targets)
        }
        _ => return Err("invalid_targets"),
    };
    let mut issues = Vec::new();
    let mut checked = 0;
    let mut skipped_templates = 0;
    for note in &notes {
        if targets
            .as_ref()
            .is_some_and(|paths| !paths.contains(note.path.to_str().unwrap()))
        {
            continue;
        }
        if note.path.starts_with("90-templates") {
            skipped_templates += 1;
            continue;
        }
        checked += 1;
        let source = sources[note.path.to_str().unwrap()];
        for edit in edits(source, &note.path, &notes) {
            issues.push(json!({"path":note.path,
                "line":source[..edit.start].bytes().filter(|&b| b == b'\n').count()+1,
                "kind":edit.reason,"old":&source[edit.start..edit.end],"new":edit.replacement}));
        }
    }
    Ok(
        json!({"inventoried":notes.len(),"checked":checked,"skipped_templates":skipped_templates,
        "findings":issues.len(),"changed":{},"issues":issues}),
    )
}

/// Complete means this rule set evaluated the supplied snapshot, never all vault checks.
/// Invalid or incomplete input has no report that could be mistaken for a clean result.
pub fn respond(bytes: &[u8]) -> Value {
    let result = if bytes.len() > MAX_REQUEST_BYTES {
        Err("request_limit")
    } else {
        serde_json::from_slice(bytes)
            .map_err(|_| "invalid_json")
            .and_then(|value| validate(&value))
    };
    match result {
        Ok(report) => {
            json!({"schema":"viggobsidian.link-style-result.v1","status":"complete","report":report})
        }
        Err(reason) => {
            json!({"schema":"viggobsidian.link-style-result.v1","status":"incomplete","reason":reason})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> Value {
        json!({"schema":"viggobsidian.link-style-snapshot.v1","complete":true,
        "inventory":["a.md","folder/target.md"],"notes":[{"path":"a.md","markdown":"[[folder/target]]"},
        {"path":"folder/target.md","markdown":"# Target"}]})
    }
    #[test]
    fn absent_duplicate_unsafe_and_oversize_snapshots_are_never_clean() {
        let mut cases = Vec::new();
        let mut v = request();
        v["complete"] = json!(false);
        cases.push(v);
        let mut v = request();
        v["notes"].as_array_mut().unwrap().pop();
        cases.push(v);
        let mut v = request();
        let duplicate = v["notes"][0].clone();
        v["notes"].as_array_mut().unwrap().push(duplicate);
        cases.push(v);
        let mut v = request();
        v["inventory"][0] = json!("../a.md");
        cases.push(v);
        let mut v = request();
        v["inventory"][0] = json!("a\\b.md");
        cases.push(v);
        let mut v = request();
        v["notes"][0]["markdown"] = json!("x".repeat(MAX_NOTE_BYTES + 1));
        cases.push(v);
        let mut v = request();
        v["targets"] = json!(["missing.md"]);
        cases.push(v);
        for case in cases {
            let report = respond(&serde_json::to_vec(&case).unwrap());
            assert_eq!(report["status"], "incomplete");
            assert!(report.get("report").is_none());
        }
    }
    #[test]
    fn selected_notes_keep_the_whole_resolution_inventory() {
        let mut value = request();
        value["targets"] = json!(["a.md"]);
        let report = respond(&serde_json::to_vec(&value).unwrap());
        assert_eq!(report["report"]["checked"], 1);
        assert_eq!(report["report"]["inventoried"], 2);
        assert_eq!(report["report"]["issues"][0]["kind"], "visible-link-path");
    }
}
