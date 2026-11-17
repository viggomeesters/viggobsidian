//! One exact maintained-Wiki block; no generic rewrite or baseline waiver.
use serde_json::Value;
use std::path::Path;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fence_line(line: &str) -> Option<(char, usize, &str)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let ch = rest.chars().next()?;
    if !matches!(ch, '`' | '~') {
        return None;
    }
    let width = rest.bytes().take_while(|b| *b == ch as u8).count();
    (width >= 3).then(|| (ch, width, &rest[width..]))
}
fn closes_fence(line: &str, ch: char, width: usize) -> bool {
    fence_line(line).is_some_and(|(marker, n, tail)| {
        marker == ch && n >= width && tail.bytes().all(|b| matches!(b, b' ' | b'\t'))
    })
}
fn opens_fence(line: &str) -> Option<(char, usize)> {
    fence_line(line)
        .and_then(|(ch, width, tail)| (ch != '`' || !tail.contains('`')).then_some((ch, width)))
}

fn section_bounds(text: &str, start: &str, end: &str) -> Result<(usize, usize)> {
    if text.contains('\r') {
        return Err("managed Wiki sections currently require LF line endings".into());
    }
    if text.matches(start).count() != 1 || text.matches(end).count() != 1 {
        return Err("managed Wiki markers must be unique".into());
    }
    let mut offset = 0;
    let mut fence: Option<(char, usize)> = None;
    let mut comment = false;
    let mut first = None;
    let mut last = None;
    for line in text.split_inclusive('\n') {
        let plain = line.trim_end_matches(['\r', '\n']);
        if plain == start || plain == end {
            if fence.is_some() || comment {
                return Err("managed Wiki marker is inside protected text".into());
            }
            if plain == start {
                first = Some(offset + plain.len());
            } else {
                last = Some(offset);
            }
        } else {
            if let Some((ch, width)) = fence {
                if closes_fence(plain, ch, width) {
                    fence = None;
                }
            } else if !comment {
                fence = opens_fence(plain);
            }
            if fence.is_none() {
                let mut rest = plain;
                while let Some(index) = rest.find(if comment { "-->" } else { "<!--" }) {
                    let width = if comment { 3 } else { 4 };
                    rest = &rest[index + width..];
                    comment = !comment;
                }
            }
        }
        offset += line.len();
    }
    match (first, last) {
        (Some(a), Some(b)) if a < b => Ok((a, b)),
        _ => Err("managed Wiki markers must be standalone and ordered".into()),
    }
}

fn replacement(old: &str, new: &str) -> Result<()> {
    if new.contains('\r') {
        return Err("managed replacement requires LF line endings".into());
    }
    let heading = old
        .trim_start_matches('\n')
        .lines()
        .next()
        .ok_or("managed section has no heading")?;
    if !heading.starts_with("## ")
        || heading.len() <= 3
        || new.trim_start_matches('\n').lines().next() != Some(heading)
        || !new.starts_with('\n')
        || !new.ends_with('\n')
    {
        return Err("managed Wiki section must retain its exact H2 role".into());
    }
    let mut fence: Option<(char, usize)> = None;
    let mut headings = 0;
    for line in new.lines() {
        if line.contains('<') || line.contains("-->") {
            return Err("HTML and comments are not allowed in a managed replacement".into());
        }
        let trimmed = line.trim_start();
        if let Some((ch, width)) = fence {
            if closes_fence(line, ch, width) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = opens_fence(line) {
            fence = Some(open);
            continue;
        }
        if trimmed == "---"
            || trimmed.starts_with("# ")
            || trimmed.starts_with("> [!navigation")
            || trimmed.starts_with("> - [Finder]")
            || trimmed.starts_with("cssclasses:")
        {
            return Err("managed replacement cannot inject metadata or navigation".into());
        }
        if trimmed.starts_with("## ") {
            headings += 1;
        }
    }
    if fence.is_some() || headings != 1 {
        return Err("managed replacement must be one balanced section".into());
    }
    Ok(())
}

pub(crate) fn retained_finder_uri(proof: &Value) -> Result<&str> {
    let before = proof["before"].as_str().ok_or("Wiki preimage absent")?;
    let marker = proof["marker"].as_str().ok_or("Wiki marker absent")?;
    let start = format!("<!-- {marker} begin -->");
    let end = format!("<!-- {marker} end -->");
    let (a, _) = section_bounds(before, &start, &end)?;
    before[..a - start.len()]
        .lines()
        .find_map(|line| {
            line.strip_prefix("> - [Finder](")
                .and_then(|s| s.strip_suffix(')'))
        })
        .ok_or_else(|| "retained Finder missing".into())
}

pub(crate) fn transform(owner: &Path, before: &str, data: &Value, proof: &Value) -> Result<String> {
    if data["type"] != "note"
        || data["layer"] != "wiki"
        || data.get("status").is_some()
        || owner.file_name().is_none_or(|s| s != "wiki.md")
        || owner.parent().is_none_or(|p| !p.ends_with("wiki"))
        || proof.as_object().is_none_or(|p| {
            p.keys().any(|k| {
                !["kind", "before", "archive", "marker", "replacement"].contains(&k.as_str())
            })
        })
    {
        return Err("managed section replacement is restricted to maintained Wiki notes".into());
    }
    let marker = proof["marker"].as_str().ok_or("managed marker missing")?;
    if marker.is_empty()
        || marker.len() > 128
        || !marker
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_:".contains(&b))
    {
        return Err("invalid managed marker identity".into());
    }
    let archive =
        crate::maintenance::path(proof["archive"].as_str().ok_or("managed archive missing")?)?;
    let dossier = owner
        .parent()
        .and_then(Path::parent)
        .ok_or("Wiki dossier missing")?;
    let relative = archive
        .strip_prefix(dossier.join("raw"))
        .map_err(|_| "managed archive must remain in the same dossier Raw")?;
    let parts: Vec<_> = relative.iter().collect();
    if parts.len() != 3
        || parts[1] != "_source-assets"
        || !archive.to_string_lossy().ends_with(".md.txt")
        || parts[0]
            .to_str()
            .is_none_or(|p| p.len() < 8 || !p.as_bytes()[..8].iter().all(u8::is_ascii_digit))
    {
        return Err("managed archive must be a dated Raw package source asset".into());
    }
    let start = format!("<!-- {marker} begin -->");
    let end = format!("<!-- {marker} end -->");
    let (a, b) = section_bounds(before, &start, &end)?;
    let prefix = &before[..a - start.len()];
    let body = prefix
        .split_once("\n---\n")
        .ok_or("Wiki frontmatter must precede markers")?
        .1;
    if body.matches("> [!navigation|shortcuts]").count() != 1
        || body
            .lines()
            .filter(|s| s.starts_with("> - [Finder](file:///") && s.ends_with(')'))
            .count()
            != 1
    {
        return Err("complete retained Wiki navigation must precede the managed block".into());
    }
    let new = proof["replacement"]
        .as_str()
        .ok_or("managed replacement missing")?;
    replacement(&before[a..b], new)?;
    Ok(format!("{}{new}{}", &before[..a], &before[b..]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    const OWNER: &str = "topics/example/wiki/wiki.md";
    const BEFORE: &str = "---\ntype: note\ncategory: knowledge\nlayer: wiki\n---\n# Example\n\n> [!navigation|shortcuts]\n> - [[topics/example/example|Back]]\n> - [[topics/example/raw/raw|Raw]]\n> - [Finder](file:///different-host/topics/example/wiki)\n\n<!-- report:v1 begin -->\n## Daily report\n\nOld result.\n<!-- report:v1 end -->\n\n## Retained history\n\nUnchanged history.\n";
    fn proof() -> Value {
        json!({"kind":"managed-wiki-section","before":BEFORE,"archive":"topics/example/raw/20261117-report/_source-assets/before.md.txt","marker":"report:v1","replacement":"\n## Daily report\n\nNew result.\n"})
    }
    fn run(before: &str, proof: &Value) -> Result<String> {
        crate::maintenance::transform(OWNER, &{
            let mut p = proof.clone();
            p["before"] = json!(before);
            p
        })
    }
    #[test]
    fn exact_section_preserves_frontmatter_navigation_markers_and_suffix() {
        assert_eq!(
            run(BEFORE, &proof()).unwrap(),
            BEFORE.replace("Old result.", "New result.")
        );
        let mut p = proof();
        p["archive"] = json!("topics/other/raw/20261117-report/_source-assets/before.md.txt");
        assert!(run(BEFORE, &p).is_err());
        assert!(
            crate::maintenance::transform(
                "topics/example/raw/20261117-source/20261117-source.md",
                &proof()
            )
            .is_err()
        );
        assert!(run(&BEFORE.replace("type: note", "type: task"), &proof()).is_err());
    }
    #[test]
    fn ambiguous_protected_and_out_of_order_markers_refuse() {
        for before in [
            BEFORE.replace("Old result.", "Old result.\n<!-- report:v1 begin -->"),
            BEFORE.replace(
                "<!-- report:v1 begin -->",
                "```text\n<!-- report:v1 begin -->",
            ),
            BEFORE.replace(
                "<!-- report:v1 begin -->",
                "<!-- open\n<!-- report:v1 begin -->",
            ),
            BEFORE.replace("<!-- report:v1 begin -->", " <!-- report:v1 begin -->"),
            BEFORE
                .replace("<!-- report:v1 begin -->", "<!-- report:v1 end -->")
                .replace("Old result.", "Old result.\n<!-- report:v1 begin -->"),
        ] {
            assert!(run(&before, &proof()).is_err(), "{before}");
        }
        for width in 0..=3 {
            for fence in ["```text", "~~~text"] {
                let before = BEFORE.replace(
                    "<!-- report:v1 begin -->",
                    &format!("{}{fence}\n<!-- report:v1 begin -->", " ".repeat(width)),
                );
                assert!(run(&before, &proof()).is_err());
            }
        }
        assert!(run(&BEFORE.replace('\n', "\r\n"), &proof()).is_err());
        for bad_close in ["    ```", "\t```", " \t```", "```\u{a0}"] {
            let before = BEFORE.replace(
                "<!-- report:v1 begin -->",
                &format!("```text\n{bad_close}\n<!-- report:v1 begin -->"),
            );
            assert!(run(&before, &proof()).is_err());
        }
    }
    #[test]
    fn replacements_cannot_swallow_suffix_or_inject_metadata_navigation_or_new_role() {
        for replacement in [
            "\n## Different role\nNew.\n",
            "\n## Daily report\n```text\nunclosed\n",
            "\n## Daily report\n<!-- open\n",
            "\n## Daily report\n<div>\n",
            "\n## Daily report\n---\nstatus: done\n",
            "\n## Daily report\n> [!navigation|shortcuts]\n",
            "\n## Daily report\n> - [Finder](file:///wrong)\n",
            "\n## Daily report\n## Other section\n",
            "\n    ## Daily report\nNew.\n",
            "\n## Daily report\n    ```text\n# Injected H1\n```\n",
            "\n## Daily report\n\t```text\n# Injected H1\n```\n",
            "\n## Daily report\n \t```text\n# Injected H1\n```\n",
            "\n## Daily report\n```text\n \t```\n",
            "\n## Daily report\n```text\n    ```\n",
        ] {
            let mut p = proof();
            p["replacement"] = json!(replacement);
            assert!(run(BEFORE, &p).is_err(), "{replacement}");
        }
    }
}
