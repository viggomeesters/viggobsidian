//! Read-only authoring gate for Wiki changes made with any editor.
use crate::{
    direct_wiki_note, event_header, frontmatter,
    index_navigation::{visible_body, wiki_links},
    iso_date, raw_processing,
    shortcut_links::{decode_percent, heading},
    strings, text,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};
use wiki_layout_core::headings::{anchor_count, heading_text};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn git(vault: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args([
            "--no-replace-objects",
            "--literal-pathspecs",
            "-c",
            "core.fsmonitor=false",
            "-C",
        ])
        .arg(vault)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env_remove("GIT_NAMESPACE")
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

fn wiki_location(path: &Path) -> Option<(&Path, bool)> {
    if !visible_markdown(path) || path.components().any(|part| part.as_os_str() == "raw") {
        return None;
    }
    let parent = path.parent()?;
    match parent
        .ancestors()
        .find(|ancestor| ancestor.ends_with("wiki"))
        .and_then(Path::parent)
    {
        Some(dossier) => Some((dossier, false)),
        None if direct_wiki_note(parent).as_deref() == Some(path) => Some((parent, true)),
        None => None,
    }
}

fn maintained(path: &Path, note: Option<&str>, direct: bool) -> bool {
    path.ends_with("wiki/wiki.md") || is_wiki(note, direct)
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

#[derive(Default)]
struct MarkdownVisibility {
    fence: Option<(char, usize, usize)>,
    comment: bool,
}

impl MarkdownVisibility {
    fn line(&mut self, line: &str) -> String {
        let content = link_style_core::quote_content(line);
        let quote_depth = line[..line.len() - content.len()].matches('>').count();
        let trimmed = content.trim_start_matches(' ');
        let marker = (content.len() - trimmed.len() <= 3)
            .then(|| trimmed.chars().next())
            .flatten()
            .filter(|ch| matches!(ch, '`' | '~'));
        let count = marker.map_or(0, |marker| {
            trimmed.chars().take_while(|ch| *ch == marker).count()
        });
        if let Some((opening, length, depth)) = self.fence {
            if quote_depth >= depth {
                if marker == Some(opening) && count >= length && trimmed[count..].trim().is_empty()
                {
                    self.fence = None;
                }
                return String::new();
            }
            self.fence = None;
        }
        if !self.comment && count >= 3 {
            self.fence = marker.map(|marker| (marker, count, quote_depth));
            return String::new();
        }
        let mut result = String::new();
        let mut rest = line;
        loop {
            if self.comment {
                let Some((_, tail)) = rest.split_once("-->") else {
                    break;
                };
                self.comment = false;
                rest = tail;
            } else if let Some((start, end)) = code_span_before_comment(rest) {
                result.push_str(&rest[..end]);
                rest = &rest[end..];
                debug_assert!(start < end);
            } else if let Some((pre, tail)) = rest.split_once("<!--") {
                result.push_str(pre);
                self.comment = true;
                rest = tail;
            } else {
                result.push_str(rest);
                break;
            }
        }
        result
    }
}

fn code_span_before_comment(line: &str) -> Option<(usize, usize)> {
    let comment = line.find("<!--")?;
    let mut offset = 0;
    while let Some(open) = line[offset..comment].find('`') {
        let start = offset + open;
        let length = line[start..].bytes().take_while(|&b| b == b'`').count();
        offset = start + length;
        let mut search = offset;
        while let Some(close) = line[search..].find('`') {
            let end = search + close;
            let closing = line[end..].bytes().take_while(|&b| b == b'`').count();
            search = end + closing;
            if closing == length {
                return Some((start, search));
            }
        }
    }
    None
}

fn sections(wiki: &str) -> Value {
    let mut stack = Vec::<(usize, String)>::new();
    let mut sections = BTreeMap::<String, (usize, String, BTreeSet<String>)>::new();
    let mut key = String::new();
    let mut previous: Option<String> = None;
    let mut visibility = MarkdownVisibility::default();
    let mut metadata = wiki.starts_with("---\n") || wiki.starts_with("---\r\n");
    for (position, original) in wiki.split_inclusive('\n').enumerate() {
        if metadata {
            sections
                .entry(String::new())
                .or_default()
                .1
                .push_str(original);
            metadata = position == 0 || original.trim_end_matches(['\r', '\n']) != "---";
            continue;
        }
        let line = visibility.line(original.trim_end_matches(['\r', '\n']));
        let trimmed = line.trim_start_matches(' ');
        if line.len() - trimmed.len() <= 3
            && let Some((level, title)) = heading(trimmed, previous.as_deref())
        {
            stack.retain(|(old, _)| *old < level);
            stack.push((level, heading_text(title)));
            key = stack
                .iter()
                .map(|(_, title)| title.as_str())
                .collect::<Vec<_>>()
                .join("\0");
            previous = None;
        } else {
            previous = (line.len() - trimmed.len() <= 3
                && !trimmed.is_empty()
                && !trimmed.starts_with(['>', '\t']))
            .then(|| trimmed.to_owned());
        }
        let (_, body, links) = sections
            .entry(key.clone())
            .or_insert_with(|| (position, String::new(), BTreeSet::new()));
        body.push_str(original);
        if line.len() - trimmed.len() <= 3 && !trimmed.starts_with(['>', '\t']) {
            let visible_line = visible_body(&line);
            for link in wiki_links(&visible_line) {
                if visible_line
                    .match_indices(&format!("[[{link}]]"))
                    .any(|(i, _)| !visible_line[..i].ends_with(['!', '\\']))
                {
                    links.insert(link.split('|').next().unwrap_or(link).to_owned());
                }
            }
        }
    }
    json!(
        sections
            .into_iter()
            .map(|(key, (position, body, links))| {
                (key, json!({"hash":sha256(body.trim_end_matches(['\r','\n'])),"links":links,"position":position}))
            })
            .collect::<BTreeMap<_, _>>()
    )
}

fn section_key(sections: &Value, anchor: &str) -> Option<String> {
    let parts = anchor
        .split('#')
        .map(|part| decode_percent(part).trim().to_owned())
        .collect::<Vec<_>>();
    let keys = sections.as_object()?;
    let mut matches = keys.keys().filter(|key| {
        let titles = key.split('\0').collect::<Vec<_>>();
        titles.ends_with(&parts.iter().map(String::as_str).collect::<Vec<_>>())
    });
    let key = matches.next()?.clone();
    matches.next().is_none().then_some(key)
}

fn covers(section: &str, target: &str) -> bool {
    section == target
        || section
            .strip_prefix(target)
            .is_some_and(|tail| tail.starts_with('\0'))
        || (section.is_empty() && !target.contains('\0'))
}

fn cites_event(sections: &Value, target: &str, event: &Path, event_text: Option<&str>) -> bool {
    let path = event.with_extension("");
    sections.as_object().is_some_and(|sections| {
        sections.iter().any(|(key, section)| {
            covers(key, target)
                && section["links"].as_array().is_some_and(|links| {
                    links.iter().filter_map(Value::as_str).any(|link| {
                        let (note, anchor) = link
                            .split_once('#')
                            .map_or((link, None), |(note, anchor)| (note, Some(anchor)));
                        Path::new(note.strip_suffix(".md").unwrap_or(note)) == path
                            && anchor.is_none_or(|heading| {
                                event_text.is_none_or(|source| anchor_count(source, heading) == 1)
                            })
                    })
                })
        })
    })
}

fn has_event(
    vault: &Path,
    raw: &Path,
    wiki_path: &Path,
    before: &Value,
    wiki: &str,
    new_paths: &BTreeSet<&Path>,
    fresh: impl Fn(&Path, &str) -> Result<bool>,
) -> Result<bool> {
    let after = sections(wiki);
    let empty = serde_json::Map::new();
    let prior = before["sections"].as_object().unwrap_or(&empty);
    let next = after.as_object().ok_or("invalid Wiki section state")?;
    let mut changed: BTreeSet<_> = prior
        .keys()
        .chain(next.keys())
        .filter(|key| {
            // Blank separation between sections is not a separate revision.
            prior.get(*key).map(|entry| &entry["hash"])
                != next.get(*key).map(|entry| &entry["hash"])
        })
        .collect();
    let mut old_order = prior
        .keys()
        .filter(|key| next.contains_key(*key))
        .collect::<Vec<_>>();
    let mut new_order = old_order.clone();
    old_order.sort_by_key(|key| prior[*key]["position"].as_u64());
    new_order.sort_by_key(|key| next[*key]["position"].as_u64());
    for (old, new) in old_order.into_iter().zip(new_order) {
        if old != new {
            changed.extend([old, new]);
        }
    }
    let after_hash = sha256(wiki);
    let mut covered = BTreeSet::new();
    for event in new_paths.range(raw..) {
        if !event.starts_with(raw) {
            break;
        }
        let Some(package) = event_package(raw, event) else {
            continue;
        };
        let Some(event_text) = working_note(vault, event)? else {
            continue;
        };
        let Some((yaml, body)) = frontmatter(&event_text) else {
            continue;
        };
        let options = serde_saphyr::options! {
            duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
            merge_keys: serde_saphyr::MergeKeyPolicy::Error
        };
        let Ok(data): std::result::Result<Value, _> =
            serde_saphyr::from_str_with_options(&yaml, options)
        else {
            continue;
        };
        let Some((note, anchor)) = text(&data, "wiki_target").and_then(raw_processing::target)
        else {
            continue;
        };
        let Some(target) = section_key(&after, anchor) else {
            continue;
        };
        let note = Path::new(note.strip_suffix(".md").unwrap_or(note));
        let proof = json!({
            "schema":"viggobsidian.wiki-revision.v1",
            "before_sha256":before["sha256"],
            "after_sha256":after_hash
        });
        if text(&data, "wiki_status") == Some("processed")
            && text(&data, "wiki_processed_on").is_some_and(iso_date)
            && text(&data, "type").is_some_and(|kind| !kind.is_empty())
            && text(&data, "category").is_some_and(|kind| !kind.is_empty())
            && note == wiki_path.with_extension("")
            && data["wiki_revision"] == proof
            && event_header::check(vault, &vault.join(package), &data, &body).is_ok()
            && raw_processing::check_event(&body, &note.to_string_lossy(), anchor, wiki).is_ok()
            && cites_event(&after, &target, event, Some(&event_text))
            && !prior
                .keys()
                .any(|key| cites_event(&before["sections"], key, event, None))
            && fresh(event, &event_text)?
        {
            let package = vault.join(package);
            let (_, archive_findings) = crate::native_source_archives::check(&package, &data)?;
            if !archive_findings.is_empty()
                || !crate::source_inventory::check(vault, &package, &data, &body)?.is_empty()
            {
                continue;
            }
            covered.extend(changed.iter().filter(|key| covers(key, &target)).copied());
        }
    }
    Ok(!covered.is_empty() && covered == changed)
}

fn finding(path: &Path) -> Value {
    json!({
        "path": path,
        "code": "wiki-revision-missing-raw-event",
        "message": "keep the Wiki and cite a new processed Event Note from this dossier's Raw under the exact heading it targets"
    })
}

fn sha256(note: &str) -> String {
    format!("{:x}", Sha256::digest(note.as_bytes()))
}

/// Record only hashes for possible Wiki paths and prior presence for event
/// candidates. The writer already seals the report and its full change set.
pub(crate) fn checkpoint(vault: &Path, paths: &BTreeMap<PathBuf, &str>) -> Result<Value> {
    let mut before = BTreeMap::new();
    for path in paths.keys() {
        if let Some((_, direct)) = wiki_location(path) {
            let note = working_note(vault, path)?;
            before.insert(
                path,
                json!({
                    "wiki": maintained(path, note.as_deref(), direct),
                    "sha256": note.as_deref().map(sha256),
                    "sections": note.as_deref().map(sections)
                }),
            );
        } else if visible_markdown(path)
            && let Some(raw) = path.parent().and_then(Path::parent)
            && raw.ends_with("raw")
            && event_package(raw, path).is_some()
        {
            let observed = vault.join(path).is_file()
                || (vault.join(".git").exists()
                    && !git(
                        vault,
                        &[
                            "log",
                            "-1",
                            "--format=%H",
                            "--all",
                            "--",
                            path.to_str().ok_or("non-UTF-8 Event path")?,
                        ],
                    )?
                    .is_empty());
            before.insert(path, json!({"event_existed": observed}));
        }
    }
    Ok(json!(before))
}

pub(crate) fn validate_write_set(
    vault: &Path,
    paths: &BTreeMap<PathBuf, &str>,
    baseline: &Value,
    current: &Value,
    post_hashes: &BTreeMap<PathBuf, &str>,
) -> Result<Vec<Value>> {
    let before = baseline
        .as_object()
        .ok_or("missing Wiki revision baseline")?;
    let now = current
        .as_object()
        .ok_or("missing current Wiki revision state")?;
    if !before.keys().eq(now.keys()) {
        return Err("Wiki revision baseline does not cover the exact change paths".into());
    }
    let new_events = paths
        .iter()
        .filter(|(path, expect)| {
            let key = path.to_string_lossy();
            **expect == "present"
                && post_hashes.contains_key(path.as_path())
                && before
                    .get(key.as_ref())
                    .is_some_and(|entry| entry["event_existed"] == false)
                && now
                    .get(key.as_ref())
                    .is_some_and(|entry| entry["event_existed"] == true)
        })
        .map(|(path, _)| path.as_path())
        .collect();
    let mut findings = Vec::new();
    for path in paths.keys() {
        let Some((dossier, _)) = wiki_location(path) else {
            continue;
        };
        let key = path.to_string_lossy();
        let old = &before[key.as_ref()];
        let new = &now[key.as_ref()];
        let old_wiki = old["wiki"]
            .as_bool()
            .ok_or("invalid Wiki revision baseline")?;
        if !(old_wiki || new["wiki"] == true) || old["sha256"] == new["sha256"] {
            continue;
        }
        let wiki = working_note(vault, path)?;
        if new["sha256"] != json!(wiki.as_deref().map(sha256)) {
            return Err(format!("Wiki changed during validation: {}", path.display()).into());
        }
        if let Some(wiki) = wiki
            && has_event(
                vault,
                &dossier.join("raw"),
                path,
                old,
                &wiki,
                &new_events,
                |event, note| {
                    Ok(post_hashes
                        .get(event)
                        .is_some_and(|hash| *hash == sha256(note)))
                },
            )?
        {
            continue;
        }
        findings.push(finding(path));
    }
    Ok(findings)
}

fn committed_note(vault: &Path, commit: &str, path: &Path) -> Result<Option<String>> {
    let path = path.to_str().ok_or("non-UTF-8 Wiki audit path")?;
    if git(vault, &["ls-tree", "--name-only", commit, "--", path])?.is_empty() {
        return Ok(None);
    }
    git(vault, &["show", &format!("{commit}:{path}")]).map(Some)
}

fn event_history(
    vault: &Path,
    base: &str,
    head: &str,
    wiki_path: &Path,
    wiki: &str,
    event: &Path,
    event_text: &str,
) -> Result<bool> {
    let range = format!("{base}..{head}");
    let event_path = event.to_str().ok_or("non-UTF-8 Raw path")?;
    let history = git(
        vault,
        &[
            "log",
            "--full-history",
            "--format=%H %P",
            head,
            "--",
            event_path,
        ],
    )?;
    let head_wiki = committed_note(vault, head, wiki_path)?;
    let base_wiki = committed_note(vault, base, wiki_path)?;
    let current_is_committed = head_wiki.as_deref() == Some(wiki);
    if !current_is_committed && head_wiki != base_wiki {
        return Ok(false);
    }
    if history.is_empty() {
        return Ok(!current_is_committed);
    }
    if committed_note(vault, head, event)?.as_deref() != Some(event_text) {
        return Ok(false);
    }
    let mut introductions = 0;
    for line in history.lines() {
        let mut words = line.split_ascii_whitespace();
        let commit = words.next().ok_or("invalid Raw history commit")?;
        if committed_note(vault, commit, event)?.as_deref() != Some(event_text) {
            return Ok(false);
        }
        let mut carried = false;
        for parent in words {
            if let Some(parent_event) = committed_note(vault, parent, event)? {
                if parent_event != event_text {
                    return Ok(false);
                }
                carried = true;
            }
        }
        introductions += usize::from(!carried);
    }
    if introductions != 1 {
        return Ok(false);
    }
    let path = wiki_path.to_str().ok_or("non-UTF-8 Wiki path")?;
    let changes = git(
        vault,
        &[
            "log",
            "--full-history",
            "--format=%H %P",
            &range,
            "--",
            path,
        ],
    )?;
    let mut used = 0;
    for line in changes.lines() {
        let mut words = line.split_ascii_whitespace();
        let commit = words.next().ok_or("invalid Wiki history commit")?;
        if committed_note(vault, commit, wiki_path)?.as_deref() == Some(wiki)
            && committed_note(vault, commit, event)?.is_some()
        {
            let mut carry = false;
            let mut matching_preimage = false;
            for parent in words {
                let before = committed_note(vault, parent, wiki_path)?;
                carry |= before.as_deref() == Some(wiki)
                    && committed_note(vault, parent, event)?.is_some();
                matching_preimage |= before == base_wiki;
            }
            if !carry {
                if !matching_preimage {
                    return Ok(false);
                }
                used += 1;
            }
        }
    }
    Ok(used == usize::from(current_is_committed))
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
    let head = git(vault, &["rev-parse", "--verify", "HEAD"])?;
    let head = head.trim();
    git(vault, &["merge-base", "--is-ancestor", baseline, head])?;
    if !git(vault, &["ls-files", "--unmerged", "-z"])?.is_empty() {
        return Err("resolve unmerged files before auditing Wiki revisions".into());
    }
    let index = git(vault, &["ls-files", "-v", "-z"])?;
    if index.split('\0').any(|entry| {
        entry.split_once(' ').is_some_and(|(flag, name)| {
            (flag == "S" || flag.bytes().any(|value| value.is_ascii_lowercase()))
                && visible_markdown(Path::new(name))
        })
    }) {
        return Err("Wiki audit requires visible Git index entries; remove assume-unchanged/skip-worktree first".into());
    }
    let tree = git(vault, &["ls-tree", "-r", "--name-only", "-z", baseline])?;
    let known: BTreeSet<_> = paths(&tree).collect();
    let tracked = git(vault, &["ls-files", "--cached", "-z"])?;
    let untracked = git(vault, &["ls-files", "--others", "--exclude-standard", "-z"])?;
    let changes: BTreeSet<_> = paths(&tree)
        .chain(paths(&tracked))
        .chain(paths(&untracked))
        .filter(|p| visible_markdown(p))
        .collect();
    let new_paths = changes.difference(&known).copied().collect();
    let mut checked = 0;
    let mut findings = Vec::new();
    for path in &changes {
        let Some((dossier, direct)) = wiki_location(path) else {
            continue;
        };
        let raw = dossier.join("raw");
        let before = known
            .contains(path)
            .then(|| git(vault, &["show", &format!("{baseline}:{}", path.display())]))
            .transpose()?;
        let current = working_note(vault, path)?;
        if !maintained(path, before.as_deref(), direct)
            && !maintained(path, current.as_deref(), direct)
        {
            continue;
        }
        if before == current {
            continue;
        }
        checked += 1;
        let prior = json!({"sha256":before.as_deref().map(sha256), "sections":before.as_deref().map(sections)});
        if let Some(wiki) = current
            && has_event(
                vault,
                &raw,
                path,
                &prior,
                &wiki,
                &new_paths,
                |event, source| event_history(vault, baseline, head, path, &wiki, event, source),
            )?
        {
            continue;
        }
        findings.push(finding(path));
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
