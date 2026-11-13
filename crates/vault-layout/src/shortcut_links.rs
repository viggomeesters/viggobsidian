use std::{
    cell::OnceCell,
    fs, io,
    path::{Path, PathBuf},
};
pub(crate) use wiki_layout_core::headings::{anchor_count, decode_percent, heading};

pub fn check(
    vault: &Path,
    inventory: &OnceCell<Vec<PathBuf>>,
    source: &Path,
    link: &str,
) -> io::Result<Option<&'static str>> {
    let (target, anchor) = link
        .split_once('#')
        .map_or((link, None), |(target, anchor)| (target, Some(anchor)));
    let note = match resolve_note(vault, inventory, source, target)? {
        Ok(note) => note,
        Err(code) => return Ok(Some(code)),
    };
    let Some(anchor) = anchor else {
        return Ok(None);
    };
    let contents = fs::read_to_string(note)?;
    Ok(match anchor_count(&contents, anchor) {
        0 => Some("broken-shortcut-anchor"),
        1 => None,
        _ => Some("ambiguous-shortcut-anchor"),
    })
}

pub(crate) fn resolve_note(
    vault: &Path,
    inventory: &OnceCell<Vec<PathBuf>>,
    source: &Path,
    target: &str,
) -> io::Result<Result<PathBuf, &'static str>> {
    let mut candidates = Vec::new();
    if target.is_empty() {
        candidates.push(source.to_path_buf());
    } else {
        let filename = if target.ends_with(".md") {
            target.to_owned()
        } else {
            format!("{target}.md")
        };
        for base in [vault, source.parent().unwrap_or(vault)] {
            let candidate = base.join(&filename);
            if let Ok(path) = candidate.canonicalize()
                && visible_note(vault, &path)
                && candidate
                    .ancestors()
                    .take_while(|part| *part != vault)
                    .all(|part| {
                        fs::symlink_metadata(part).is_ok_and(|metadata| !metadata.is_symlink())
                    })
                && !candidates.contains(&path)
            {
                candidates.push(path);
            }
        }
        if candidates.is_empty() && !Path::new(target).is_absolute() {
            if inventory.get().is_none() {
                let mut notes = Vec::new();
                collect_notes(vault, &mut notes)?;
                let _ = inventory.set(notes);
            }
            candidates.extend(
                inventory
                    .get()
                    .into_iter()
                    .flatten()
                    .filter(|path| path.ends_with(&filename))
                    .cloned(),
            );
        }
    }
    Ok(match candidates.pop() {
        None => Err("broken-shortcut-note"),
        Some(note) if candidates.is_empty() => Ok(note),
        Some(_) => Err("ambiguous-shortcut-note"),
    })
}

fn visible_note(vault: &Path, path: &Path) -> bool {
    path.strip_prefix(vault).is_ok_and(|relative| {
        relative
            .components()
            .all(|part| !part.as_os_str().to_string_lossy().starts_with('.'))
    }) && path.is_file()
}

fn collect_notes(folder: &Path, notes: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() {
            collect_notes(&path, notes)?;
        } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            notes.push(path);
        }
    }
    Ok(())
}
