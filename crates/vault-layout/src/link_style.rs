//! Native filesystem and optional fix adapter; rules live in link-style-core.
use link_style_core::{Note, edits};
use serde_json::json;
use std::{collections::BTreeMap, fs, io, path::Path};

fn inventory(root: &Path, dir: &Path, notes: &mut Vec<Note>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() {
            inventory(root, &path, notes)?;
        } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            let source = fs::read_to_string(&path)?;
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            let title = crate::index_navigation::visible_body(&source)
                .lines()
                .find_map(|line| line.strip_prefix("# "))
                .filter(|s| !s.contains("[["))
                .unwrap_or(&name)
                .to_owned();
            notes.push(Note {
                path: path.strip_prefix(root).unwrap().to_owned(),
                name,
                title,
            });
        }
    }
    Ok(())
}

pub(crate) fn run(
    root: &Path,
    apply: bool,
    json_output: bool,
) -> Result<u8, Box<dyn std::error::Error>> {
    let mut notes = Vec::new();
    inventory(root, root, &mut notes)?;
    notes.sort_by(|a, b| a.path.cmp(&b.path));
    let mut issues = Vec::new();
    let mut changed = BTreeMap::new();
    let mut checked = 0;
    for note in &notes {
        if note.path.starts_with("90-templates") {
            continue;
        }
        checked += 1;
        let path = root.join(&note.path);
        // Match the producers' OS file lock; reread and write only while held.
        let _daily_lock = if apply && note.path.parent() == Some(Path::new("10-daily/notes")) {
            let lock = fs::OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(root.join("10-daily/notes/.raycast-capture.lock"))?;
            lock.lock()?;
            Some(lock)
        } else {
            None
        };
        let source = fs::read_to_string(&path)?;
        let changes = edits(&source, &note.path, &notes);
        for edit in &changes {
            issues.push(json!({"path":note.path,"line":source[..edit.start].bytes().filter(|&b| b == b'\n').count()+1,"kind":edit.reason,"old":&source[edit.start..edit.end],"new":edit.replacement}));
        }
        let fixes: Vec<_> = changes
            .iter()
            .filter(|edit| edit.replacement != source[edit.start..edit.end])
            .collect();
        if apply && !fixes.is_empty() {
            let mut output = source.clone();
            for edit in fixes.iter().rev() {
                output.replace_range(edit.start..edit.end, &edit.replacement);
            }
            // Detect concurrent edits rather than replacing a newly changed note.
            if fs::read_to_string(&path)? != source {
                return Err(format!("concurrent edit: {}", note.path.display()).into());
            }
            fs::write(&path, output)?;
            changed.insert(note.path.to_string_lossy().into_owned(), fixes.len());
        }
    }
    let report = json!({"inventoried":notes.len(),"checked":checked,"skipped_templates":notes.len()-checked,"findings":issues.len(),"changed":changed,"issues":issues});
    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "Link style: {} notes checked, {} findings, {} files changed",
            checked,
            issues.len(),
            changed.len()
        );
        for issue in &issues {
            println!(
                "{}:{} {}: {}",
                issue["path"].as_str().unwrap(),
                issue["line"],
                issue["kind"].as_str().unwrap(),
                issue["old"].as_str().unwrap()
            );
        }
    }
    let unresolved = issues.iter().any(|issue| {
        issue["kind"] == "unresolved-long-link"
            || (issue["kind"] == "wiki-source-style-class" && issue["new"] == "")
    });
    Ok(u8::from((!apply && !issues.is_empty()) || unresolved))
}
