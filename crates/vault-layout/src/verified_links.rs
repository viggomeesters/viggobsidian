//! Typed link-only maintenance. Authority and publication remain with callers.
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) fn repair(before: &str, mapping: &Value) -> Result<String> {
    let rows = mapping
        .as_object()
        .filter(|m| !m.is_empty() && m.len() <= 100)
        .ok_or("verified links need 1..100 exact targets")?;
    for (old, replacement) in rows {
        if old.is_empty() || old.contains(['[', ']', '|', '`', '\n', '\r']) {
            return Err("invalid original WikiLink target".into());
        }
        if replacement.as_object().is_none() {
            return Err("typed link destination required".into());
        }
        match replacement["kind"].as_str() {
            Some("file") => {
                if replacement
                    .as_object()
                    .unwrap()
                    .keys()
                    .any(|k| !["kind", "uri", "root", "sha256"].contains(&k.as_str()))
                {
                    return Err("unknown file destination field".into());
                }
                let uri = replacement["uri"].as_str().ok_or("file URI absent")?;
                if !uri.starts_with("file:///")
                    || uri.contains(['\n', '\r', ' ', '`', '(', ')', '<', '>'])
                {
                    return Err("exact local file URI required".into());
                }
                if replacement["root"].as_str().is_none() || !hash(&replacement["sha256"]) {
                    return Err("file destination needs explicit root and byte hash".into());
                }
            }
            Some("git") => {
                if replacement
                    .as_object()
                    .unwrap()
                    .keys()
                    .any(|k| !["kind", "revision", "path", "sha256"].contains(&k.as_str()))
                {
                    return Err("unknown historical destination field".into());
                }
                let rev = replacement["revision"]
                    .as_str()
                    .ok_or("Git revision absent")?;
                if rev.len() != 40
                    || !rev.bytes().all(|b| b.is_ascii_hexdigit())
                    || !hash(&replacement["sha256"])
                {
                    return Err("exact Git object and byte hash required".into());
                }
                crate::maintenance::path(replacement["path"].as_str().ok_or("Git path absent")?)?;
            }
            _ => return Err("unsupported verified destination".into()),
        }
    }
    let mut used = BTreeSet::new();
    let mut result = String::new();
    let mut fence = None;
    let mut metadata = 0;
    let mut comment = false;
    for line in before.split_inclusive('\n') {
        if metadata < 2 {
            if line.trim_end() == "---" {
                metadata += 1;
            }
            result.push_str(line);
            continue;
        }
        let content = link_style_core::quote_content(line);
        let trimmed = content.trim_start_matches(' ');
        let indent = content.len() - trimmed.len();
        let marker = trimmed.as_bytes().first().copied().unwrap_or(0);
        let width = trimmed.bytes().take_while(|b| *b == marker).count();
        if let Some((opening, length)) = fence {
            if indent <= 3
                && marker == opening
                && width >= length
                && trimmed[width..].trim().is_empty()
            {
                fence = None;
            }
            result.push_str(line);
            continue;
        }
        if indent <= 3 && matches!(marker, b'`' | b'~') && width >= 3 {
            fence = Some((marker, width));
            result.push_str(line);
            continue;
        }
        if indent >= 4 || content.starts_with('\t') {
            result.push_str(line);
            continue;
        }
        if comment || line.contains("<!--") {
            update_comment(line, &mut comment);
            result.push_str(line);
            continue;
        }
        // Only complete WikiLink tokens are changed; surrounding human prose,
        // source quotation markers, aliases and every fenced example stay exact.
        let mut rest = line;
        while let Some(start) = rest.find("[[") {
            let Some(close) = rest[start + 2..].find("]]").map(|i| start + 2 + i) else {
                break;
            };
            let inside = &rest[start + 2..close];
            let (target, alias) = inside
                .split_once('|')
                .map_or((inside, None), |(t, a)| (t, Some(a)));
            result.push_str(&rest[..start]);
            if let Some(to) = rows.get(target) {
                if rest[..start].contains('`')
                    || rest[..start].ends_with('!')
                    || inside.contains(['[', ']', '\\'])
                    || line.trim_start().starts_with('<')
                {
                    return Err("complex inline/source link is outside verified repair".into());
                }
                let label = alias.unwrap_or(target);
                if label.contains(['[', ']', '`', '\\']) {
                    return Err("unsafe preserved link label".into());
                }
                if to["kind"] == "file" {
                    result.push_str(&format!("[{label}]({})", to["uri"].as_str().unwrap()));
                } else {
                    result.push_str(&format!(
                        "`{label}` (historische Git-bron: `{}:{}`)",
                        to["revision"].as_str().unwrap(),
                        to["path"].as_str().unwrap()
                    ));
                }
                used.insert(target.to_owned());
            } else {
                result.push_str(&rest[start..close + 2]);
            }
            rest = &rest[close + 2..];
        }
        result.push_str(rest);
    }
    if metadata != 2 || fence.is_some() || comment || used.len() != rows.len() {
        return Err("incomplete metadata/fence or unused verified mapping".into());
    }
    Ok(result)
}
fn hash(value: &Value) -> bool {
    value
        .as_str()
        .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
}

pub(crate) fn final_activity(source: &str) -> Result<bool> {
    let mut fence = None;
    let mut comment = false;
    for line in source.lines() {
        if comment {
            update_comment(line, &mut comment);
            continue;
        }
        let content = link_style_core::quote_content(line);
        let trimmed = content.trim_start_matches(' ');
        let indent = content.len() - trimmed.len();
        let marker = trimmed.as_bytes().first().copied().unwrap_or(0);
        let width = trimmed.bytes().take_while(|b| *b == marker).count();
        match fence {
            Some((opening, length))
                if indent <= 3
                    && marker == opening
                    && width >= length
                    && trimmed[width..].trim().is_empty() =>
            {
                fence = None
            }
            None if indent <= 3 && matches!(marker, b'`' | b'~') && width >= 3 => {
                fence = Some((marker, width))
            }
            _ => {}
        }
        if fence.is_none() && !(matches!(marker, b'`' | b'~') && width >= 3) {
            update_comment(line, &mut comment);
        }
    }
    if fence.is_some() || comment {
        return Err("unclosed source fence before Activity addition".into());
    }
    let mut activity = false;
    for line in link_style_core::visible_blocks(source).lines() {
        let leading = line.trim_start_matches(' ');
        if line.len() - leading.len() > 3 {
            continue;
        }
        let trimmed = leading.trim_end();
        if trimmed == "## Activity" {
            if activity {
                return Err("multiple Activity sections".into());
            }
            activity = true;
        } else if activity && (trimmed.starts_with("# ") || trimmed.starts_with("## ")) {
            return Err("Activity must be the final main section".into());
        }
    }
    Ok(activity)
}
fn update_comment(mut line: &str, open: &mut bool) {
    loop {
        let marker = if *open { "-->" } else { "<!--" };
        let Some((_, after)) = line.split_once(marker) else {
            break;
        };
        *open = !*open;
        line = after;
    }
}

pub(crate) fn bindings(vault: &Path, proofs: &BTreeMap<PathBuf, Value>) -> Result<()> {
    for (path, proof) in proofs {
        if matches!(
            proof["kind"].as_str(),
            Some("verified-link-repair" | "activity-append")
        ) {
            let archive = Path::new(proof["archive"].as_str().ok_or("archive absent")?);
            let archived = proofs
                .get(archive)
                .ok_or("archive must be in the same exact write set")?;
            if archived["kind"] != "source-preimage-archive"
                || archived["source"] != path.to_str().ok_or("invalid source path")?
                || archived["source_before"] != proof["before"]
            {
                return Err("archive does not preserve the exact owner preimage".into());
            }
        }
        if proof["kind"] == "source-preimage-archive" {
            let source = Path::new(proof["source"].as_str().ok_or("archive source absent")?);
            let owner = proofs.get(source).ok_or("archive cannot stand alone")?;
            if !matches!(
                owner["kind"].as_str(),
                Some("verified-link-repair" | "activity-append")
            ) || owner["archive"] != path.to_str().unwrap_or("")
            {
                return Err("archive is not paired to its maintenance owner".into());
            }
        }
        if proof["kind"] != "verified-link-repair" {
            continue;
        }
        for (old, to) in proof["mapping"].as_object().ok_or("mapping absent")? {
            let bytes = if to["kind"] == "file" {
                let root = Path::new(to["root"].as_str().ok_or("explicit file root absent")?);
                if !root.is_absolute() || root.canonicalize()? != root || !root.is_dir() {
                    return Err("invalid explicit file root".into());
                }
                let uri = to["uri"].as_str().ok_or("URI absent")?;
                let (uri, fragment) = uri
                    .split_once('#')
                    .map_or((uri, None), |(a, b)| (a, Some(b)));
                let decoded = crate::shortcut_links::decode_percent(
                    uri.strip_prefix("file://").ok_or("file URI required")?,
                );
                let destination = Path::new(&decoded);
                let relative = destination
                    .strip_prefix(root)
                    .map_err(|_| "file target exceeds its explicitly supplied root")?;
                let mut cursor = root.to_path_buf();
                for part in relative.components() {
                    if !matches!(part, std::path::Component::Normal(_)) {
                        return Err("unsafe external target".into());
                    }
                    cursor.push(part);
                    if fs::symlink_metadata(&cursor)?.is_symlink() {
                        return Err("external target contains a symlink".into());
                    }
                }
                if !cursor.is_file() || fs::metadata(&cursor)?.len() > 20 * 1024 * 1024 {
                    return Err("file target is not a bounded regular source".into());
                }
                if old.split_once('#').map(|(_, a)| a)
                    != fragment
                        .map(crate::shortcut_links::decode_percent)
                        .as_deref()
                {
                    return Err("file anchor must remain exact".into());
                }
                fs::read(cursor)?
            } else {
                let revision = to["revision"].as_str().ok_or("revision absent")?;
                let source = to["path"].as_str().ok_or("Git source absent")?;
                let object = format!("{revision}:{source}");
                let size = Command::new("git")
                    .arg("-C")
                    .arg(vault)
                    .args(["cat-file", "-s", &object])
                    .output()?;
                if !size.status.success()
                    || std::str::from_utf8(&size.stdout)?.trim().parse::<u64>()? > 20 * 1024 * 1024
                {
                    return Err("historical Git source missing or too large".into());
                }
                let output = Command::new("git")
                    .arg("-C")
                    .arg(vault)
                    .args(["cat-file", "blob", &object])
                    .output()?;
                if !output.status.success() {
                    return Err("historical Git object unavailable".into());
                }
                output.stdout
            };
            if crate::adoption::hash(&bytes) != to["sha256"] {
                return Err("source target bytes changed".into());
            }
            if let Some((_, anchor)) = old.split_once('#') {
                let text = std::str::from_utf8(&bytes)?;
                if !text.lines().any(|line| {
                    line.starts_with('#') && line.trim_start_matches('#').trim() == anchor
                }) {
                    return Err("verified source anchor missing".into());
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    const OWNER: &str = "30-topics/tools/commands/commands.md";
    const ARCHIVE: &str = "30-topics/tools/commands/before.md.txt";
    const BEFORE: &str = "---\ntype: reference\ncategory: guide\n---\n# Commands\n\nHuman source [[old-note]].\n\n> Original quote [[old-note]].\n\n```md\n[[old-note]]\n```\n";
    fn destination() -> Value {
        json!({"kind":"file","uri":"file:///explicit-work/source.md","root":"/explicit-work","sha256":"a".repeat(64)})
    }
    fn proof() -> Value {
        json!({"kind":"verified-link-repair","before":BEFORE,"archive":ARCHIVE,"mapping":{"old-note":destination()}})
    }
    #[test]
    fn exact_link_tokens_and_quotes_keep_every_other_byte_and_fenced_example() {
        let after = crate::maintenance::transform(OWNER, &proof()).unwrap();
        assert_eq!(
            after,
            BEFORE.replacen(
                "[[old-note]]",
                "[old-note](file:///explicit-work/source.md)",
                2
            )
        );
        let mut invalid = proof();
        invalid["mapping"]["unused"] = destination();
        assert!(crate::maintenance::transform(OWNER, &invalid).is_err());
        invalid = proof();
        invalid["extra"] = json!("body rewrite");
        assert!(crate::maintenance::transform(OWNER, &invalid).is_err());
        invalid = proof();
        invalid["mapping"]["old-note"]["uri"] = json!("https://example.invalid/source");
        assert!(crate::maintenance::transform(OWNER, &invalid).is_err());
    }
    #[test]
    fn historical_locators_are_explicit_inert_and_keep_the_original_anchor_name() {
        let before = BEFORE.replace("[[old-note]]", "[[old-note#Status]]");
        let p = json!({"kind":"verified-link-repair","before":before,"archive":ARCHIVE,"mapping":{"old-note#Status":{"kind":"git","revision":"b".repeat(40),"path":"30-topics/history/source.md","sha256":"a".repeat(64)}}});
        let after = crate::maintenance::transform(OWNER, &p).unwrap();
        assert!(after.contains("`old-note#Status` (historische Git-bron: `"));
        assert!(!after.contains(".md.txt#Status"));
        assert!(after.contains("```md\n[[old-note#Status]]\n```"));
    }
    #[test]
    fn nested_quoted_fences_indented_code_and_embeds_are_never_rewritten_as_links() {
        for protected in [
            "> ```md\n> [[old-note]]\n> ```\n",
            "> > ~~~md\n> > [[old-note]]\n> > ~~~\n",
            "    [[old-note]]\n",
            "\t[[old-note]]\n",
        ] {
            let original = format!("{BEFORE}\n{protected}");
            let mut p = proof();
            p["before"] = json!(original);
            let after = crate::maintenance::transform(OWNER, &p).unwrap();
            assert!(after.ends_with(protected), "{protected}");
        }
        let mut p = proof();
        p["before"] = json!(format!("{BEFORE}\n![[old-note]]\n"));
        assert!(crate::maintenance::transform(OWNER, &p).is_err());
    }
    #[test]
    fn archives_are_mandatory_paired_exact_and_not_free_standalone_payloads() {
        let p = proof();
        let mut map = BTreeMap::from([(PathBuf::from(OWNER), p.clone())]);
        assert!(
            bindings(Path::new("/missing"), &map)
                .unwrap_err()
                .to_string()
                .contains("same exact write set")
        );
        let archived = json!({"kind":"source-preimage-archive","before":null,"source":OWNER,"source_before":BEFORE});
        assert_eq!(
            crate::maintenance::transform(ARCHIVE, &archived).unwrap(),
            BEFORE
        );
        map.insert(PathBuf::from(ARCHIVE), archived.clone());
        map.get_mut(Path::new(ARCHIVE)).unwrap()["source_before"] = json!("foreign");
        assert!(
            bindings(Path::new("/missing"), &map)
                .unwrap_err()
                .to_string()
                .contains("exact owner preimage")
        );
        map.remove(Path::new(OWNER));
        assert!(bindings(Path::new("/missing"), &map).is_err());
        assert!(crate::maintenance::transform("other.md", &archived).is_err());
    }
    #[test]
    fn activity_additions_use_existing_provenance_gate_without_free_prose_append() {
        let p = json!({"kind":"activity-append","before":BEFORE,"archive":ARCHIVE,"append":"\n\n## Activity\n\n> [!activity-day|2026-11-16] 16 november 2026\n\n> [!activity-message|assistant] Codex\n> <!-- viggobsidian:activity:v1 kind=result goal=source-repair -->\n> Repaired source links.\n>\n> *09:00 · Codex · maintenance*\n"});
        assert!(
            crate::maintenance::transform(OWNER, &p)
                .unwrap()
                .starts_with(BEFORE)
        );
        let mut invalid = p.clone();
        invalid["append"] = json!("\n\nArbitrary prose\n");
        assert!(crate::maintenance::transform(OWNER, &invalid).is_err());
        invalid = p.clone();
        invalid["append"] = json!(p["append"].as_str().unwrap().replace("09:00", "29:99"));
        assert!(crate::maintenance::transform(OWNER, &invalid).is_err());
        invalid = p.clone();
        invalid["before"] = json!(format!("{BEFORE}\n```unclosed\n"));
        assert!(crate::maintenance::transform(OWNER, &invalid).is_err());
        let mut fictitious = p.clone();
        fictitious["before"] = json!(format!("{BEFORE}\n```md\n## Activity\n```\n"));
        fictitious["append"] = json!(
            p["append"]
                .as_str()
                .unwrap()
                .replace("\n\n## Activity\n", "\n")
        );
        assert!(crate::maintenance::transform(OWNER, &fictitious).is_err());
        let mut after_section = p.clone();
        after_section["before"] = json!(format!("{BEFORE}\n## Activity\n\n## Later section\n"));
        assert!(crate::maintenance::transform(OWNER, &after_section).is_err());
        let mut comment = p.clone();
        comment["before"] = json!(format!("{BEFORE}\n<!-- unclosed original comment\n"));
        assert!(crate::maintenance::transform(OWNER, &comment).is_err());
        let mut trailing = p.clone();
        trailing["before"] = json!(format!("{BEFORE}\n## Activity  \n"));
        trailing["append"] = json!(
            p["append"]
                .as_str()
                .unwrap()
                .replace("\n\n## Activity\n", "\n\n")
        );
        assert!(crate::maintenance::transform(OWNER, &trailing).is_ok());
    }
    #[test]
    fn explicit_read_only_file_root_hash_anchor_and_symlinks_are_checked() {
        let temp = std::env::var_os("VIGGO_TEST_TMP")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let base = temp.join(format!("verified-links-{}", std::process::id()));
        fs::create_dir(&base).unwrap();
        let base = base.canonicalize().unwrap();
        let file = base.join("source.md");
        fs::write(&file, "# Status\nHuman source\n").unwrap();
        let mut p = proof();
        p["mapping"]["old-note"] = json!({"kind":"file","root":base,"uri":format!("file://{}",file.display()),"sha256":crate::adoption::hash(&fs::read(&file).unwrap())});
        let archive = json!({"kind":"source-preimage-archive","before":null,"source":OWNER,"source_before":BEFORE});
        let map = |p: Value| {
            BTreeMap::from([
                (PathBuf::from(OWNER), p),
                (PathBuf::from(ARCHIVE), archive.clone()),
            ])
        };
        assert!(bindings(&base, &map(p.clone())).is_ok());
        let mut bad = p.clone();
        bad["mapping"]["old-note"]["sha256"] = json!("0".repeat(64));
        assert!(bindings(&base, &map(bad)).is_err());
        bad = p.clone();
        bad["mapping"]["old-note"]["root"] = json!(base.join("elsewhere"));
        assert!(bindings(&base, &map(bad)).is_err());
        #[cfg(unix)]
        {
            let link = base.join("alias.md");
            std::os::unix::fs::symlink(&file, &link).unwrap();
            let mut bad = p.clone();
            bad["mapping"]["old-note"]["uri"] = json!(format!("file://{}", link.display()));
            assert!(bindings(&base, &map(bad)).is_err());
            fs::remove_file(link).unwrap();
        }
        fs::remove_file(file).unwrap();
        fs::remove_dir(base).unwrap();
    }
}
