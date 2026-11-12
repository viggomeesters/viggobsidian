//! A sealed writer's exact change set, with bounded dossier/dependency coverage.
//! Legacy exceptions require explicit adoption and immutable historical bytes.
use crate::{
    Finding, SCHEMA, Scan, TILE_CSS, navigation_contract, tile_styles, without_css_comments,
};
use serde_json::{Value, json};
use std::{
    cell::OnceCell,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REQUEST: &str = "viggobsidian.write-set.v1";
const REPORT: &str = "viggobsidian.write-check.v1";

fn keys(value: &Value, allowed: &[&str]) -> Result<()> {
    let object = value.as_object().ok_or("expected an object")?;
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("unknown write-set field".into());
    }
    Ok(())
}

fn checked_path(vault: &Path, raw: &str) -> Result<PathBuf> {
    let path = Path::new(raw);
    if raw.contains('\\')
        || raw
            .split('/')
            .any(|p| p.is_empty() || (p.starts_with('.') && p != ".mail-source.json"))
        || path
            .components()
            .any(|p| !matches!(p, Component::Normal(_)))
        || path.components().count() < 2
    {
        return Err("write-set paths must be visible, relative, normalized files".into());
    }
    let mut current = vault.to_path_buf();
    for part in path.components() {
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.is_symlink() => return Err("symbolic link in write-set path".into()),
            Ok(meta) if current == vault.join(path) && !meta.is_file() => {
                return Err("write-set target is not a regular file".into());
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(path.to_path_buf())
}

fn findings(rows: &[Finding]) -> Vec<Value> {
    let mut rows: Vec<_> = rows
        .iter()
        .map(|f| json!({"path":f.path,"code":f.code,"message":f.message}))
        .collect();
    rows.sort_by_key(Value::to_string);
    rows.dedup();
    rows
}

pub(crate) fn run(vault: &Path, request: Value) -> Result<u8> {
    let (report, code) = check(vault, request)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(code)
}

fn check(vault: &Path, request: Value) -> Result<(Value, u8)> {
    keys(&request, &["schema", "phase", "changes", "baseline"])?;
    if request["schema"] != REQUEST {
        return Err("unknown write-set schema".into());
    }
    let phase = request["phase"].as_str().ok_or("write-set phase missing")?;
    if !matches!(phase, "baseline" | "validate") {
        return Err("unknown write-set phase".into());
    }
    let changes = request["changes"]
        .as_array()
        .ok_or("write-set changes missing")?;
    if changes.is_empty() || changes.len() > 10000 {
        return Err("write-set needs 1..10000 exact changed files".into());
    }
    let mut paths = BTreeMap::new();
    let mut proofs = BTreeMap::new();
    for change in changes {
        keys(change, &["path", "expect", "proof"])?;
        let path = checked_path(vault, change["path"].as_str().ok_or("missing change path")?)?;
        if let Some(proof) = change.get("proof") {
            let after = crate::maintenance::transform(path.to_str().ok_or("invalid path")?, proof)?;
            let expected = if phase == "baseline" {
                proof["before"].as_str().ok_or("missing before")?
            } else {
                &after
            };
            if fs::read_to_string(vault.join(&path))? != expected {
                return Err("compatibility proof does not match the installed bytes".into());
            }
            if change["expect"] != "present" {
                return Err("compatibility changes cannot delete notes".into());
            }
            let targets: Vec<_> = if proof["kind"] == "owner-handoff" {
                vec![proof["target"].as_str().ok_or("missing target")?]
            } else {
                proof["mapping"]
                    .as_object()
                    .ok_or("missing mapping")?
                    .values()
                    .map(|v| v.as_str().ok_or("invalid target"))
                    .collect::<std::result::Result<Vec<_>, _>>()?
            };
            for target in targets {
                let destination = checked_path(vault, target)?;
                if phase == "validate"
                    && (!vault.join(&destination).is_file()
                        || fs::read_to_string(vault.join(&destination))
                            .is_ok_and(|s| s.contains("<!-- viggobsidian:knowledge-owner -->")))
                {
                    return Err("missing target or knowledge handoff cycle".into());
                }
            }
            proofs.insert(path.clone(), proof.clone());
        }
        let expect = change["expect"]
            .as_str()
            .ok_or("missing expected file state")?;
        if !matches!(expect, "present" | "absent") || paths.insert(path, expect).is_some() {
            return Err("duplicate change path or invalid expected state".into());
        }
    }
    let changes: Vec<_> = paths
        .iter()
        .map(|(p, e)| match proofs.get(p) {
            Some(proof) => json!({"path":p,"expect":e,"proof":proof}),
            None => json!({"path":p,"expect":e}),
        })
        .collect();
    let baseline = &request["baseline"];
    let adoption = crate::adoption::load(vault)?;
    if phase == "baseline" && !baseline.is_null() {
        return Err("baseline phase cannot import a baseline".into());
    }
    let old_boundary: BTreeSet<String> = if phase == "validate" {
        keys(
            baseline,
            &[
                "schema",
                "phase",
                "changes",
                "owned_scopes",
                "boundary_scopes",
                "folders_checked",
                "owned_findings",
                "boundary_findings",
                "findings",
                "adoption",
                "legacy_findings",
                "preservation",
            ],
        )?;
        if baseline["schema"] != REPORT
            || baseline["phase"] != "baseline"
            || baseline["changes"] != json!(changes)
            || baseline["adoption"] != adoption
        {
            return Err("write-set baseline is not bound to the exact changes".into());
        }
        let rows = baseline["boundary_findings"]
            .as_array()
            .ok_or("baseline boundary findings missing")?;
        for row in rows {
            keys(row, &["path", "code", "message"])?;
            if ["path", "code", "message"]
                .iter()
                .any(|k| row[*k].as_str().is_none())
            {
                return Err("invalid baseline finding".into());
            }
        }
        rows.iter().map(Value::to_string).collect()
    } else {
        BTreeSet::new()
    };

    // A Raw/Wiki change owns its entire dossier: this includes inventory and
    // processed-source reverse references, rather than just the edited file.
    // Other changes own their direct folder; metadata cannot broaden a scope.
    let mut schema: Value = serde_saphyr::from_str(&fs::read_to_string(vault.join(SCHEMA))?)?;
    let mut owned = BTreeMap::new();
    let mut boundary = BTreeSet::new();
    for path in paths.keys() {
        let parent = path.parent().ok_or("change has no folder")?;
        owned.entry(parent.to_path_buf()).or_insert(false);
        if vault.join(parent).join("raw").is_dir() {
            owned.insert(parent.to_path_buf(), true);
        }
        if path.file_stem() == parent.file_name()
            && crate::strings(&schema["work_project_code_roots"])
                .any(|root| parent != Path::new(root) && parent.starts_with(root))
        {
            owned.insert(parent.to_path_buf(), true);
        }
        let mut prefix = PathBuf::new();
        for component in parent.components() {
            if matches!(component.as_os_str().to_str(), Some("raw" | "wiki"))
                && !prefix.as_os_str().is_empty()
            {
                owned.insert(prefix.clone(), true);
                break;
            }
            prefix.push(component);
        }
        for ancestor in parent.ancestors().filter(|p| !p.as_os_str().is_empty()) {
            boundary.insert(ancestor.to_path_buf());
        }
    }
    // Deleting Raw or moving an entrance cannot shrink preimage coverage.
    if phase == "validate" {
        for (path, recursive) in baseline["owned_scopes"]
            .as_object()
            .ok_or("baseline owners missing")?
        {
            let checked = checked_path(vault, &format!("{path}/owner.md"))?;
            let dir = checked.parent().unwrap().to_path_buf();
            let previous = recursive.as_bool().ok_or("invalid baseline owner depth")?;
            owned
                .entry(dir)
                .and_modify(|full| *full |= previous)
                .or_insert(previous);
        }
    }
    for proof in proofs.values().filter(|p| p["kind"] == "owner-handoff") {
        let target = Path::new(proof["target"].as_str().ok_or("handoff target absent")?);
        let parent = target.parent().ok_or("handoff parent absent")?;
        let dossier = if parent.ends_with("wiki") {
            parent.parent().ok_or("handoff dossier absent")?
        } else {
            parent
        };
        owned.insert(dossier.to_path_buf(), true);
    }
    let full: Vec<_> = owned
        .iter()
        .filter(|(_, full)| **full)
        .map(|(p, _)| p.clone())
        .collect();
    owned.retain(|p, _| !full.iter().any(|root| p != root && p.starts_with(root)));
    boundary.retain(|p| !owned.contains_key(p) && !full.iter().any(|root| p.starts_with(root)));
    // Only a fixed byte-preserving operation can retain old owner findings.
    // Every other member is fingerprinted, so concurrent sibling drift cannot
    // hide behind the existing errors. Ordinary write sets remain strict.
    let mut preservation = BTreeMap::new();
    for dir in owned.keys() {
        let members: Vec<_> = paths.keys().filter(|p| p.starts_with(dir)).collect();
        if members.iter().any(|p| proofs.contains_key(*p)) {
            let skip = members
                .iter()
                .map(|p| p.strip_prefix(dir).unwrap().to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            preservation.insert(
                dir.clone(),
                crate::maintenance::inventory(&vault.join(dir), &skip)?,
            );
        }
    }
    if phase == "validate"
        && baseline
            .get("preservation")
            .cloned()
            .unwrap_or_else(|| json!({}))
            != json!(preservation)
    {
        return Err("preserved owner inventory drift".into());
    }
    if !schema["validation_profiles"]["minimal"]["required"].is_array()
        || !schema["contracts"]["raw_package"]["required_fields"].is_array()
        || !schema["contracts"]["raw_package"]["wiki_status_values"].is_object()
        || !schema["common_fields"]["layer"]["allowed_values"].is_array()
    {
        return Err("invalid vault schema".into());
    }
    navigation_contract::apply(vault, &mut schema)?;
    if paths.keys().any(|p| p.starts_with("99-system")) {
        schema["system_layout"] = serde_saphyr::from_str(&fs::read_to_string(
            vault.join("99-system/contracts/system-layout.yaml"),
        )?)?;
    }
    let css = match fs::read_to_string(vault.join(TILE_CSS)) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let css = without_css_comments(&css);
    let mut scan = Scan {
        vault,
        schema: &schema,
        tile_styles: tile_styles(&css),
        findings: vec![],
        folders: 0,
        tiles: 0,
        shortcuts: 0,
        notes: OnceCell::new(),
        dossier_tables_checked: BTreeSet::new(),
        raw_summary_checked: false,
        strict: true,
    };
    // Stable project identities remain a global invariant. A registry error
    // never receives a baseline exemption, including pre-existing duplicates.
    if crate::strings(&schema["work_project_code_roots"])
        .any(|root| paths.keys().any(|p| p.starts_with(root)))
        && let Err(message) =
            crate::project_codes::validate_registry(vault, &schema["work_project_code_roots"])
    {
        scan.issue(vault, "project-code-registry", message);
    }
    if phase == "validate" {
        for (path, expect) in &paths {
            if vault.join(path).is_file() != (*expect == "present") {
                scan.issue(
                    &vault.join(path),
                    "write-set-state",
                    "changed file does not have its expected state",
                );
            }
        }
    }
    for (dir, recurse) in &owned {
        let dir = vault.join(dir);
        if dir.is_dir() {
            let under_raw = dir
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|n| n == "raw");
            scan.folder_with_depth(&dir, under_raw, *recurse)?;
        }
    }
    let owned_findings = findings(&scan.findings);
    scan.findings.clear();
    for dir in &boundary {
        let dir = vault.join(dir);
        if dir.is_dir() {
            let under_raw = dir
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|n| n == "raw");
            scan.folder_with_depth(&dir, under_raw, false)?;
        }
    }
    let boundary_findings = findings(&scan.findings);
    let old_owned: BTreeSet<_> = baseline["owned_findings"]
        .as_array()
        .into_iter()
        .flatten()
        .map(Value::to_string)
        .collect();
    let mut legacy_findings = Vec::new();
    let mut errors = Vec::new();
    for row in &owned_findings {
        let path = row["path"].as_str().ok_or("finding path missing")?;
        let exempt = crate::adoption::formatting(vault, row)?
            && (phase == "baseline" || old_owned.contains(&row.to_string()))
            && crate::adoption::retained(vault, &adoption, path)?;
        let preserved = (!paths.contains_key(Path::new(path))
            || proofs.contains_key(Path::new(path)))
            && preservation
                .keys()
                .any(|scope| Path::new(path).starts_with(scope))
            && old_owned.contains(&row.to_string());
        if exempt || preserved {
            legacy_findings.push(row.clone());
        } else {
            errors.push(row.clone());
        }
    }
    // Adopted strict owners cannot disappear during an ordinary content move.
    if phase == "validate" {
        for (owner, entry) in adoption["owners"].as_object().unwrap() {
            if entry["state"] == "strict"
                && paths.keys().any(|p| p.starts_with(owner))
                && !vault.join(owner).is_dir()
            {
                errors.push(json!({"path":owner,"code":"adoption-strict-owner","message":"strict owner relocation requires explicit maintenance adoption"}));
            }
        }
    }
    errors.extend(
        boundary_findings
            .iter()
            .filter(|row| !old_boundary.contains(&row.to_string()))
            .cloned(),
    );
    let code = if phase == "validate" && !errors.is_empty() {
        1
    } else {
        0
    };
    Ok((
        json!({"schema":REPORT,"phase":phase,"changes":changes,"owned_scopes":owned,"boundary_scopes":boundary,"folders_checked":scan.folders,"owned_findings":owned_findings,"boundary_findings":boundary_findings,"legacy_findings":legacy_findings,"adoption":adoption,"findings":errors,"preservation":preservation}),
        code,
    ))
}
