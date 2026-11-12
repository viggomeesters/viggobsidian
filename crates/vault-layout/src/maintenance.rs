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

/// Only actual wiki-link targets and single-backtick path tokens are locators.
/// Prose, quoted original input and fenced source text remain byte-identical.
pub(crate) fn rebind(before: &str, mapping: &Value) -> Result<String> {
    let mapping = mapping
        .as_object()
        .filter(|m| !m.is_empty())
        .ok_or("mapping missing")?;
    for (old, new) in mapping {
        for value in [old.as_str(), new.as_str().ok_or("invalid locator")?] {
            let p = path(value)?;
            if p.components().count() < 3 || p.extension().is_none() {
                return Err("expected a complete file locator".into());
            }
        }
    }
    let mut changed = std::collections::BTreeSet::new();
    let mut output = String::new();
    let mut fence: Option<(char, usize)> = None;
    for line in before.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if let Some(marker @ ('`' | '~')) = trimmed.chars().next() {
            let n = trimmed.chars().take_while(|c| *c == marker).count();
            if n >= 3 {
                if fence.is_none() {
                    fence = Some((marker, n))
                } else if fence.is_some_and(|(m, w)| m == marker && n >= w) {
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
            if let Some(tail) = rest.strip_prefix("[[") {
                if let Some((inside, after)) = tail.split_once("]]") {
                    let (target, suffix) = inside
                        .find(['#', '|'])
                        .map_or((inside, ""), |i| inside.split_at(i));
                    let found = mapping.iter().find(|(old, _)| {
                        old.trim_end_matches(".md") == target || old.as_str() == target
                    });
                    if let Some((old, new)) = found {
                        output.push_str("[[");
                        output.push_str(new.as_str().unwrap().trim_end_matches(".md"));
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
            }
            if let Some(tail) = rest.strip_prefix('`') {
                if let Some((inside, after)) = tail.split_once('`') {
                    if let Some(new) = mapping.get(inside) {
                        output.push('`');
                        output.push_str(new.as_str().unwrap());
                        output.push('`');
                        changed.insert(inside.to_owned());
                        rest = after;
                        continue;
                    }
                }
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
            if !dest.components().any(|p| p.as_os_str() == "wiki")
                || dest.extension().is_none_or(|e| e != "md")
                || dest == p
                || before.contains(START)
                || before.contains(END)
            {
                return Err("conflicting or invalid knowledge handoff".into());
            }
            let (fm, body) = before.split_once("\n---\n").ok_or("frontmatter missing")?;
            Ok(format!(
                "{fm}\n---\n\n{START}\n> [!info] Actuele kennis\n> [[{}|Open de onderhouden Wiki]]\n{END}\n{body}",
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
            if metadata(&after)? != data {
                return Err("locator change cannot alter identity or metadata".into());
            }
            Ok(after)
        }
        _ => Err("unknown compatibility operation".into()),
    }
}
/// Preserve the original bytes for audits while allowing a fixed, visible notice.
pub(crate) fn without_notice(body: &str) -> &str {
    if let Some(rest) = body.trim_start_matches('\n').strip_prefix(START) {
        if let Some((notice, tail)) = rest.split_once(END) {
            let lines: Vec<_> = notice.lines().filter(|l| !l.is_empty()).collect();
            if lines.len() == 2
                && lines[0] == "> [!info] Actuele kennis"
                && lines[1].starts_with("> [[")
                && lines[1].ends_with("|Open de onderhouden Wiki]]")
            {
                return tail.strip_prefix('\n').unwrap_or(tail);
            }
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
