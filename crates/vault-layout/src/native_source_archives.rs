//! Frozen producer bundles are source payloads, not new operational notes.
use crate::{mail_manifest, read_metadata, text};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Component, Path, PathBuf},
};
type Result<T> = std::result::Result<T, String>;

pub(crate) fn check(package: &Path, data: &Value) -> io::Result<(BTreeSet<PathBuf>, Vec<String>)> {
    if data.get("source_archives").is_none() {
        return Ok((BTreeSet::new(), vec![]));
    }
    let mut accepted = BTreeSet::new();
    let result = (|| -> Result<()> {
        if data["source_archives"] != "source-archives.json" {
            return Err("source_archives must name source-archives.json".into());
        }
        let index = package.join("source-archives.json");
        regular(&index)?;
        let v: Value = serde_json::from_slice(&fs::read(index).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        keys(&v, &["schema", "package_note", "archives"])?;
        let note = format!(
            "{}.md",
            package
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or("invalid package name")?
        );
        if v["schema"] != "viggobsidian.native-source-archives.v1" || v["package_note"] != note {
            return Err("source archive inventory must bind its canonical package note".into());
        }
        let archives = v["archives"]
            .as_array()
            .filter(|a| !a.is_empty())
            .ok_or("source archives array required")?;
        for archive in archives {
            keys(
                archive,
                &[
                    "directory",
                    "producer",
                    "capture_note",
                    "files",
                    "directories",
                    "origin_path",
                ],
            )?;
            let relative = archive["directory"]
                .as_str()
                .ok_or("archive directory required")?;
            let rel = member(relative)?;
            if !relative.starts_with("_source-assets/") || rel.components().count() < 2 {
                return Err("frozen archives belong below _source-assets".into());
            }
            let root = package.join(rel);
            safe_ancestors(package, &root)?;
            if !fs::symlink_metadata(&root)
                .map_err(|e| e.to_string())?
                .is_dir()
            {
                return Err("archive must be a regular directory".into());
            }
            if accepted
                .iter()
                .any(|p: &PathBuf| root.starts_with(p) || p.starts_with(&root))
            {
                return Err("duplicate or overlapping frozen archives".into());
            }
            let expected = archive["files"]
                .as_object()
                .filter(|a| !a.is_empty())
                .ok_or("complete file inventory required")?;
            let mut actual = BTreeMap::new();
            let mut directories = BTreeSet::new();
            walk(&root, &root, &mut actual, &mut directories)?;
            let expected_dirs = archive["directories"]
                .as_array()
                .ok_or("complete directory inventory required")?
                .iter()
                .map(|d| {
                    member(d.as_str().ok_or("invalid archive directory member")?)
                        .map(Path::to_owned)
                })
                .collect::<Result<BTreeSet<_>>>()?;
            if expected_dirs.len() != archive["directories"].as_array().unwrap().len() {
                return Err("duplicate source archive directory member".into());
            }
            if directories != expected_dirs {
                return Err("archive directory inventory changed".into());
            }
            if actual.len() != expected.len() {
                return Err("archive has missing or extra members".into());
            }
            for (name, row) in expected {
                keys(row, &["sha256", "bytes"])?;
                member(name)?;
                let (hash, bytes) = actual
                    .get(Path::new(name))
                    .ok_or("archive member missing")?;
                if row["sha256"].as_str() != Some(hash.as_str())
                    || row["bytes"].as_u64() != Some(*bytes)
                {
                    return Err(format!("frozen source bytes changed: {name}"));
                }
            }
            let capture = archive["capture_note"]
                .as_str()
                .ok_or("original capture note required")?;
            let capture_path = member(capture)?;
            if capture_path.components().count() != 1
                || capture_path.extension().is_none_or(|s| s != "md")
            {
                return Err("capture note must be an original root Markdown file".into());
            }
            let (header, _) = read_metadata(&root.join(capture_path))
                .map_err(|e| e.to_string())?
                .ok_or("original capture frontmatter required")?;
            if [
                "task-id",
                "task_id",
                "component-id",
                "component_id",
                "project_id",
                "layer",
            ]
            .iter()
            .any(|k| header.get(*k).is_some())
                || text(&header, "type") == Some("task")
            {
                return Err(
                    "operational tasks and Wikis cannot become opaque capture archives".into(),
                );
            }
            let pinned = match archive["producer"].as_str() {
                Some("safari-reading-list") => reading(&root, capture, &header, &actual)?,
                Some("mail-ingestion") => mail(&root, capture, &header, &actual)?,
                _ => return Err("unsupported frozen source producer".into()),
            };
            for name in actual.keys() {
                if name.extension().is_some_and(|s| s == "md")
                    && name != capture_path
                    && !pinned.contains(name)
                {
                    return Err(
                        "unlisted ordinary Markdown cannot be an opaque source payload".into(),
                    );
                }
            }
            accepted.insert(root);
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok((accepted, vec![])),
        Err(e) => Ok((BTreeSet::new(), vec![e])),
    }
}
fn keys(value: &Value, allowed: &[&str]) -> Result<()> {
    let object = value
        .as_object()
        .ok_or("source archive declarations must be objects")?;
    if object.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err("unknown source archive declaration field".into());
    }
    Ok(())
}
fn member(s: &str) -> Result<&Path> {
    let p = Path::new(s);
    if s.is_empty()
        || s.contains(['\\', '\r', '\n', '\0'])
        || s.split('/').any(|p| p.is_empty() || p == "." || p == "..")
        || !p.components().all(|p| matches!(p, Component::Normal(_)))
    {
        return Err("unsafe source archive member".into());
    }
    Ok(p)
}
fn safe_ancestors(base: &Path, path: &Path) -> Result<()> {
    let relative = path.strip_prefix(base).map_err(|e| e.to_string())?;
    let mut p = base.to_owned();
    for c in relative.components() {
        p.push(c);
        let m = fs::symlink_metadata(&p).map_err(|e| e.to_string())?;
        if m.is_symlink() {
            return Err("symbolic links are outside source archives".into());
        }
    }
    Ok(())
}
fn regular(p: &Path) -> Result<()> {
    let m = fs::symlink_metadata(p).map_err(|e| e.to_string())?;
    if !m.is_file() || m.is_symlink() {
        return Err("source archive member must be a regular file".into());
    }
    Ok(())
}
fn walk(
    base: &Path,
    dir: &Path,
    files: &mut BTreeMap<PathBuf, (String, u64)>,
    directories: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_dir() {
            directories.insert(
                path.strip_prefix(base)
                    .map_err(|e| e.to_string())?
                    .to_owned(),
            );
            walk(base, &path, files, directories)?;
        } else if kind.is_file() {
            regular(&path)?;
            files.insert(
                path.strip_prefix(base)
                    .map_err(|e| e.to_string())?
                    .to_owned(),
                mail_manifest::hash_file(&path).map_err(|e| e.to_string())?,
            );
        } else {
            return Err("symbolic or special source archive member".into());
        }
    }
    Ok(())
}
fn json(p: &Path) -> Result<Value> {
    regular(p)?;
    let bytes = mail_manifest::read_manifest(p).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
fn digest(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
fn reading(
    root: &Path,
    capture: &str,
    header: &Value,
    actual: &BTreeMap<PathBuf, (String, u64)>,
) -> Result<BTreeSet<PathBuf>> {
    let marker = json(&root.join(".reading-list-source.json"))?;
    let url = marker["url"].as_str().ok_or("reading-list URL required")?;
    let local = url.starts_with("file:///") && url.len() > 8 && !url.contains(['\r', '\n', '\0']);
    if marker["schema"] != 1
        || !marker["identity"].as_str().is_some_and(digest)
        || !(url.starts_with("https://") || url.starts_with("http://") || local)
        || text(header, "type") != Some("bookmark")
        || text(header, "capture") != Some("safari-reading-list")
        || !text(header, "captured").is_some_and(crate::export_artifacts::timestamp)
        || text(header, "url") != Some(url)
    {
        return Err("reading-list capture header and producer identity must agree".into());
    }
    let files = marker["files"]
        .as_object()
        .ok_or("reading-list files required")?;
    if !(3..=4).contains(&files.len())
        || !["entry.plist", "source.json", "extracted.md"]
            .iter()
            .all(|s| files.contains_key(*s))
        || (files.len() == 4 && !files.contains_key("source.html"))
    {
        return Err("unsupported reading-list payload shape".into());
    }
    let mut pinned = BTreeSet::new();
    for (name, hash) in files {
        let name = member(name)?;
        if actual.get(name).map(|(s, _)| s.as_str()) != hash.as_str() {
            return Err("reading-list protected payload hash mismatch".into());
        }
        pinned.insert(name.to_owned());
    }
    let source = json(&root.join("source.json"))?;
    if source["url"].as_str() != Some(url) {
        return Err("reading-list source URL binding differs".into());
    }
    if !actual.contains_key(Path::new(capture))
        || !actual.contains_key(Path::new(".reading-list-source.json"))
    {
        return Err("reading-list package members missing".into());
    }
    Ok(pinned)
}
fn mail(
    root: &Path,
    capture: &str,
    header: &Value,
    actual: &BTreeMap<PathBuf, (String, u64)>,
) -> Result<BTreeSet<PathBuf>> {
    let name = Path::new(mail_manifest::MANIFEST);
    let marker = json(&root.join(name))?;
    let bytes = mail_manifest::read_manifest(&root.join(name)).map_err(|e| e.to_string())?;
    mail_manifest::verify_manifest(&bytes, name, Some(Path::new(capture)), |p| {
        Ok(actual.get(p).cloned())
    })
    .map_err(|e| e.to_string())?;
    let id = marker["mail_id"]
        .as_str()
        .filter(|s| digest(s))
        .ok_or("mail capture identity required")?;
    if text(header, "type") != Some("event")
        || text(header, "category") != Some("email")
        || text(header, "id") != Some(format!("mail-{id}").as_str())
        || text(header, "eml_file") != marker["eml"].as_str()
        || text(header, "eml_sha256") != marker["raw_sha256"].as_str()
        || !marker["message_ids"]
            .as_array()
            .is_some_and(|a| match header["message_id"].as_str() {
                Some(id) => a.iter().any(|v| v.as_str() == Some(id)),
                None => header["message_id"].is_null() && a.is_empty(),
            })
    {
        return Err("mail capture header must match its original producer identity".into());
    }
    let mut pinned = BTreeSet::new();
    for attachment in marker["attachments"]
        .as_array()
        .ok_or("mail attachments missing")?
    {
        let name = attachment["file"]
            .as_str()
            .ok_or("mail attachment path missing")?;
        let path = member(name)?;
        if path.extension().is_some_and(|ext| ext == "md") && !name.starts_with("attachments/") {
            return Err(
                "opaque Markdown mail payloads must be declared attachments inside attachments/"
                    .into(),
            );
        }
        pinned.insert(path.to_owned());
    }
    Ok(pinned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use sha2::{Digest, Sha256};
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        base: PathBuf,
        package: PathBuf,
        root: PathBuf,
        producer: &'static str,
    }
    impl Fixture {
        fn reading() -> Self {
            let base = std::env::temp_dir().join(format!(
                "source-archive-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let package = base.join("20200101-review");
            let root = package.join("_source-assets/original-package");
            fs::create_dir_all(&root).unwrap();
            let f = Self {
                base,
                package,
                root,
                producer: "safari-reading-list",
            };
            f.put("capture.md","---\ntype: bookmark\ncapture: safari-reading-list\nurl: https://example.invalid/page\ncaptured: 2020-01-01T12:30:00Z\n---\n# Original capture\n");
            f.put("entry.plist", "Fictional producer input");
            f.put("source.json", r#"{"url":"https://example.invalid/page"}"#);
            f.put("extracted.md","# Original extracted page\n\nA fictional captured source, not an operational note.\n");
            f.reading_marker();
            f.index();
            f
        }
        fn mail() -> Self {
            let mut f = Self::reading();
            for name in [
                ".reading-list-source.json",
                "entry.plist",
                "source.json",
                "extracted.md",
            ] {
                fs::remove_file(f.root.join(name)).unwrap();
            }
            let id = "a".repeat(64);
            let eml =
                "Message-ID: <fictional@example.invalid>\r\n\r\nFictional original email.\r\n";
            f.put("capture.eml", eml);
            let hash = format!("{:x}", Sha256::digest(eml.as_bytes()));
            f.put("capture.md",&format!("---\ntype: event\ncategory: email\nid: mail-{id}\nmessage_id: '<fictional@example.invalid>'\neml_file: capture.eml\neml_sha256: {hash}\n---\n# Original email\n"));
            f.put(".mail-source.json",&json!({"schema":mail_manifest::MAIL_SCHEMA,"mail_id":id,"raw_sha256":hash,"eml":"capture.eml","message_ids":["<fictional@example.invalid>"],"files":{"capture.eml":hash},"attachments":[],"capture_foldernote":"capture.md"}).to_string());
            f.producer = "mail-ingestion";
            f.index();
            f
        }
        fn put(&self, name: &str, s: &str) {
            let p = self.root.join(name);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, s).unwrap();
        }
        fn reading_marker(&self) {
            let mut files = BTreeMap::new();
            for name in ["entry.plist", "source.json", "extracted.md"] {
                files.insert(
                    name,
                    mail_manifest::hash_file(&self.root.join(name)).unwrap().0,
                );
            }
            self.put(".reading-list-source.json",&json!({"schema":1,"identity":"a".repeat(64),"url":"https://example.invalid/page","files":files}).to_string());
        }
        fn index(&self) {
            let mut files = BTreeMap::new();
            let mut dirs = BTreeSet::new();
            walk(&self.root, &self.root, &mut files, &mut dirs).unwrap();
            let files = files
                .into_iter()
                .map(|(p, (hash, n))| (p, json!({"sha256":hash,"bytes":n})))
                .collect::<BTreeMap<_, _>>();
            fs::write(self.package.join("source-archives.json"),json!({"schema":"viggobsidian.native-source-archives.v1","package_note":"20200101-review.md","archives":[{"directory":"_source-assets/original-package","producer":self.producer,"capture_note":"capture.md","files":files,"directories":dirs}]}).to_string()).unwrap();
        }
        fn check(&self) -> (BTreeSet<PathBuf>, Vec<String>) {
            check(
                &self.package,
                &json!({"source_archives":"source-archives.json"}),
            )
            .unwrap()
        }
        fn bad(&self) {
            let (a, e) = self.check();
            assert!(a.is_empty() && !e.is_empty(), "{e:?}");
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.base).unwrap();
        }
    }
    #[test]
    fn known_native_producers_preserve_opaque_original_bytes() {
        for f in [Fixture::reading(), Fixture::mail()] {
            let before = fs::read(f.root.join("capture.md")).unwrap();
            let (a, e) = f.check();
            assert!(e.is_empty(), "{e:?}");
            assert_eq!(a, BTreeSet::from([f.root.clone()]));
            assert_eq!(before, fs::read(f.root.join("capture.md")).unwrap());
        }
    }
    #[test]
    fn drift_missing_extra_files_and_empty_directories_fail_closed() {
        let f = Fixture::reading();
        f.put("extracted.md", "Changed");
        f.bad();
        let f = Fixture::reading();
        fs::remove_file(f.root.join("source.json")).unwrap();
        f.bad();
        let f = Fixture::reading();
        f.put("extra.txt", "Extra");
        f.bad();
        let f = Fixture::reading();
        fs::create_dir(f.root.join("extra-empty-dir")).unwrap();
        f.bad();
    }
    #[test]
    fn rehashed_but_unlisted_ordinary_markdown_is_not_an_archive() {
        let f = Fixture::reading();
        f.put("ordinary.md", "---\ntype: task\n---\n# Active task\n");
        f.index();
        f.bad();
    }
    #[test]
    fn primary_tasks_wikis_and_wrong_header_bindings_are_refused() {
        for replacement in [
            "type: task",
            "type: note",
            "type: bookmark\ntask-id: task:active",
            "type: bookmark\nlayer: wiki",
        ] {
            let f = Fixture::reading();
            let s = fs::read_to_string(f.root.join("capture.md"))
                .unwrap()
                .replace("type: bookmark", replacement);
            f.put("capture.md", &s);
            f.index();
            f.bad();
        }
        let f = Fixture::mail();
        let s = fs::read_to_string(f.root.join("capture.md"))
            .unwrap()
            .replace("id: mail-", "id: wrong-");
        f.put("capture.md", &s);
        f.index();
        f.bad();
    }
    #[test]
    fn rehashing_outer_inventory_does_not_waive_native_protected_hashes() {
        let f = Fixture::reading();
        f.put("extracted.md", "Changed");
        f.index();
        f.bad();
        let f = Fixture::mail();
        f.put("capture.eml", "Changed EML");
        f.index();
        f.bad();
    }
    #[test]
    fn generic_mail_files_cannot_hide_an_ordinary_task_but_declared_attachments_are_input() {
        let f = Fixture::mail();
        let body = "---\ntype: task\ntask-id: task:fictional\n---\n# Task text\n- [ ] A quoted source instruction\n";
        f.put("active-task.md", body);
        let p = f.root.join(".mail-source.json");
        let mut m: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
        m["files"]["active-task.md"] = json!(format!("{:x}", Sha256::digest(body.as_bytes())));
        fs::write(&p, m.to_string()).unwrap();
        f.index();
        f.bad();
        let f = Fixture::mail();
        f.put("attachments/proposal.md", body);
        let p = f.root.join(".mail-source.json");
        let mut m: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
        let hash = format!("{:x}", Sha256::digest(body.as_bytes()));
        m["files"]["attachments/proposal.md"] = json!(hash);
        m["attachments"] =
            json!([{"file":"attachments/proposal.md","sha256":hash,"bytes":body.len()}]);
        fs::write(p, m.to_string()).unwrap();
        f.index();
        assert!(f.check().1.is_empty());
    }
    #[test]
    fn duplicate_directory_entries_and_unknown_contract_fields_are_rejected() {
        let f = Fixture::reading();
        fs::create_dir(f.root.join("empty")).unwrap();
        f.index();
        let p = f.package.join("source-archives.json");
        let v: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
        let mut duplicate = v.clone();
        duplicate["archives"][0]["directories"] = json!(["empty", "empty"]);
        fs::write(&p, duplicate.to_string()).unwrap();
        f.bad();
        for level in 0..3 {
            let mut unknown = v.clone();
            match level {
                0 => unknown["unexpected"] = json!(true),
                1 => unknown["archives"][0]["unexpected"] = json!(true),
                _ => unknown["archives"][0]["files"]["capture.md"]["unexpected"] = json!(true),
            }
            fs::write(&p, unknown.to_string()).unwrap();
            f.bad();
        }
    }
    #[test]
    fn invalid_inventory_binding_traversal_and_overlapping_roots_fail() {
        let f = Fixture::reading();
        let p = f.package.join("source-archives.json");
        let original: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
        for directory in [
            "../outside",
            "_source-assets/../original-package",
            "_source-assets//original-package",
            "_source-assets/./original-package",
        ] {
            let mut v = original.clone();
            v["archives"][0]["directory"] = json!(directory);
            fs::write(&p, v.to_string()).unwrap();
            f.bad();
        }
        let mut v = original.clone();
        let row = v["archives"][0].clone();
        v["archives"].as_array_mut().unwrap().push(row);
        fs::write(&p, v.to_string()).unwrap();
        f.bad();
        let mut v = original;
        v["package_note"] = json!("other.md");
        fs::write(p, v.to_string()).unwrap();
        f.bad();
    }
    #[test]
    fn url_is_provenance_and_must_match_the_captured_header_and_source() {
        let f = Fixture::reading();
        let marker = f.root.join(".reading-list-source.json");
        let mut m: Value = serde_json::from_slice(&fs::read(&marker).unwrap()).unwrap();
        m["url"] = json!("https://other.invalid");
        fs::write(marker, m.to_string()).unwrap();
        f.index();
        f.bad();
        let f = Fixture::reading();
        f.put("source.json", r#"{"url":"https://other.invalid"}"#);
        f.reading_marker();
        f.index();
        f.bad();
    }
    #[cfg(unix)]
    #[test]
    fn directory_leaf_and_dangling_symlinks_are_refused() {
        use std::os::unix::fs::symlink;
        let f = Fixture::reading();
        fs::remove_file(f.root.join("extracted.md")).unwrap();
        symlink("missing", f.root.join("extracted.md")).unwrap();
        f.bad();
        let f = Fixture::reading();
        let actual = f.package.join("outside");
        fs::rename(&f.root, &actual).unwrap();
        symlink(&actual, &f.root).unwrap();
        f.bad();
    }
}
