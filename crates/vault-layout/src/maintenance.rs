//! Typed, byte-preserving compatibility changes. Not a general baseline waiver.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const START: &str = "<!-- viggobsidian:knowledge-owner -->";
const END: &str = "<!-- /viggobsidian:knowledge-owner -->";
pub(crate) fn path(raw: &str) -> Result<&Path> {
    let p = Path::new(raw);
    if raw.is_empty()
        || raw.contains(['\\', '\n', '\r', '|', '#', '[', ']'])
        || !p
            .components()
            .all(|c| matches!(c,Component::Normal(s) if !s.to_string_lossy().starts_with('.')))
    {
        return Err("unsafe maintenance locator".into());
    }
    Ok(p)
}
fn metadata(text: &str) -> Result<Value> {
    let fm = text
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n"))
        .ok_or("maintenance note needs frontmatter")?
        .0;
    Ok(serde_saphyr::from_str(fm)?)
}

fn file_locator(value: &str) -> bool {
    !value.contains([':', '?', '\0'])
        && path(value).is_ok_and(|p| p.components().count() >= 3 && p.extension().is_some())
}
fn local_uri(value: &str) -> bool {
    if value.chars().any(|c| {
        c.is_whitespace() || matches!(c, '`' | '<' | '>' | '[' | ']' | '(' | ')' | '"' | '\'')
    }) {
        return false;
    }
    if let Some(value) = value.strip_prefix("file:///") {
        return path(&crate::shortcut_links::decode_percent(value)).is_ok();
    }
    let Some(query) = value.strip_prefix("obsidian://open?") else {
        return false;
    };
    if query.split('&').count() != 2 {
        return false;
    }
    let fields = query
        .split('&')
        .filter_map(|v| v.split_once('='))
        .collect::<BTreeMap<_, _>>();
    fields.len() == 2
        && fields.get("vault").is_some_and(|v| !v.is_empty())
        && fields
            .get("file")
            .is_some_and(|v| path(&crate::shortcut_links::decode_percent(v)).is_ok())
}
fn wiki_locator(value: &str) -> bool {
    path(value).is_ok() && !value.contains([' ', ':', '`', '"', '\''])
}
fn comment_continues(line: &str, was_open: bool) -> bool {
    match (line.rfind("<!--"), line.rfind("-->")) {
        (Some(open), Some(close)) => open > close,
        (Some(_), None) => true,
        (None, Some(_)) => false,
        (None, None) => was_open,
    }
}
/// Only explicit WikiLinks, local Markdown links, declared external-task paths
/// and complete single-backtick file locators are maintenance tokens.
/// Prose, quoted original input and fenced source text remain byte-identical.
pub(crate) fn rebind(before: &str, mapping: &Value) -> Result<String> {
    let mapping = mapping
        .as_object()
        .filter(|m| !m.is_empty())
        .ok_or("mapping missing")?;
    for (old, new) in mapping {
        if !(wiki_locator(old) || local_uri(old)) {
            return Err("invalid original locator".into());
        }
        if new.is_null() {
            if !local_uri(old) {
                return Err(
                    "only an unavailable local Markdown URI can become historical text".into(),
                );
            }
        } else if !new
            .as_str()
            .is_some_and(|n| file_locator(n) || local_uri(n))
        {
            return Err("expected a complete replacement file locator or local URI".into());
        }
    }
    let mut changed = std::collections::BTreeSet::new();
    let mut output = String::new();
    let mut fence: Option<(char, usize)> = None;
    let mut frontmatter = true;
    let mut external = false;
    let mut delimiters = 0;
    let mut source_quote = false;
    let mut html_end: Option<&str> = None;
    let mut html_block = false;
    let mut inline_run: Option<usize> = None;
    for line in before.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if line.trim_end() == "---" {
            delimiters += 1;
            frontmatter = delimiters < 2;
            output.push_str(line);
            continue;
        }
        if frontmatter {
            if line.trim_end() == "external-tasks:" {
                external = true;
            } else if !line.starts_with([' ', '-', '\t']) && !line.trim().is_empty() {
                external = false;
            }
            if external && let Some(value) = line.strip_prefix("- path: ") {
                let (value, newline) = value.strip_suffix('\n').map_or((value, ""), |v| (v, "\n"));
                if let Some(new) = mapping
                    .get(value)
                    .and_then(Value::as_str)
                    .filter(|new| file_locator(value) && file_locator(new))
                {
                    output.push_str(&format!("- path: {new}{newline}"));
                    changed.insert(value.into());
                    continue;
                }
            }
            output.push_str(line);
            continue;
        }
        if let Some(width) = inline_run {
            let marker = "`".repeat(width);
            if let Some((i, _)) = line.match_indices(&marker).find(|(i, _)| {
                (*i == 0 || line.as_bytes()[i - 1] != b'`')
                    && line.as_bytes().get(i + width) != Some(&b'`')
            }) {
                if line[i + width..].contains('`') {
                    return Err("complex multiline inline-code transition is outside typed locator maintenance".into());
                }
                inline_run = None;
            }
            output.push_str(line);
            continue;
        }
        if let Some((marker, width)) = fence {
            let indent = line.len() - trimmed.len();
            let n = trimmed.chars().take_while(|c| *c == marker).count();
            if indent <= 3 && n >= width && trimmed[n..].trim().is_empty() {
                fence = None;
            }
            output.push_str(line);
            continue;
        }
        if html_block {
            if trimmed.is_empty() {
                html_block = false;
            }
            output.push_str(line);
            continue;
        }
        if let Some(end) = html_end {
            if end == "-->" {
                html_end = comment_continues(line, true).then_some("-->");
            } else if line.to_ascii_lowercase().contains(end) {
                html_end = None;
            }
            output.push_str(line);
            continue;
        }
        let lower = line.to_ascii_lowercase();
        if lower.contains("<!--") {
            if comment_continues(&lower, false) {
                html_end = Some("-->");
            }
            output.push_str(line);
            continue;
        }
        let html = [
            ("<pre", "</pre>"),
            ("<code", "</code>"),
            ("<script", "</script>"),
            ("<style", "</style>"),
        ]
        .into_iter()
        .find(|(start, _)| lower.contains(start));
        if let Some((_, end)) = html {
            if !lower.contains(end) {
                html_end = Some(end);
            }
            output.push_str(line);
            continue;
        }
        if line.contains('<') {
            if trimmed.starts_with('<') {
                html_block = true;
            }
            output.push_str(line);
            continue;
        }
        if trimmed.starts_with('>') {
            source_quote = true;
            output.push_str(line);
            continue;
        }
        if source_quote {
            if trimmed.is_empty() || trimmed.starts_with('#') {
                source_quote = false;
            } else {
                output.push_str(line);
                continue;
            }
        }
        if line.starts_with("    ") || line.starts_with('\t') {
            output.push_str(line);
            continue;
        }
        if let Some(marker @ ('`' | '~')) = trimmed.chars().next() {
            let n = trimmed.chars().take_while(|c| *c == marker).count();
            if n >= 3 {
                if fence.is_none() {
                    fence = Some((marker, n))
                } else if fence
                    .is_some_and(|(m, w)| m == marker && n >= w && trimmed[n..].trim().is_empty())
                {
                    fence = None
                }
                output.push_str(line);
                continue;
            }
        }
        if fence.is_some() || trimmed.starts_with('>') {
            output.push_str(line);
            continue;
        }
        let mut rest = line;
        while !rest.is_empty() {
            if let Some(tail) = rest.strip_prefix('\\') {
                output.push('\\');
                if let Some(c) = tail.chars().next() {
                    output.push(c);
                    rest = &tail[c.len_utf8()..];
                } else {
                    rest = tail;
                }
                continue;
            }
            if let Some(tail) = rest.strip_prefix("[[")
                && let Some((inside, after)) = tail.split_once("]]")
            {
                let (target, suffix) = inside
                    .find(['#', '|'])
                    .map_or((inside, ""), |i| inside.split_at(i));
                let found = mapping.iter().find(|(old, _)| {
                    !local_uri(old)
                        && (old.strip_suffix(".md").unwrap_or(old) == target
                            || old.as_str() == target)
                });
                if let Some((old, new)) = found {
                    let new = new
                        .as_str()
                        .filter(|n| file_locator(n))
                        .ok_or("WikiLink needs a complete replacement file locator")?;
                    output.push_str("[[");
                    output.push_str(new.strip_suffix(".md").unwrap_or(new));
                    output.push_str(suffix);
                    output.push_str("]]");
                    changed.insert(old.clone());
                } else {
                    output.push_str("[[");
                    output.push_str(inside);
                    output.push_str("]]");
                }
                rest = after;
                continue;
            }
            if let Some(tail) = rest.strip_prefix('[')
                && let Some((label, tail)) = tail.split_once("](")
                && let Some((uri, after)) = tail.split_once(')')
                && let Some(new) = mapping.get(uri).filter(|_| local_uri(uri))
            {
                if new.is_null() {
                    output.push_str(&format!("{label} (historische verwijzing; oorspronkelijke lokale bron niet teruggevonden: `{uri}`)"));
                } else {
                    let new = new
                        .as_str()
                        .filter(|n| local_uri(n))
                        .ok_or("local Markdown link requires a local replacement URI")?;
                    output.push_str(&format!("[{label}]({new})"));
                }
                changed.insert(uri.into());
                rest = after;
                continue;
            }
            if rest.starts_with('`') {
                let width = rest.bytes().take_while(|b| *b == b'`').count();
                let marker = "`".repeat(width);
                let tail = &rest[width..];
                let closing = tail.match_indices(&marker).find(|(i, _)| {
                    (*i == 0 || tail.as_bytes()[i - 1] != b'`')
                        && tail.as_bytes().get(i + width) != Some(&b'`')
                });
                let Some((end, _)) = closing else {
                    inline_run = Some(width);
                    output.push_str(rest);
                    break;
                };
                let inside = &tail[..end];
                let after = &tail[end + width..];
                output.push_str(&marker);
                if let Some(new) = mapping
                    .get(inside)
                    .and_then(Value::as_str)
                    .filter(|n| width == 1 && file_locator(inside) && file_locator(n))
                {
                    output.push_str(new);
                    changed.insert(inside.into());
                } else {
                    output.push_str(inside);
                }
                output.push_str(&marker);
                rest = after;
                continue;
            }
            let c = rest.chars().next().unwrap();
            output.push(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    if changed.len() != mapping.len() {
        return Err("a mapped locator token was not found".into());
    }
    Ok(output)
}
pub(crate) fn transform(raw: &str, proof: &Value) -> Result<String> {
    let p = path(raw)?;
    let before = proof["before"]
        .as_str()
        .ok_or("maintenance preimage missing")?;
    let data = metadata(before)?;
    if ![
        "note",
        "collection",
        "reference",
        "task",
        "process",
        "dossier",
    ]
    .contains(&data["type"].as_str().unwrap_or(""))
        || data["category"] == "source"
        || data.get("activity_contract").is_some()
        || data.get("source_inventory").is_some()
        || p.components().any(|part| {
            matches!(
                part.as_os_str().to_str(),
                Some("raw" | "sources" | "events" | "imported" | "generated")
            ) && !(part.as_os_str() == "sources"
                && p.file_name()
                    .is_some_and(|s| s == "source-index.md" || s == "sources.md"))
        })
    {
        return Err("protected source is not a maintained owner".into());
    }
    match proof["kind"].as_str() {
        Some("owner-handoff") => {
            if proof
                .as_object()
                .ok_or("invalid proof")?
                .keys()
                .any(|k| !["kind", "before", "target"].contains(&k.as_str()))
                || data.get("status").is_some()
                || data["type"] == "task"
            {
                return Err("handoff is not a task or free-form rewrite".into());
            }
            let target = proof["target"].as_str().ok_or("handoff target missing")?;
            let dest = path(target)?;
            let wiki = dest.parent().is_some_and(|p| p.ends_with("wiki"));
            let folder = dest.parent().and_then(Path::file_name) == dest.file_stem();
            if !(wiki || folder)
                || dest.components().any(|p| {
                    matches!(
                        p.as_os_str().to_str(),
                        Some("raw" | "sources" | "events" | "generated" | "imported")
                    )
                })
                || dest.extension().is_none_or(|e| e != "md")
                || dest == p
                || before.contains(START)
                || before.contains(END)
            {
                return Err("conflicting or invalid knowledge handoff".into());
            }
            let (fm, body) = before.split_once("\n---\n").ok_or("frontmatter missing")?;
            Ok(format!(
                "{fm}\n---\n\n{START}\n> [!info] Actuele kennis\n> [[{}|Open actuele kennis]]\n{END}\n{body}",
                target.trim_end_matches(".md")
            ))
        }
        Some("locator-rebind") => {
            if proof
                .as_object()
                .ok_or("invalid proof")?
                .keys()
                .any(|k| !["kind", "before", "mapping"].contains(&k.as_str()))
            {
                return Err("unknown locator proof field".into());
            }
            let after = rebind(before, &proof["mapping"])?;
            let mut restored = metadata(&after)?;
            if let Some(rows) = restored
                .get_mut("external-tasks")
                .and_then(Value::as_array_mut)
                && let Some(originals) = data["external-tasks"].as_array()
            {
                for (row, original) in rows.iter_mut().zip(originals) {
                    if let Some(old) = original["path"].as_str()
                        && proof["mapping"].get(old) == row.get("path")
                        && file_locator(old)
                    {
                        row["path"] = original["path"].clone();
                    }
                }
            }
            if restored != data {
                return Err("locator change cannot alter identity or metadata".into());
            }
            Ok(after)
        }
        _ => Err("unknown compatibility operation".into()),
    }
}
/// Preserve the original bytes for audits while allowing a fixed, visible notice.
pub(crate) fn without_notice(body: &str) -> &str {
    if let Some(rest) = body.trim_start_matches('\n').strip_prefix(START)
        && let Some((notice, tail)) = rest.split_once(END)
    {
        let lines: Vec<_> = notice.lines().filter(|l| !l.is_empty()).collect();
        if lines.len() == 2
            && lines[0] == "> [!info] Actuele kennis"
            && lines[1].starts_with("> [[")
            && lines[1].ends_with("|Open actuele kennis]]")
        {
            return tail.strip_prefix('\n').unwrap_or(tail);
        }
    }
    body
}
pub(crate) fn inventory(root: &Path, excluded: &[String]) -> Result<BTreeMap<String, String>> {
    fn visit(
        base: &Path,
        at: &Path,
        skip: &[String],
        out: &mut BTreeMap<String, String>,
    ) -> Result<()> {
        if !at.is_dir() {
            return Ok(());
        }
        for e in fs::read_dir(at)? {
            let e = e?;
            let p = e.path();
            let rel = p.strip_prefix(base)?.to_string_lossy().into_owned();
            if e.file_type()?.is_symlink() {
                return Err("symbolic preserved owner member".into());
            }
            if e.file_type()?.is_dir() {
                visit(base, &p, skip, out)?
            } else if e.file_type()?.is_file() && !skip.contains(&rel) {
                out.insert(rel, format!("{:x}", Sha256::digest(fs::read(p)?)));
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    visit(root, root, excluded, &mut out)?;
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    const NOTE: &str = "---\ntype: note\ncategory: knowledge\n---\n\n# Existing\n\nA fact.\n";
    #[test]
    fn locator_repairs_preserve_protected_markdown_source_contexts() {
        for source in [
            "`` [[old-note]] ``",
            "\\[[old-note]]",
            "    [[old-note]]",
            "<!-- archived [[old-note]] -->",
            "<!-- archived\n[[old-note]]\n-->",
            "<pre>\n[[old-note]]\n</pre>",
            "> Original source says\n[[old-note]]",
            "```md\n[[unmapped]]\n```not-a-close\n[[old-note]]\n```",
            "~~~md\n[[unmapped]]\n~~~not-a-close\n[[old-note]]\n~~~",
            "<div>\n[[old-note]]\n</div>",
            "<span data-original=\"[[old-note]]\">Archived</span>",
            "`original\n[[old-note]]\nsource`",
            "--> <!-- original\n[[old-note]]\n-->",
        ] {
            let before = NOTE.replace("A fact.", source);
            let p = json!({"kind":"locator-rebind","before":before,"mapping":{"old-note":"30-topics/new/source.md"}});
            assert!(
                transform("30-topics/case/case.md", &p).is_err(),
                "protected input was treated as a locator: {source}"
            );
            let before = NOTE.replace("A fact.", &format!("{source}\n\n[[old-note]]"));
            let p = json!({"kind":"locator-rebind","before":before,"mapping":{"old-note":"30-topics/new/source.md"}});
            let after = transform("30-topics/case/case.md", &p).unwrap();
            assert!(after.contains(source), "protected input changed: {source}");
            assert!(after.ends_with("[[30-topics/new/source]]\n"));
        }
    }
    #[test]
    fn complex_multiline_code_transitions_fail_closed_without_changing_source() {
        let before = NOTE.replace(
            "A fact.",
            "`one\nend` and `two\n[[old-note]]\nend`\n\n[[old-note]]",
        );
        assert!(transform("30-topics/case/case.md",&json!({"kind":"locator-rebind","before":before,"mapping":{"old-note":"30-topics/new/source.md"}})).is_err());
    }
    #[test]
    fn actual_basename_wikilinks_preserve_labels_anchors_and_source_quotes() {
        let before=NOTE.replace("A fact.","[[old-note#Original|Read]]\n`[[old-note]]`\n> [[old-note]]\n```md\n[[old-note]]\n```\nOld-note ordinary prose.");
        let proof = json!({"kind":"locator-rebind","before":before,"mapping":{"old-note":"30-topics/case/raw/source.md"}});
        let after = transform("30-topics/case/case.md", &proof).unwrap();
        assert!(after.contains("[[30-topics/case/raw/source#Original|Read]]"));
        assert!(after.contains(
            "`[[old-note]]`\n> [[old-note]]\n```md\n[[old-note]]\n```\nOld-note ordinary prose."
        ));
        assert!(transform("30-topics/case/case.md",&json!({"kind":"locator-rebind","before":NOTE.replace("A fact.","`old-note`"),"mapping":{"old-note":"30-topics/case/raw/source.md"}})).is_err());
    }
    #[test]
    fn external_task_locator_changes_only_the_declared_path_and_preserves_identity() {
        let before = "---\ntype: collection\ncategory: tasks\nexternal-tasks:\n- path: 30-topics/old/task.md\n  task-id: task:example\nstatus: done\n---\n\n[[30-topics/old/task]]\n";
        let proof = json!({"kind":"locator-rebind","before":before,"mapping":{"30-topics/old/task.md":"30-topics/new/raw/task.md"}});
        let after = transform("20-projects/example/01-tasks/01-tasks.md", &proof).unwrap();
        assert_eq!(
            after,
            before.replace("30-topics/old/task", "30-topics/new/raw/task")
        );
        assert!(after.contains("task-id: task:example\nstatus: done"));
        let other = before.replace("external-tasks:", "ordinary-metadata:");
        let p = json!({"kind":"locator-rebind","before":other,"mapping":{"30-topics/old/task.md":"30-topics/new/raw/task.md"}});
        assert!(
            transform("20-projects/example/01-tasks/01-tasks.md", &p)
                .unwrap()
                .contains("- path: 30-topics/old/task.md")
        );
    }
    #[test]
    fn local_markdown_uris_rebind_or_retain_unavailable_original_as_literal_evidence() {
        let old = "obsidian://open?vault=example&file=30-topics%2Fold%2Fsource.md";
        let new = "obsidian://open?vault=example&file=30-topics%2Fnew%2Fsource.md";
        let before = NOTE.replace(
            "A fact.",
            &format!("[Read source]({old})\n> [Read source]({old})"),
        );
        let p = json!({"kind":"locator-rebind","before":before,"mapping":{old:new}});
        let after = transform("30-topics/case/case.md", &p).unwrap();
        assert!(after.contains(&format!("[Read source]({new})\n> [Read source]({old})")));
        let p = json!({"kind":"locator-rebind","before":before,"mapping":{old:null}});
        let after = transform("30-topics/case/case.md", &p).unwrap();
        assert!(after.contains(&format!("Read source (historische verwijzing; oorspronkelijke lokale bron niet teruggevonden: `{old}`)")));
        for invalid in [
            "https://example.org/file",
            "obsidian://open?vault=x&file=a&file=b",
            "obsidian://open?vault=x&file=..%2Fsecret",
            "file:///..%2Fsecret",
            "file:///x/`bad`",
        ] {
            assert!(transform("30-topics/case/case.md",&json!({"kind":"locator-rebind","before":NOTE.replace("A fact.",&format!("[Read]({invalid})")),"mapping":{invalid:null}})).is_err(),"{invalid}");
        }
    }
    #[test]
    fn handoff_has_no_free_postimage() {
        let p = json!({"kind":"owner-handoff","before":NOTE,"target":"30-topics/fiction/case/wiki/wiki.md"});
        let after = transform("30-topics/fiction/old/wiki/wiki.md", &p).unwrap();
        let (_, body) = after.split_once("\n---\n").unwrap();
        assert_eq!(without_notice(body), NOTE.split_once("\n---\n").unwrap().1);
        let mut wrong = p.clone();
        wrong["text"] = json!("invented fact");
        assert!(transform("30-topics/fiction/old/wiki/wiki.md", &wrong).is_err());
        assert!(transform("30-topics/fiction/raw/source.md", &p).is_err());
    }
    #[test]
    fn locator_metadata_is_immutable() {
        let p = json!({"kind":"locator-rebind","before":NOTE,"mapping":{"30-topics/a/a.md":"30-topics/b/b.md"}});
        assert!(transform("30-topics/fiction/old.md", &p).is_err());
        let p = json!({"kind":"locator-rebind","before":NOTE.replace("A fact.","[[30-topics/a/a.md]]"),"mapping":{"30-topics/a/a.md":"30-topics/b/b.md"}});
        assert!(
            transform("30-topics/fiction/old.md", &p)
                .unwrap()
                .contains("[[30-topics/b/b]]")
        );
    }
}
