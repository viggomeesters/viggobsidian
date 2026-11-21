//! Read-only authoring gate for Wiki changes made with any editor.
use crate::{
    direct_wiki_note, event_header, frontmatter,
    index_navigation::{visible_body, wiki_links},
    iso_date, raw_processing,
    shortcut_links::{decode_percent, heading},
    strings, text,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, io, path::Path, process::Command};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn git(vault: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(["--no-replace-objects", "-C"])
        .arg(vault)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "git {}: {}",
            args[0],
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn paths(list: &str) -> impl Iterator<Item = &Path> {
    list.split('\0')
        .filter(|name| !name.is_empty())
        .map(Path::new)
}

fn visible_markdown(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "md")
        && path.components().all(|part| {
            part.as_os_str()
                .to_str()
                .is_some_and(|s| !s.starts_with('.'))
        })
}

fn working_note(vault: &Path, path: &Path) -> Result<Option<String>> {
    let mut target = vault.to_path_buf();
    for component in path.components() {
        target.push(component);
        match fs::symlink_metadata(&target) {
            Ok(meta) if meta.is_symlink() => {
                return Err(format!("symbolic link in Wiki audit path: {}", path.display()).into());
            }
            Ok(meta) if target == vault.join(path) && !meta.is_file() => {
                return Err(
                    format!("Wiki audit path is not a regular file: {}", path.display()).into(),
                );
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(Some(fs::read_to_string(target)?))
}

fn is_wiki(note: Option<&str>, direct: bool) -> bool {
    note.and_then(frontmatter)
        .and_then(|(yaml, _)| serde_saphyr::from_str::<Value>(&yaml).ok())
        .is_some_and(|data| {
            text(&data, "layer") == Some("wiki")
                && (!direct
                    || strings(&data["cssclasses"]).any(|class| class == "vault-wiki-direct"))
        })
}

fn event_package<'a>(raw: &Path, path: &'a Path) -> Option<&'a Path> {
    let package = path.parent()?;
    let name = package.file_name()?.to_str()?;
    let (date, slug) = name.split_once('-')?;
    if package.parent()? != raw
        || path.file_stem()?.to_str()? != name
        || date.len() != 8
        || !date.bytes().all(|b| b.is_ascii_digit())
        || !iso_date(&format!("{}-{}-{}", &date[..4], &date[4..6], &date[6..]))
        || slug.is_empty()
    {
        return None;
    }
    Some(package)
}

fn cites_event(wiki: &str, target: &str, event: &Path) -> bool {
    let wanted: Vec<_> = target
        .split('#')
        .map(|part| decode_percent(part).trim().to_lowercase())
        .collect();
    let target_path = event.with_extension("");
    let mut stack = Vec::<(usize, String)>::new();
    let mut previous = None;
    let visible = visible_body(wiki);
    for line in visible.lines() {
        let trimmed = line.trim_start_matches(' ');
        if line.len() - trimmed.len() <= 3
            && let Some((level, title)) = heading(trimmed, previous)
        {
            stack.retain(|(old, _)| *old < level);
            stack.push((level, title.trim().to_lowercase()));
            previous = None;
            continue;
        }
        previous = (!trimmed.is_empty() && !trimmed.starts_with('>')).then_some(trimmed);
        if !trimmed.starts_with('>')
            && stack.len() >= wanted.len()
            && stack[stack.len() - wanted.len()..]
                .iter()
                .map(|(_, title)| title)
                .eq(&wanted)
            && wiki_links(line).any(|link| {
                let path = link.split(['|', '#']).next().unwrap_or("");
                let path = path.strip_suffix(".md").unwrap_or(path);
                Path::new(path) == target_path
                    && line
                        .match_indices(&format!("[[{link}]]"))
                        .any(|(i, _)| !line[..i].ends_with(['!', '\\']))
            })
        {
            return true;
        }
    }
    false
}

fn has_event(
    vault: &Path,
    raw: &Path,
    wiki_path: &Path,
    before: Option<&str>,
    wiki: &str,
    new_paths: &BTreeSet<&Path>,
) -> Result<bool> {
    for event in new_paths.range(raw..) {
        if !event.starts_with(raw) {
            break;
        }
        let Some(package) = event_package(raw, event) else {
            continue;
        };
        let Some((yaml, body)) = working_note(vault, event)?.as_deref().and_then(frontmatter)
        else {
            continue;
        };
        let Ok(data) = serde_saphyr::from_str::<Value>(&yaml) else {
            continue;
        };
        let Some((note, anchor)) = text(&data, "wiki_target").and_then(raw_processing::target)
        else {
            continue;
        };
        let note = Path::new(note.strip_suffix(".md").unwrap_or(note));
        if text(&data, "wiki_status") == Some("processed")
            && text(&data, "wiki_processed_on").is_some_and(iso_date)
            && text(&data, "type").is_some_and(|kind| !kind.is_empty())
            && text(&data, "category").is_some_and(|kind| !kind.is_empty())
            && note == wiki_path.with_extension("")
            && event_header::check(vault, &vault.join(package), &data, &body).is_ok()
            && raw_processing::check_event(&body, &note.to_string_lossy(), anchor, wiki).is_ok()
            && cites_event(wiki, anchor, event)
            && !before.is_some_and(|before| cites_event(before, anchor, event))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn run(vault: &Path, base: &str, json_output: bool) -> Result<u8> {
    let root = git(vault, &["rev-parse", "--show-toplevel"])?;
    if Path::new(root.trim_end_matches('\n')).canonicalize()? != vault {
        return Err("--vault must be the Git working-tree root".into());
    }
    let baseline = git(
        vault,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{base}^{{commit}}"),
        ],
    )?;
    let baseline = baseline.trim();
    git(vault, &["merge-base", "--is-ancestor", baseline, "HEAD"])?;
    if !git(vault, &["ls-files", "--unmerged", "-z"])?.is_empty() {
        return Err("resolve unmerged files before auditing Wiki revisions".into());
    }
    let tree = git(vault, &["ls-tree", "-r", "--name-only", "-z", baseline])?;
    let known: BTreeSet<_> = paths(&tree).collect();
    let diff = git(
        vault,
        &[
            "diff",
            "--no-ext-diff",
            "--no-renames",
            "--name-only",
            "-z",
            baseline,
            "--",
        ],
    )?;
    let untracked = git(vault, &["ls-files", "--others", "--exclude-standard", "-z"])?;
    let changes: BTreeSet<_> = paths(&diff)
        .chain(paths(&untracked))
        .filter(|p| visible_markdown(p))
        .collect();
    let new_paths = changes.difference(&known).copied().collect();
    let mut checked = 0;
    let mut findings = Vec::new();
    for path in &changes {
        let Some(parent) = path.parent() else {
            continue;
        };
        let (dossier, direct) = match parent
            .ancestors()
            .find(|ancestor| ancestor.ends_with("wiki"))
            .and_then(Path::parent)
        {
            Some(dossier) => (dossier, false),
            None if direct_wiki_note(parent).as_deref() == Some(*path) => (parent, true),
            None => continue,
        };
        let raw = dossier.join("raw");
        let before = known
            .contains(path)
            .then(|| git(vault, &["show", &format!("{baseline}:{}", path.display())]))
            .transpose()?;
        let current = working_note(vault, path)?;
        if !path.ends_with("wiki/wiki.md")
            && !is_wiki(before.as_deref(), direct)
            && !is_wiki(current.as_deref(), direct)
        {
            continue;
        }
        if before == current {
            continue;
        }
        checked += 1;
        if let Some(wiki) = current
            && has_event(vault, &raw, path, before.as_deref(), &wiki, &new_paths)?
        {
            continue;
        }
        findings.push(json!({
            "path": path,
            "code": "wiki-revision-missing-raw-event",
            "message": "keep the Wiki and cite a new processed Event Note from this dossier's Raw under the exact heading it targets"
        }));
    }
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "base": baseline, "wiki_checked": checked, "findings": findings
            }))?
        );
    } else {
        println!(
            "{checked} Wiki revisions checked against {baseline}, {} findings",
            findings.len()
        );
        for finding in &findings {
            println!(
                "{} [{}] {}",
                finding["path"], finding["code"], finding["message"]
            );
        }
    }
    Ok(u8::from(!findings.is_empty()))
}
