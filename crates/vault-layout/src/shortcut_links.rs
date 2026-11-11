use std::{
    cell::OnceCell,
    fs, io,
    path::{Path, PathBuf},
};

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

pub(crate) fn anchor_count(markdown: &str, anchor: &str) -> usize {
    let mut lines = markdown.lines().peekable();
    if lines.peek() == Some(&"---") {
        lines.next();
        for line in lines.by_ref() {
            if line == "---" {
                break;
            }
        }
    }
    let headings: Vec<_> = anchor.split('#').map(decoded_heading).collect();
    let mut stack = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut comment = false;
    let mut previous: Option<String> = None;
    let mut count = 0;
    for line in lines {
        let line = crate::index_navigation::quote_content(line).trim_end();
        let trimmed = line.trim_start_matches(' ');
        if line.len() - trimmed.len() <= 3
            && let Some(marker @ (b'`' | b'~')) = trimmed.bytes().next()
        {
            let length = trimmed.bytes().take_while(|&byte| byte == marker).count();
            if let Some((opening, minimum)) = fence {
                if marker == opening && length >= minimum && trimmed[length..].trim().is_empty() {
                    fence = None;
                }
                continue;
            }
            if !comment && length >= 3 {
                fence = Some((marker, length));
                previous = None;
                continue;
            }
        }
        if fence.is_some() {
            continue;
        }
        let mut rest = line;
        let mut visible = String::new();
        loop {
            if comment {
                let Some((_, after)) = rest.split_once("-->") else {
                    break;
                };
                comment = false;
                rest = after;
            } else if let Some((before, after)) = rest.split_once("<!--") {
                visible.push_str(before);
                comment = true;
                rest = after;
            } else {
                visible.push_str(rest);
                break;
            }
        }
        if let Some(block) = anchor.strip_prefix('^') {
            count += usize::from(
                !block.is_empty()
                    && visible
                        .split_whitespace()
                        .last()
                        .is_some_and(|word| word == anchor),
            );
            continue;
        }
        let trimmed = visible.trim_start_matches(' ');
        let prior = previous.take();
        let Some((level, title)) =
            heading(trimmed, prior.as_deref()).filter(|_| visible.len() - trimmed.len() <= 3)
        else {
            if !trimmed.trim().is_empty() && visible.len() - trimmed.len() <= 3 {
                previous = Some(trimmed.trim().to_owned());
            }
            continue;
        };
        stack.truncate(
            stack
                .iter()
                .position(|(old, _)| *old >= level)
                .unwrap_or(stack.len()),
        );
        stack.push((level, heading_text(title)));
        if !title.is_empty()
            && stack.len() >= headings.len()
            && stack[stack.len() - headings.len()..]
                .iter()
                .map(|(_, title)| title.as_str())
                .eq(headings.iter().map(String::as_str))
        {
            count += 1;
        }
    }
    count
}

pub(crate) fn heading<'a>(line: &'a str, previous: Option<&'a str>) -> Option<(usize, &'a str)> {
    let level = line.bytes().take_while(|&byte| byte == b'#').count();
    if (1..=6).contains(&level) && line[level..].starts_with([' ', '\t']) {
        let title = line[level..].trim();
        let tail = title.trim_end_matches('#');
        return Some((
            level,
            if tail.ends_with([' ', '\t']) {
                tail.trim_end()
            } else {
                title
            },
        ));
    }
    let marker = line.as_bytes().first()?;
    if !line.bytes().all(|byte| byte == *marker) {
        return None;
    }
    let level = match marker {
        b'=' => 1,
        b'-' => 2,
        _ => return None,
    };
    previous.map(|title| (level, title))
}

fn decoded_heading(heading: &str) -> String {
    decode_percent(heading).trim().to_owned()
}

pub(crate) fn decode_percent(heading: &str) -> String {
    let mut bytes = heading.as_bytes().iter().copied().peekable();
    let mut decoded = Vec::with_capacity(heading.len());
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let mut lookahead = bytes.clone();
            let hex = |byte: u8| (byte as char).to_digit(16);
            if let (Some(high), Some(low)) = (
                lookahead.next().and_then(hex),
                lookahead.next().and_then(hex),
            ) {
                decoded.push((high * 16 + low) as u8);
                bytes = lookahead;
                continue;
            }
        }
        decoded.push(byte);
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn heading_text(title: &str) -> String {
    let mut title = title.replace('`', "");
    for marker in ["**", "__", "~~", "*", "_"] {
        let mut offset = 0;
        while let Some(start) = title[offset..].find(marker) {
            let start = offset + start;
            let content = start + marker.len();
            let Some(end) = title[content..].find(marker) else {
                break;
            };
            let end = content + end;
            title.replace_range(end..end + marker.len(), "");
            title.replace_range(start..content, "");
            offset = end - marker.len();
        }
    }
    title
}

#[cfg(test)]
mod tests {
    use super::anchor_count;

    #[test]
    fn counts_live_headings_and_rejects_examples() {
        let note = "---\n# YAML\n---\n# Parent\n## Child ##\n<!--\n## Hidden\n-->\n```md\n## Example\n```\n# Other\n## Child\n";
        assert_eq!(anchor_count(note, "Parent#Child"), 1);
        assert_eq!(anchor_count(note, "Child"), 2);
        assert_eq!(anchor_count("## **Mijn kop**", "Mijn%20kop"), 1);
        assert_eq!(
            anchor_count("Intro\n=====\nDetails\n-------", "Intro#Details"),
            1
        );
        for absent in ["YAML", "Hidden", "Example", "Missing", ""] {
            assert_eq!(anchor_count(note, absent), 0);
        }
    }
}
