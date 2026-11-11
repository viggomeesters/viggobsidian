//! Work-only link presentation gate. Never rewrites preserved source bodies.
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct Edit {
    start: usize,
    end: usize,
    replacement: String,
    reason: &'static str,
}

struct Note {
    path: PathBuf,
    name: String,
    title: String,
}

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

fn resolve<'a>(notes: &'a [Note], file: &Path, target: &str) -> Option<&'a Note> {
    let target = target.strip_suffix(".md").unwrap_or(target);
    let parent = file.parent().unwrap_or(Path::new(""));
    let relative = parent.join(format!("{target}.md"));
    let exact = format!("{target}.md");
    let mut matches: Vec<_> = notes
        .iter()
        .filter(|note| {
            if target.contains('/') {
                note.path == Path::new(&exact) || note.path == relative
            } else {
                note.name == target
            }
        })
        .collect();
    if matches.is_empty() && target.contains('/') {
        matches = notes
            .iter()
            .filter(|note| note.path.ends_with(&exact))
            .collect();
    }
    (matches.len() == 1).then(|| matches[0])
}

fn compact_title(title: &str) -> String {
    let title = title.replace(['[', ']', '|', '\n', '\r'], " ");
    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
    if title.chars().count() <= 64 {
        return title;
    }
    let prefix: String = title.chars().take(61).collect();
    let prefix = prefix
        .rsplit_once(' ')
        .map_or(prefix.as_str(), |(head, _)| head);
    format!("{}…", prefix.trim_end())
}

fn in_component(path: &Path, component: &str) -> bool {
    path.components().any(|part| part.as_os_str() == component)
}

fn path_like(label: &str, target: &str) -> bool {
    label.starts_with('/')
        || label.contains('\\')
        || label.matches('/').count() >= 2
        || (label.contains('/') && label == target)
}

fn frontmatter(source: &str) -> Option<(&str, usize, &str)> {
    let newline = if source.starts_with("---\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let opening = format!("---{newline}");
    let closing = format!("{newline}---{newline}");
    let rest = source.strip_prefix(&opening)?;
    let (metadata, _) = rest.split_once(&closing)?;
    Some((metadata, opening.len() + metadata.len(), newline))
}

/// Remove excluded syntax without moving byte offsets. Unicode is masked byte-wise.
fn mask(source: &str) -> Vec<u8> {
    let bytes = source.as_bytes();
    let mut out = bytes.to_vec();
    let mut fence: Option<(u8, usize)> = None;
    let mut comment: Option<&str> = None;
    let mut frontmatter = source.starts_with("---\n") || source.starts_with("---\r\n");
    let mut offset = 0;
    for (index, line) in source.split_inclusive('\n').enumerate() {
        let trimmed = line.trim_start_matches(' ');
        let indent = line.len() - trimmed.len();
        let first = trimmed.as_bytes().first().copied().unwrap_or(0);
        let count = trimmed.bytes().take_while(|&b| b == first).count();
        let marker = indent <= 3 && matches!(first, b'`' | b'~') && count >= 3;
        let excluded = if frontmatter {
            if index > 0 && line.trim_end() == "---" {
                frontmatter = false;
            }
            true
        } else if let Some((opening, length)) = fence {
            if marker && first == opening && count >= length && trimmed[count..].trim().is_empty() {
                fence = None;
            }
            true
        } else if marker && comment.is_none() {
            fence = Some((first, count));
            true
        } else {
            indent >= 4 || line.starts_with('\t')
        };
        if excluded {
            out[offset..offset + line.len()].fill(b' ');
            offset += line.len();
            continue;
        }
        let mut i = offset;
        let end = offset + line.len();
        while i < end {
            if let Some(closing) = comment {
                if bytes[i..].starts_with(closing.as_bytes()) {
                    out[i..i + closing.len()].fill(b' ');
                    i += closing.len();
                    comment = None;
                } else {
                    out[i] = b' ';
                    i += 1;
                }
            } else if bytes[i..].starts_with(b"<!--") || bytes[i..].starts_with(b"%%") {
                let (length, closing) = if bytes[i..].starts_with(b"<!--") {
                    (4, "-->")
                } else {
                    (2, "%%")
                };
                out[i..i + length].fill(b' ');
                i += length;
                comment = Some(closing);
            } else if bytes[i] == b'`' {
                let length = bytes[i..end].iter().take_while(|&&b| b == b'`').count();
                let marker = "`".repeat(length);
                if let Some(close) = source[i + length..end].find(&marker) {
                    let next = i + length + close + length;
                    out[i..next].fill(b' ');
                    i = next;
                } else {
                    i += length;
                }
            } else {
                i += 1;
            }
        }
        offset = end;
    }
    out
}

fn edits(source: &str, file: &Path, notes: &[Note]) -> Vec<Edit> {
    let mut masked = mask(source);
    // Raw and inbox bodies are immutable evidence; only header navigation is maintained.
    let preserved = in_component(file, "raw") || file.starts_with("00-inbox");
    let mut in_navigation = false;
    let mut processing = false;
    let mut processing_agent = false;
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            processing = trimmed.trim_end() == "### Verwerking in Wiki";
            processing_agent = false;
        }
        if processing && trimmed.starts_with("> [!activity-message|") {
            processing_agent = trimmed.starts_with("> [!activity-message|agent]");
        } else if !trimmed.starts_with('>') && !trimmed.trim().is_empty() {
            processing_agent = false;
        }
        if trimmed.starts_with("> [!navigation") {
            in_navigation = true;
        } else if !trimmed.starts_with('>') {
            in_navigation = false;
        }
        if (preserved || trimmed.starts_with('>'))
            && !in_navigation
            && !(processing && processing_agent)
        {
            masked[offset..offset + line.len()].fill(b' ');
        }
        offset += line.len();
    }
    let mut result = Vec::new();
    // Leaf knowledge notes need the same existing source-link CSS as Wiki hubs.
    if in_component(file, "wiki")
        && let Some((metadata, end, newline)) = frontmatter(source)
        && let Ok(data) = serde_saphyr::from_str::<serde_json::Value>(metadata)
        && data["layer"] == "wiki"
        && !data["cssclasses"]
            .as_array()
            .is_some_and(|classes| classes.iter().any(|c| c == "vault-wiki"))
    {
        let addition = if data.get("cssclasses").is_none() {
            Some(format!("{newline}cssclasses: [vault-wiki]"))
        } else {
            None
        };
        result.push(Edit {
            start: end,
            end,
            replacement: addition.unwrap_or_default(),
            reason: "wiki-source-style-class",
        });
    }
    let mut i = 0;
    while i + 1 < masked.len() {
        if &masked[i..i + 2] != b"[[" {
            i += 1;
            continue;
        }
        let start = i;
        let Some(end) = masked[i + 2..].windows(2).position(|pair| pair == b"]]") else {
            break;
        };
        let end = i + 2 + end + 2;
        i = end;
        if start > 0 && matches!(source.as_bytes()[start - 1], b'!' | b'\\') {
            continue;
        }
        let raw = &source[start + 2..end - 2];
        if raw.contains(['\n', '[', ']']) {
            continue;
        }
        let (destination, alias) = raw.split_once('|').map_or((raw, None), |(to, label)| {
            (to.trim_end_matches('\\'), Some(label))
        });
        let (target, anchor) = destination
            .split_once('#')
            .map_or((destination, ""), |(target, _)| {
                (target, &destination[target.len()..])
            });
        if target.is_empty() {
            continue;
        }
        let Some(note) = resolve(notes, file, target) else {
            if (target.contains('/') && alias.is_none())
                || alias.is_some_and(|s| path_like(s, target))
            {
                result.push(Edit {
                    start,
                    end,
                    replacement: source[start..end].to_owned(),
                    reason: "unresolved-long-link",
                });
            }
            continue;
        };
        let navigation = source[..start]
            .rsplit('\n')
            .next()
            .is_some_and(|line| line.trim_start().starts_with('>'));
        let is_source = !navigation
            && in_component(file, "wiki")
            && in_component(&note.path, "raw")
            && note.name != "raw";
        let path_label = alias.is_some_and(|s| path_like(s, target));
        let bare_path = target.contains('/') && alias.is_none();
        let source_label = is_source
            && alias.is_none_or(|s| !s.trim_end().ends_with('↗') || s.chars().count() > 66);
        if !(bare_path || path_label || source_label) {
            continue;
        }
        let short_target =
            if !preserved && notes.iter().filter(|n| n.name == note.name).count() == 1 {
                note.name.clone()
            } else {
                target.to_owned()
            };
        let label = match alias.filter(|s| !s.trim().is_empty() && !path_like(s, target)) {
            Some(label) if is_source => {
                compact_title(label.trim().trim_end_matches('↗').trim_end())
            }
            Some(label) => label.trim().to_owned(),
            None => compact_title(&note.title),
        };
        let label = if is_source && !label.ends_with('↗') {
            format!("{label} ↗")
        } else {
            label
        };
        let separator = if source[..start]
            .rsplit('\n')
            .next()
            .is_some_and(|line| line.trim_start().starts_with('|'))
        {
            "\\|"
        } else {
            "|"
        };
        let replacement = if !is_source
            && alias.is_none()
            && short_target == note.name
            && !target.ends_with("/wiki")
            && !target.ends_with("/raw")
        {
            format!("[[{short_target}{anchor}]]")
        } else {
            format!("[[{short_target}{anchor}{separator}{label}]]")
        };
        let reason = if source_label {
            "source-link-label"
        } else {
            "visible-link-path"
        };
        result.push(Edit {
            start,
            end,
            replacement,
            reason,
        });
    }
    result
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

#[cfg(test)]
mod tests {
    use super::*;
    fn notes() -> Vec<Note> {
        vec![
            Note {
                path: "d/wiki/wiki.md".into(),
                name: "wiki".into(),
                title: "Kennisonderwerp".into(),
            },
            Note {
                path: "other/wiki/wiki.md".into(),
                name: "wiki".into(),
                title: "Andere kennis".into(),
            },
            Note {
                path: "d/raw/20261111-source/20261111-source.md".into(),
                name: "20261111-source".into(),
                title: "Gesprek over toegang".into(),
            },
            Note {
                path: "d/wiki/guide/guide.md".into(),
                name: "guide".into(),
                title: "Handleiding".into(),
            },
        ]
    }
    fn repair(source: &str, path: &str) -> String {
        let mut output = source.to_owned();
        for edit in edits(source, Path::new(path), &notes()).iter().rev() {
            output.replace_range(edit.start..edit.end, &edit.replacement);
        }
        output
    }
    #[test]
    fn fixes_paths_without_changing_destination_or_anchor() {
        assert_eq!(
            repair("[[d/wiki/wiki#Sectie]] [[d/wiki/guide/guide]]", "owner.md"),
            "[[d/wiki/wiki#Sectie|Kennisonderwerp]] [[guide]]"
        );
        assert_eq!(
            repair(
                "[[d/raw/20261111-source/20261111-source#Bewijs]]",
                "d/wiki/wiki.md"
            ),
            "[[20261111-source#Bewijs|Gesprek over toegang ↗]]"
        );
    }
    #[test]
    fn requires_subtle_source_labels_and_is_idempotent() {
        let original =
            "[[20261111-source]] [[20261111-source|Toegang]] [[guide]] [[d/wiki/wiki|Kennis]]";
        let fixed = repair(original, "d/wiki/wiki.md");
        assert_eq!(
            fixed,
            "[[20261111-source|Gesprek over toegang ↗]] [[20261111-source|Toegang ↗]] [[guide]] [[d/wiki/wiki|Kennis]]"
        );
        assert_eq!(repair(&fixed, "d/wiki/wiki.md"), fixed);
        let too_long = format!("[[20261111-source|{} ↗]]", "Lang bronlabel ".repeat(12));
        let fixed = repair(&too_long, "d/wiki/wiki.md");
        let label = fixed.split_once('|').unwrap().1.trim_end_matches("]]");
        assert!(label.chars().count() <= 66);
        assert!(label.ends_with('↗'));
        assert_eq!(repair(&fixed, "d/wiki/wiki.md"), fixed);
    }
    #[test]
    fn ignores_metadata_code_comments_embeds_quotes_and_sources() {
        let text = "---\nref: '[[d/wiki/wiki]]'\n---\n```md\n[[d/wiki/wiki]]\n```\n~~~md\n[[d/wiki/wiki]]\n~~~\n`[[d/wiki/wiki]]` <!-- [[d/wiki/wiki]] --> %% [[d/wiki/wiki]] %%\n![[d/wiki/wiki]]\n> Quote [[d/wiki/wiki]]\n    [[d/wiki/wiki]]\n";
        assert_eq!(repair(text, "d/wiki/wiki.md"), text);
        let raw = "> [!navigation|shortcuts]\n> - [[d/wiki/wiki]]\n\n## Activity\nMens [[d/wiki/wiki]]\n> [!activity-message|agent] Codex\n> [[d/wiki/wiki]]\n";
        let fixed = repair(raw, "d/raw/20261111-source/20261111-source.md");
        assert!(fixed.contains("> - [[d/wiki/wiki|Kennisonderwerp]]"));
        assert!(fixed.contains("Mens [[d/wiki/wiki]]"));
        assert!(fixed.contains("> [[d/wiki/wiki]]"));
    }
    #[test]
    fn handles_unicode_and_escaped_table_aliases() {
        assert_eq!(
            repair(
                "| é | [[d/raw/20261111-source/20261111-source\\|Bewijs]] |",
                "d/wiki/wiki.md"
            ),
            "| é | [[20261111-source\\|Bewijs ↗]] |"
        );
        assert_eq!(
            repair("<!-- ü [[d/wiki/wiki]] --> é [[d/wiki/wiki]]", "owner.md"),
            "<!-- ü [[d/wiki/wiki]] --> é [[d/wiki/wiki|Kennisonderwerp]]"
        );
    }
    #[test]
    fn refuses_missing_and_ambiguous_targets() {
        let original = "[[missing/path]] [[wiki]]";
        assert_eq!(repair(original, "owner.md"), original);
        assert_eq!(
            edits(original, Path::new("owner.md"), &notes())[0].reason,
            "unresolved-long-link"
        );
    }

    #[test]
    fn keeps_fixed_navigation_labels_and_human_slashes() {
        let source = "> [!navigation|shortcuts]\n> - [[d/raw/20261111-source/20261111-source|Raw]]\n\n[[d/wiki/wiki|AI/ML]]";
        assert_eq!(repair(source, "d/wiki/wiki.md"), source);
    }

    #[test]
    fn raw_processing_only_changes_the_named_agent_bubble() {
        let source = "---\nwiki_target: '[[d/wiki/wiki#Kop]]'\n---\n## Activity\n\n### Verwerking in Wiki\n\n> [!activity-message|agent] Codex\n> [[d/wiki/wiki#Kop]]\n> *Codex*\n\n> [!activity-message|user] Alex\n> [[d/wiki/wiki#Kop]]\n\n## Oorspronkelijk\n[[d/wiki/wiki#Kop]]\n";
        let fixed = repair(source, "d/raw/20261111-source/20261111-source.md");
        assert_eq!(
            fixed,
            source.replacen(
                "> [[d/wiki/wiki#Kop]]",
                "> [[d/wiki/wiki#Kop|Kennisonderwerp]]",
                1
            )
        );
        assert_eq!(
            repair(&fixed, "d/raw/20261111-source/20261111-source.md"),
            fixed
        );
        let unique = source.replace("d/wiki/wiki", "d/wiki/guide/guide");
        assert_eq!(
            repair(&unique, "d/raw/20261111-source/20261111-source.md"),
            unique.replacen(
                "> [[d/wiki/guide/guide#Kop]]",
                "> [[d/wiki/guide/guide#Kop|Handleiding]]",
                1
            )
        );
    }

    #[test]
    fn adds_existing_wiki_style_to_leaf_notes_in_both_line_endings() {
        for newline in ["\n", "\r\n"] {
            let source = format!(
                "---{newline}layer: wiki{newline}title: élève{newline}---{newline}# Wiki{newline}"
            );
            let fixed = repair(&source, "d/wiki/guide/guide.md");
            assert_eq!(
                fixed,
                source.replace(
                    &format!("title: élève{newline}"),
                    &format!("title: élève{newline}cssclasses: [vault-wiki]{newline}")
                )
            );
            assert_eq!(repair(&fixed, "d/wiki/guide/guide.md"), fixed);
        }
    }
}
