//! Bounded visible-entrance experiment, not native filesystem eligibility.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[cfg(target_arch = "wasm32")]
mod wasm;

pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;
pub const MAX_ENTRIES: usize = 10_000;
pub const MAX_FOLDERS: usize = 1_000;
pub const MAX_FINDINGS: usize = 100;
const RESULT: &str = "viggobsidian.structure-result.v1";
const CAPABILITY: &str = "visible-folder-entrance.v1";

/// Native callers establish this fact using symlink_metadata; a portable host
/// can only establish its visible equivalent, explicitly named in its result.
#[derive(Clone, Copy)]
pub enum Entrance {
    File,
    Missing,
    NotFile,
}

/// Exact existing Scan::note codes/messages, independent of discovery/policy.
pub fn entrance_finding(entrance: Entrance) -> Option<(&'static str, &'static str)> {
    match entrance {
        Entrance::File => None,
        Entrance::Missing => Some(("folder-note", "missing matching folder note")),
        Entrance::NotFile => Some(("folder-note", "entrance must be a regular Markdown file")),
    }
}

fn path(value: &Value) -> Result<&str, &'static str> {
    let path = value.as_str().ok_or("path_required")?;
    if path.len() > 1024
        || path.contains(['\\', ':', '\0'])
        || path
            .split('/')
            .any(|part| part.is_empty() || part.starts_with('.'))
    {
        return Err("unsafe_visible_path");
    }
    Ok(path)
}

fn within(path: &str, scope: &str) -> bool {
    path == scope
        || path
            .strip_prefix(scope)
            .is_some_and(|tail| tail.starts_with('/'))
}

fn evaluate(input: &Value) -> Result<Value, &'static str> {
    if input["schema"] != "viggobsidian.structure-snapshot.v1" {
        return Err("unsupported_schema");
    }
    if input["ruleset"] != "visible-folder-entrance.v1" {
        return Err("unsupported_ruleset");
    }
    if input["complete"] != true {
        return Err("complete_inventory_required");
    }
    let profile = input["profile"].as_str().ok_or("profile_required")?;
    if profile != "numbered-v1" {
        return Ok(
            json!({"schema":RESULT,"status":"not_applicable","reason":"profile_not_selected"}),
        );
    }
    if input["capabilities"] != json!([CAPABILITY]) {
        return Err("required_capability_missing");
    }
    let before = input["capture"]["before"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 128)
        .ok_or("capture_required")?;
    let after = input["capture"]["after"]
        .as_str()
        .ok_or("capture_required")?;
    if before != after {
        return Err("snapshot_changed");
    }
    let scope = path(&input["scope"])?;
    let entries = input["entries"].as_array().ok_or("entries_required")?;
    let folders = input["folders"].as_array().ok_or("folders_required")?;
    if entries.len() > MAX_ENTRIES || folders.len() > MAX_FOLDERS {
        return Err("inventory_limit");
    }
    if folders.is_empty() {
        return Err("selected_folders_required");
    }
    let mut inventory = BTreeMap::new();
    for entry in entries {
        let entry_path = path(&entry["path"])?;
        if !within(entry_path, scope) {
            return Err("entry_outside_scope");
        }
        let kind = entry["kind"]
            .as_str()
            .filter(|kind| matches!(*kind, "file" | "folder"))
            .ok_or("entry_kind_required")?;
        if inventory.insert(entry_path, kind).is_some() {
            return Err("duplicate_entry");
        }
    }
    if inventory.get(scope) != Some(&"folder") {
        return Err("scope_folder_required");
    }
    for entry in inventory.keys().filter(|entry| **entry != scope) {
        let parent = entry
            .rsplit_once('/')
            .map(|(parent, _)| parent)
            .ok_or("parent_folder_required")?;
        if inventory.get(parent) != Some(&"folder") {
            return Err("parent_folder_required");
        }
    }
    let mut selected = BTreeSet::new();
    for folder in folders {
        let folder = path(folder)?;
        if !within(folder, scope) || inventory.get(folder) != Some(&"folder") {
            return Err("selected_folder_missing");
        }
        if !selected.insert(folder) {
            return Err("duplicate_selected_folder");
        }
    }
    let mut findings = Vec::new();
    for folder in selected {
        let name = folder.rsplit('/').next().unwrap();
        let expected = format!("{folder}/{name}.md");
        let entrance = match inventory.get(expected.as_str()) {
            Some(&"file") => Entrance::File,
            Some(_) => Entrance::NotFile,
            None => Entrance::Missing,
        };
        if let Some((code, message)) = entrance_finding(entrance) {
            findings.push(json!({"path":expected,"code":code,"message":message}));
            if findings.len() > MAX_FINDINGS {
                return Err("finding_limit");
            }
        }
    }
    Ok(
        json!({"schema":RESULT,"status":"complete","capabilities":[CAPABILITY],"scope":scope,"checked_folders":folders.len(),"findings":findings}),
    )
}

pub fn respond(bytes: &[u8]) -> Value {
    let result = if bytes.len() > MAX_REQUEST_BYTES {
        Err("request_limit")
    } else {
        serde_json::from_slice(bytes)
            .map_err(|_| "invalid_json")
            .and_then(|input| evaluate(&input))
    };
    result.unwrap_or_else(|reason| json!({"schema":RESULT,"status":"incomplete","reason":reason}))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Value {
        serde_json::from_str(include_str!("../fixtures/dossier.json")).unwrap()
    }
    #[test]
    fn frozen_fictional_dossier_preserves_native_messages() {
        let fixture = fixture();
        assert_eq!(
            respond(&serde_json::to_vec(&fixture["input"]).unwrap()),
            fixture["expected"]
        );
    }
    #[test]
    fn required_input_and_drift_cannot_pass_clean() {
        let base = fixture()["input"].clone();
        for key in base.as_object().unwrap().keys() {
            let mut input = base.clone();
            input.as_object_mut().unwrap().remove(key);
            assert_eq!(
                respond(&serde_json::to_vec(&input).unwrap())["status"],
                "incomplete",
                "missing {key}"
            );
        }
        for (key, value, reason) in [
            (
                "capture",
                json!({"before":"one","after":"two"}),
                "snapshot_changed",
            ),
            ("capabilities", json!([]), "required_capability_missing"),
            ("entries", json!([]), "scope_folder_required"),
            ("folders", json!([]), "selected_folders_required"),
            ("scope", json!("../fiction"), "unsafe_visible_path"),
        ] {
            let mut input = base.clone();
            input[key] = value;
            assert_eq!(
                respond(&serde_json::to_vec(&input).unwrap())["reason"],
                reason
            );
        }
    }
    #[test]
    fn malformed_inventory_never_becomes_absence() {
        let base = fixture()["input"].clone();
        for entry in [
            json!({"path":"fiction/x","kind":"unknown"}),
            json!({"path":"elsewhere","kind":"folder"}),
            json!({"path":"fiction/missing/child","kind":"folder"}),
            json!({"path":"fiction","kind":"folder"}),
        ] {
            let mut input = base.clone();
            input["entries"].as_array_mut().unwrap().push(entry);
            assert_eq!(
                respond(&serde_json::to_vec(&input).unwrap())["status"],
                "incomplete"
            );
        }
    }
    #[test]
    fn limits_never_return_a_truncated_success() {
        let mut input = fixture()["input"].clone();
        input["entries"] = json!(vec![
            json!({"path":"fiction","kind":"folder"});
            MAX_ENTRIES + 1
        ]);
        assert_eq!(
            respond(&serde_json::to_vec(&input).unwrap())["reason"],
            "inventory_limit"
        );
        input = fixture()["input"].clone();
        input["folders"] = json!(vec!["fiction"; MAX_FOLDERS + 1]);
        assert_eq!(
            respond(&serde_json::to_vec(&input).unwrap())["reason"],
            "inventory_limit"
        );
        input = fixture()["input"].clone();
        input["entries"] = json!([{"path":"fiction","kind":"folder"}]);
        input["folders"] = json!([]);
        for i in 0..=MAX_FINDINGS {
            let path = format!("fiction/f{i}");
            input["entries"]
                .as_array_mut()
                .unwrap()
                .push(json!({"path":path,"kind":"folder"}));
            input["folders"].as_array_mut().unwrap().push(json!(path));
        }
        assert_eq!(
            respond(&serde_json::to_vec(&input).unwrap())["reason"],
            "finding_limit"
        );
        assert_eq!(
            respond(&vec![b' '; MAX_REQUEST_BYTES + 1])["reason"],
            "request_limit"
        );
    }
}
