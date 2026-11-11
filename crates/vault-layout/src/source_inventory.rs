use crate::{event_header, raw_processing, read_metadata, strings, text};
use serde_json::Value;
use std::{
    fs, io,
    path::{Component, Path},
};

// An inventory preserves provenance. It is not a substitute for its source events.
pub(crate) fn check(
    vault: &Path,
    package: &Path,
    data: &Value,
    body: &str,
) -> io::Result<Vec<String>> {
    let mut errors = Vec::new();
    let rows = inventory_rows(body);
    if !rows.is_empty() {
        let raw = package.parent().expect("Raw package parent");
        let mut sources = Vec::new();
        for entry in fs::read_dir(raw)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() || entry.path() == package {
                continue;
            }
            let path = entry
                .path()
                .join(format!("{}.md", entry.file_name().to_string_lossy()));
            if let Some((metadata, body)) = read_metadata(&path)? {
                sources.push((path, metadata, body));
            }
        }
        for (name, hash) in rows {
            let matches: Vec<_> = sources
                .iter()
                .filter(|(_, source, _)| {
                    text(source, "source_path") == Some(name.as_str())
                        && text(source, "source_sha256") == Some(hash.as_str())
                })
                .collect();
            if matches.len() != 1 {
                errors.push(format!("{name}: require exactly one sibling Raw event with matching source_path and source_sha256"));
            } else {
                let (note, metadata, body) = matches[0];
                let inventory_note = package.join(package.file_name().unwrap());
                let binding = format!(
                    "[[{}]]",
                    inventory_note.strip_prefix(vault).unwrap().display()
                );
                if text(metadata, "source_inventory") != Some(binding.as_str()) {
                    errors.push(format!(
                        "{name}: source event must point back to this inventory"
                    ));
                }
                if let Err(message) =
                    event_header::check(vault, note.parent().unwrap(), metadata, body)
                {
                    errors.push(format!("{name}: {message}"));
                }
                if !strings(&metadata["cssclasses"]).any(|class| class == "vault-event-v1-1") {
                    errors.push(format!("{name}: source event must use Event Note v1.1"));
                }
                for message in source_evidence(vault, note.parent().unwrap(), metadata, body)? {
                    errors.push(format!("{name}: {message}"));
                }
            }
        }
    }
    if data.get("source_inventory").is_some() {
        errors.extend(source_evidence(vault, package, data, body)?);
        let bound = text(data, "source_inventory")
            .and_then(|link| link.strip_prefix("[[")?.strip_suffix("]]"))
            .filter(|link| !link.contains(['#', '|']))
            .and_then(|link| local_inventory(vault, package.parent()?, link));
        if !bound.is_some_and(|(_, inventory)| {
            inventory_rows(&inventory).iter().any(|(name, hash)| {
                Some(name.as_str()) == text(data, "source_path")
                    && Some(hash.as_str()) == text(data, "source_sha256")
            })
        }) {
            errors.push("source_inventory must bind this source path and hash to an inventory in the same Raw".into());
        }
    }
    Ok(errors)
}

fn inventory_rows(body: &str) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    let mut table = false;
    let unquoted = body
        .lines()
        .map(|line| {
            let mut line = line.trim();
            while let Some(inner) = line.strip_prefix('>') {
                line = inner.trim_start();
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n");
    for line in crate::index_navigation::visible_blocks(&unquoted).lines() {
        let line = line.trim();
        let cells: Vec<_> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cells == ["Bronpad", "Leesbare note", "Origineel"] {
            table = true;
            continue;
        }
        if !line.contains('|') {
            table = false;
        }
        if !table
            || line
                .trim_matches('|')
                .chars()
                .all(|c| matches!(c, '-' | ':' | '|' | ' '))
        {
            continue;
        }
        let name = cells.first().copied().unwrap_or("");
        let hash = line.rsplit_once("SHA-256").and_then(|(_, tail)| {
            tail.split(|c: char| !c.is_ascii_hexdigit())
                .find(|part| part.len() == 64)
        });
        rows.push((name.to_owned(), hash.unwrap_or("").to_owned()));
    }
    rows
}

fn local_file(package: &Path, relative: &str) -> Option<std::path::PathBuf> {
    let relative = Path::new(relative);
    if relative
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
    {
        return None;
    }
    let mut path = package.to_path_buf();
    for part in relative.components() {
        path.push(part);
        if fs::symlink_metadata(&path).ok()?.file_type().is_symlink() {
            return None;
        }
    }
    path.is_file().then_some(path)
}

fn source_evidence(
    vault: &Path,
    package: &Path,
    data: &Value,
    source_body: &str,
) -> io::Result<Vec<String>> {
    let mut errors = Vec::new();
    if text(data, "source_path").is_none() {
        errors.push("record the original source_path".into());
    }
    let source = text(data, "source_file").and_then(|file| local_file(package, file));
    match source {
        Some(path) => {
            let (hash, _) = crate::mail_manifest::hash_file(&path)
                .map_err(|error| io::Error::other(error.to_string()))?;
            if text(data, "source_sha256") != Some(hash.as_str()) {
                errors.push("local source_file does not match source_sha256".into());
            }
        }
        None => {
            errors.push("preserve a regular local source_file inside its own event package".into())
        }
    }
    let read = text(data, "source_read_status");
    if !matches!(read, Some("read" | "inspected")) {
        errors.push("source content still needs individual reading or visual inspection".into());
    }
    if read == Some("read")
        && text(data, "source_extract")
            .and_then(|file| local_file(package, file))
            .and_then(|path| fs::read_to_string(path).ok())
            .is_none_or(|text| text.trim().is_empty())
    {
        errors.push("read source needs a nonempty local source_extract".into());
    }
    if let Some(expected) = text(data, "source_extract_sha256") {
        let actual = text(data, "source_extract")
            .and_then(|file| local_file(package, file))
            .and_then(|path| crate::mail_manifest::hash_file(&path).ok());
        if actual.is_none_or(|(hash, _)| hash != expected) {
            errors.push("source_extract no longer matches its recorded extraction hash".into());
        }
    }
    if text(data, "source_review").is_none() {
        errors.push("record the individual source assessment in source_review".into());
    }
    if text(data, "wiki_status") != Some("processed") {
        errors.push("source has not yet been processed into the dossier Wiki".into());
        return Ok(errors);
    }
    let processed = text(data, "wiki_processed_on");
    if !processed.is_some_and(crate::iso_date)
        || text(data, "updated").is_some_and(|updated| {
            crate::iso_date(updated) && processed.is_some_and(|processed| updated > processed)
        })
    {
        errors.push("record a valid wiki_processed_on at least as recent as updated".into());
    }
    let citation = package.strip_prefix(vault).unwrap().join(format!(
        "{}",
        package.file_name().unwrap().to_string_lossy()
    ));
    let target = text(data, "wiki_target").and_then(raw_processing::target);
    if !target.is_some_and(|(note, anchor)| {
        let Some(wiki) = local_file(vault, &format!("{}.md", note.trim_end_matches(".md"))) else {
            return false;
        };
        let own_wiki = package.parent().unwrap().parent().unwrap().join("wiki");
        if !wiki.starts_with(own_wiki) {
            return false;
        }
        let Ok(body) = fs::read_to_string(wiki) else {
            return false;
        };
        raw_processing::check_event(source_body, note, anchor, &body).is_ok()
            && wiki_cites(&body, anchor, &citation.to_string_lossy())
    }) {
        errors.push("exact Wiki target section must cite this source event".into());
    }
    Ok(errors)
}

fn local_inventory(vault: &Path, raw: &Path, link: &str) -> Option<(Value, String)> {
    let path = vault.join(format!("{}.md", link.trim_end_matches(".md")));
    if path.parent()?.parent()? != raw || path.file_stem()? != path.parent()?.file_name()? {
        return None;
    }
    if path
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return None;
    }
    if fs::symlink_metadata(path.parent()?)
        .ok()?
        .file_type()
        .is_symlink()
    {
        return None;
    }
    read_metadata(&path).ok()?
}

fn wiki_cites(body: &str, anchor: &str, citation: &str) -> bool {
    use crate::shortcut_links::{anchor_count, decode_percent, heading};
    if anchor_count(body, anchor) != 1 {
        return false;
    }
    let visible = crate::index_navigation::visible_body(body);
    let anchor = decode_percent(anchor);
    let mut level = None;
    let mut previous = None;
    let mut section = String::new();
    for line in visible.lines() {
        if let Some((depth, title)) = heading(line, previous) {
            if level.is_some_and(|level| depth <= level) {
                break;
            }
            if title == anchor {
                level = Some(depth);
            }
        } else if level.is_some() {
            section.push_str(line);
            section.push('\n');
        }
        previous = (!line.trim().is_empty()).then_some(line);
    }
    crate::index_navigation::wiki_links(&section).any(|link| {
        link.split(['#', '|'])
            .next()
            .map(|path| path.trim_end_matches('\\'))
            == Some(citation)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_register_is_detected_without_opt_in_and_keeps_duplicate_hashes() {
        let hash = "a".repeat(64);
        let body = format!(
            "| Bronpad | Leesbare note | Origineel |\n|---|---|---|\n| one.pdf | Origineel bestand | SHA-256 `{hash}` |\n| two.pdf | Origineel bestand | SHA-256 `{hash}` |\n"
        );
        assert_eq!(
            inventory_rows(&body),
            vec![("one.pdf".into(), hash.clone()), ("two.pdf".into(), hash)]
        );
        assert!(inventory_rows(&format!("```markdown\n{body}```")).is_empty());
        assert!(inventory_rows(&format!("````markdown\n```\n{body}````\n")).is_empty());
        assert!(inventory_rows(&format!("<!--\n{body}-->\n")).is_empty());
        let unwrapped = body
            .lines()
            .map(|line| line.trim_matches('|'))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(inventory_rows(&unwrapped), inventory_rows(&body));
        let nested = body
            .lines()
            .map(|line| format!("> > {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(inventory_rows(&nested), inventory_rows(&body));
        assert!(inventory_rows(&format!("> > ```markdown\n{nested}\n> > ```\n")).is_empty());
        assert_eq!(
            inventory_rows(
                "|Bronpad|Leesbare note|Origineel|\n|---|---|---|\n|missing.pdf|local|missing hash|\n"
            ),
            vec![("missing.pdf".into(), String::new())]
        );
    }

    #[test]
    fn wiki_citation_must_belong_to_the_visible_target_section() {
        let link = "topics/dossier/raw/20261111-source/20261111-source";
        assert!(wiki_cites(
            &format!("# Wiki\n\n### Doel\n\n[[{link}]]\n"),
            "Doel",
            link
        ));
        assert!(wiki_cites(
            &format!("## Doel\n\n| Bewijs |\n|---|\n| [[{link}\\|Leesbare bron]] |\n"),
            "Doel",
            link
        ));
        assert!(wiki_cites(
            &format!("# Wiki\n\nMijn doel\n---------\n\n[[{link}]]\n"),
            "Mijn%20doel",
            link
        ));
        for body in [
            format!("# Wiki\n\n## Doel\n\n### Verdieping\n\n```markdown\n[[{link}]]\n```\n"),
            format!("# Wiki\n\n## Doel\n\n<!-- [[{link}]] -->\n"),
            format!("# Wiki\n\n## Doel\n\nTekst\n\n# Volgende\n\n[[{link}]]\n"),
            format!("# Wiki\n\n```markdown\n## Doel\n\n[[{link}]]\n```\n\n## Doel\n\nTekst\n"),
        ] {
            assert!(!wiki_cites(&body, "Doel", link), "{body}");
        }
    }

    #[test]
    fn full_inventory_checks_real_source_bytes_reading_and_wiki_evidence() {
        use serde_json::json;
        use sha2::{Digest, Sha256};
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let vault = crate::test_root().join(".tmp").join(format!(
            "inventory-proof-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let raw = vault.join("topics/dossier/raw");
        let inventory = raw.join("20261111-inventory");
        let package = raw.join("20261111-source");
        fs::create_dir_all(&inventory).unwrap();
        fs::create_dir_all(&package).unwrap();
        let wiki = vault.join("topics/dossier/wiki/wiki.md");
        fs::create_dir_all(wiki.parent().unwrap()).unwrap();
        let hash = format!("{:x}", Sha256::digest(b"original source"));
        let body = format!(
            "|Bronpad|Leesbare note|Origineel|\n|---|---|---|\n|one.txt|source|SHA-256 `{hash}`|\n"
        );
        assert_eq!(
            check(&vault, &inventory, &json!({}), &body).unwrap().len(),
            1
        );
        let source_body = format!(
            "# Source\n\n> [!navigation|shortcuts]\n> - [[topics/dossier/raw/raw|Back]]\n> - [Finder](file://{})\n\n> [!wiki-facts|dossier]\n> - Dossier **[[topics/dossier/dossier|Dossier]]**\n\n```vault-event-detail\n```\n\n> [!event-context] Context\n> Original source read.\n\n## Activity\n\n> [!activity-day|2026-11-11] 11 november 2026\n\n> [!activity-message|codex] Codex\n> **Verwerking in Wiki**\n> Contractvoorwaarden en ontvangstdatum zijn gelezen en verwerkt in [[topics/dossier/wiki/wiki#Doel]].\n>\n> *Codex*\n",
            package.display()
        );
        let mut metadata = json!({"type":"reference","category":"source","created":"2026-11-11","cssclasses":["vault-navigation","vault-dashboard","vault-event","vault-event-v1-1"],"source_path":"one.txt","source_sha256":hash,"source_file":"original.txt","source_extract":"extract.txt","source_read_status":"read","source_review":"Contract read and dates checked","source_inventory":"[[topics/dossier/raw/20261111-inventory/20261111-inventory]]","wiki_status":"processed","wiki_target":"[[topics/dossier/wiki/wiki#Doel]]"});
        metadata["wiki_processed_on"] = json!("2026-11-11");
        fs::write(package.join("original.txt"), b"original source").unwrap();
        fs::write(package.join("extract.txt"), "original source").unwrap();
        fs::write(&wiki,"# Wiki\n\n## Doel\n\nContractvoorwaarden. [[topics/dossier/raw/20261111-source/20261111-source]]\n").unwrap();
        let note = package.join("20261111-source.md");
        let save = |data: &Value| {
            fs::write(
                &note,
                format!(
                    "---\n{}\n---\n\n{source_body}",
                    serde_json::to_string(data).unwrap()
                ),
            )
            .unwrap()
        };
        save(&metadata);
        assert!(
            check(&vault, &inventory, &json!({}), &body)
                .unwrap()
                .is_empty()
        );
        for (key, value) in [
            ("source_sha256", json!("b".repeat(64))),
            ("source_file", json!("../external.txt")),
            ("source_read_status", json!("pending")),
            ("wiki_status", json!("pending")),
            ("wiki_processed_on", Value::Null),
            ("updated", json!("2026-11-12")),
            ("source_extract_sha256", json!("b".repeat(64))),
            ("source_inventory", Value::Null),
        ] {
            let mut bad = metadata.clone();
            bad[key] = value;
            save(&bad);
            assert!(
                !check(&vault, &inventory, &json!({}), &body)
                    .unwrap()
                    .is_empty(),
                "{key}"
            );
        }
        save(&metadata);
        let other_wiki = vault.join("topics/other/wiki/wiki.md");
        fs::create_dir_all(other_wiki.parent().unwrap()).unwrap();
        fs::copy(&wiki, &other_wiki).unwrap();
        let mut outside = metadata.clone();
        outside["wiki_target"] = json!("[[topics/other/wiki/wiki#Doel]]");
        save(&outside);
        assert!(
            !check(&vault, &inventory, &json!({}), &body)
                .unwrap()
                .is_empty()
        );
        save(&metadata);
        fs::write(package.join("original.txt"), "changed source").unwrap();
        assert!(
            !check(&vault, &inventory, &json!({}), &body)
                .unwrap()
                .is_empty()
        );
        fs::write(package.join("original.txt"), "original source").unwrap();
        fs::write(&wiki, "# Wiki\n\n## Doel\n\nNo citation\n").unwrap();
        assert!(
            !check(&vault, &inventory, &json!({}), &body)
                .unwrap()
                .is_empty()
        );
        fs::remove_file(package.join("original.txt")).unwrap();
        std::os::unix::fs::symlink(package.join("extract.txt"), package.join("original.txt"))
            .unwrap();
        assert!(
            !check(&vault, &inventory, &json!({}), &body)
                .unwrap()
                .is_empty()
        );
        metadata["source_file"] = Value::Null;
        assert!(
            !source_evidence(&vault, &package, &metadata, &source_body)
                .unwrap()
                .is_empty()
        );
        fs::remove_dir_all(vault).unwrap();
    }
}
