//! Read-only, lexical retrieval for one Wiki/Raw dossier.
//!
//! This crate builds a disposable JSON index outside the vault and re-reads every
//! cited source at query time. It ranks excerpts; it does not infer domain facts
//! or prove semantic consistency between notes.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

const SCHEMA: &str = "viggobsidian.dossier-retrieval-index.v1";
const MAX_FILES: usize = 512;
const MAX_FILE_BYTES: u64 = 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 16 * 1024 * 1024;
const MAX_SECTIONS_PER_FILE: usize = 128;
const MAX_QUERY_BYTES: usize = 4096;
const MAX_RESULTS: usize = 20;

pub type Result<T> = std::result::Result<T, String>;

fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(message.into())
}
fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn safe_relative(text: &str) -> Result<&Path> {
    let path = Path::new(text);
    if text.is_empty()
        || text.contains('\\')
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return fail(format!("unsafe relative path: {text}"));
    }
    Ok(path)
}

fn absolute_without_symlinks(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    if absolute
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return fail("parent component refused");
    }
    let mut cursor = PathBuf::new();
    for part in absolute.components() {
        cursor.push(part.as_os_str());
        let metadata = fs::symlink_metadata(&cursor)
            .map_err(|e| format!("cannot inspect {}: {e}", cursor.display()))?;
        if metadata.file_type().is_symlink() {
            return fail(format!("symlink refused: {}", cursor.display()));
        }
    }
    let canonical = absolute
        .canonicalize()
        .map_err(|e| format!("cannot canonicalize {}: {e}", absolute.display()))?;
    if !canonical.is_dir() {
        return fail("vault is not a directory");
    }
    Ok(canonical)
}

fn checked_child(root: &Path, relative: &str) -> Result<PathBuf> {
    let mut out = root.to_path_buf();
    for part in safe_relative(relative)?.components() {
        out.push(part.as_os_str());
        let metadata = fs::symlink_metadata(&out)
            .map_err(|e| format!("cannot inspect {}: {e}", out.display()))?;
        if metadata.file_type().is_symlink() {
            return fail(format!("symlink refused: {}", out.display()));
        }
    }
    Ok(out)
}

fn dossier_root(vault: &Path, dossier: &str) -> Result<PathBuf> {
    let path = checked_child(vault, dossier)?;
    if !path.is_dir() {
        return fail("dossier is not a directory under --vault");
    }
    Ok(path)
}

fn markdown_files(root: &Path, vault: &Path) -> Result<Vec<(String, PathBuf)>> {
    fn visit(
        dir: &Path,
        vault: &Path,
        output: &mut Vec<(String, PathBuf)>,
        total: &mut u64,
    ) -> Result<()> {
        for entry in fs::read_dir(dir).map_err(|e| format!("cannot list {}: {e}", dir.display()))? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') {
                continue;
            }
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_symlink() {
                return fail(format!(
                    "symlink refused in dossier: {}",
                    entry.path().display()
                ));
            }
            if kind.is_dir() {
                visit(&entry.path(), vault, output, total)?;
                continue;
            }
            if !kind.is_file() || !name.ends_with(".md") {
                continue;
            }
            let bytes = entry.metadata().map_err(|e| e.to_string())?.len();
            if bytes > MAX_FILE_BYTES {
                return fail(format!(
                    "Markdown file exceeds {} bytes: {}",
                    MAX_FILE_BYTES,
                    entry.path().display()
                ));
            }
            *total = total.checked_add(bytes).ok_or("total size overflow")?;
            if *total > MAX_TOTAL_BYTES {
                return fail(format!(
                    "dossier Markdown exceeds {} bytes",
                    MAX_TOTAL_BYTES
                ));
            }
            let relative = entry
                .path()
                .strip_prefix(vault)
                .map_err(|_| "file escaped vault")?
                .to_string_lossy()
                .replace('\\', "/");
            output.push((relative, entry.path()));
            if output.len() > MAX_FILES {
                return fail(format!("dossier has more than {MAX_FILES} Markdown files"));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    let mut total = 0;
    visit(root, vault, &mut out, &mut total)?;
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

fn split_frontmatter(text: &str) -> Result<(Value, &str, usize)> {
    if !text.starts_with("---\n") {
        return Ok((json!({}), text, 0));
    }
    let Some(end) = text[4..].find("\n---\n") else {
        return fail("unterminated frontmatter");
    };
    let yaml = &text[4..4 + end];
    let options = serde_saphyr::options! { duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error, merge_keys: serde_saphyr::MergeKeyPolicy::Error };
    let metadata: Value = serde_saphyr::from_str_with_options(yaml, options)
        .map_err(|_| "invalid or duplicate frontmatter")?;
    if !metadata.is_object() {
        return fail("frontmatter must be a mapping");
    }
    let body_offset = 4 + end + 5;
    Ok((
        metadata,
        &text[body_offset..],
        text[..body_offset].lines().count(),
    ))
}

fn kind_for(dossier: &str, path: &str) -> Option<&'static str> {
    let normalized = format!("/{path}");
    let raw = format!("/{dossier}/raw/");
    let wiki = format!("/{dossier}/wiki/");
    if normalized.contains(&wiki) {
        Some("wiki")
    } else if normalized.contains(&raw) {
        Some("raw")
    } else {
        None
    }
}

fn heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let hashes = trimmed.bytes().take_while(|b| *b == b'#').count();
    if (1..=6).contains(&hashes) && trimmed.as_bytes().get(hashes) == Some(&b' ') {
        let title = trimmed[hashes + 1..].trim().trim_end_matches('#').trim();
        if !title.is_empty() {
            return Some((hashes, title.to_owned()));
        }
    }
    None
}

fn historical_details_summary(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("<summary")
        && [
            "wijzigingshistor",
            "wijzigingenhistor",
            "change history",
            "changelog",
        ]
        .iter()
        .any(|marker| lower.contains(marker))
}

fn visible_lines(body: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut fence: Option<char> = None;
    let mut comment = false;
    let mut details_depth = 0usize;
    let mut hidden_historical_details = false;
    for (offset, raw) in body.lines().enumerate() {
        let line = raw.trim_end_matches('\r');
        let trim = line.trim_start();
        if trim.starts_with("<!--") {
            comment = !trim.contains("-->");
            continue;
        }
        if comment {
            if trim.contains("-->") {
                comment = false;
            }
            continue;
        }
        if let Some(marker) = fence {
            if trim.starts_with(marker) {
                fence = None;
            }
            continue;
        }
        if trim.starts_with("```") || trim.starts_with("~~~") {
            fence = trim.chars().next();
            continue;
        }
        if trim.contains("<details") {
            details_depth += 1;
            hidden_historical_details |= historical_details_summary(trim);
            continue;
        }
        if details_depth > 0 {
            if historical_details_summary(trim) {
                hidden_historical_details = true;
            }
            if trim.contains("</details>") {
                details_depth -= 1;
                if details_depth == 0 {
                    hidden_historical_details = false;
                }
            }
            if hidden_historical_details || trim.contains("<summary") || trim.contains("</details>")
            {
                continue;
            }
        }
        if trim.starts_with("> [!navigation")
            || trim.contains("[Finder](file:")
            || trim.contains("vault-navigation")
        {
            continue;
        }
        if trim.contains("[[")
            && (trim.contains("|Back]]") || trim.contains("|Raw]]") || trim.contains("|Terug]]"))
        {
            continue;
        }
        out.push((offset + 1, line.to_owned()));
    }
    out
}

fn links(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some((_, tail)) = rest.split_once("[[") {
        let Some((inside, after)) = tail.split_once("]]") else {
            break;
        };
        let target = inside
            .split('|')
            .next()
            .unwrap_or(inside)
            .trim_end_matches('\\')
            .trim();
        let target = target
            .split('#')
            .next()
            .unwrap_or(target)
            .trim_end_matches(".md");
        if !target.is_empty() && !target.contains(['/', '\\']) || target.contains('/') {
            out.push(target.to_owned());
        }
        rest = after;
    }
    out
}

fn target_link(value: &str) -> Option<(String, String)> {
    let value = value.strip_prefix("[[")?.strip_suffix("]]")?;
    let value = value.split('|').next()?.trim_end_matches('\\');
    let (note, anchor) = value.split_once('#')?;
    let note = note.trim_end_matches(".md");
    (!note.is_empty() && !anchor.trim().is_empty())
        .then_some((note.to_owned(), anchor.trim().to_owned()))
}

fn normalized_words(text: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "de", "het", "een", "en", "van", "voor", "met", "op", "in", "aan", "te", "ik", "is",
        "zijn", "wat", "welke", "hoe", "kan", "kun", "mijn", "ons", "over", "naar", "bij", "om",
        "dat", "die", "dit", "als", "niet", "nog", "waar", "uit", "zijn", "wanneer", "eindigt",
        "is", "het", "mijn",
    ];
    let mut words = Vec::new();
    let mut current = String::new();
    for ch in text.to_lowercase().chars() {
        if ch.is_alphanumeric() {
            current.push(ch);
        } else if !current.is_empty() {
            let word = normalize_word(&current);
            if word.len() > 1 && !STOP.contains(&word.as_str()) {
                if let Some(part) = compound_part(&word) {
                    words.push(part);
                } else {
                    words.push(word);
                }
            }
            current.clear();
        }
    }
    if !current.is_empty() {
        let word = normalize_word(&current);
        if word.len() > 1 && !STOP.contains(&word.as_str()) {
            if let Some(part) = compound_part(&word) {
                words.push(part);
            } else {
                words.push(word);
            }
        }
    }
    words
}

fn normalize_word(word: &str) -> String {
    let word = match word {
        "kostte" | "kosten" | "kost" | "prijs" | "prijzen" => "kost",
        word if word.ends_with("kosten") || word.ends_with("prijs") => "kost",
        _ => word,
    };
    let mut out = word.to_owned();
    for suffix in ["heden", "eren", "en", "en", "es", "s"] {
        if out.len() > suffix.len() + 3 && out.ends_with(suffix) {
            out.truncate(out.len() - suffix.len());
            break;
        }
    }
    out
}

fn compound_part(word: &str) -> Option<String> {
    for suffix in ["beurt", "kosten", "prijs", "periode"] {
        if let Some(prefix) = word.strip_suffix(suffix)
            && prefix.len() >= 4
        {
            return Some(prefix.trim_end_matches('s').to_owned());
        }
    }
    None
}

fn historical(path: &str, metadata: &Value) -> bool {
    let lower = format!(
        "{path} {} {}",
        metadata
            .get("category")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        metadata
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .to_lowercase();
    [
        "histor",
        "snapshot",
        "archive",
        "archief",
        "changelog",
        "change-log",
        "logboek",
    ]
    .iter()
    .any(|word| lower.contains(word))
}

fn uncertainty(line: &str) -> bool {
    let lower = line.to_lowercase();
    [
        "onzeker",
        "onduidelijk",
        "onbekend",
        "niet bevestigd",
        "nog te",
        "moet nog",
        "ontbreekt",
        "pending",
    ]
    .iter()
    .any(|word| lower.contains(word))
}

fn file_title(lines: &[(usize, String)], fallback: &str) -> String {
    lines
        .iter()
        .find_map(|(_, line)| {
            heading(line)
                .filter(|(depth, _)| *depth == 1)
                .map(|(_, title)| title)
        })
        .unwrap_or_else(|| fallback.to_owned())
}

fn sections(path: &str, body: &str, line_offset: usize) -> Result<Vec<Value>> {
    let lines = visible_lines(body);
    let title = file_title(
        &lines,
        Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("note"),
    );
    let mut starts = Vec::new();
    for (position, (line, text)) in lines.iter().enumerate() {
        if let Some((level, title)) = heading(text) {
            starts.push((position, *line, level, title));
        }
    }
    let mut rows = Vec::new();
    if starts.is_empty() {
        let text = lines
            .iter()
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if !text.trim().is_empty() {
            rows.push(json!({"section": title, "start_line": lines.first().map(|v|v.0 + line_offset).unwrap_or(1 + line_offset), "end_line": lines.last().map(|v|v.0 + line_offset).unwrap_or(1 + line_offset), "text":text}));
        }
        return Ok(rows);
    }
    for (index, (position, start_line, _level, name)) in starts.iter().enumerate() {
        if rows.len() >= MAX_SECTIONS_PER_FILE {
            return fail(format!("too many sections in {path}"));
        }
        let mut end = lines.len();
        // Keep evidence chunks disjoint: a parent must not absorb a child's
        // matches and then hide its own current text during overlap deduplication.
        if let Some((next_position, _, _, _)) = starts.get(index + 1) {
            end = *next_position;
        }
        let text = lines[*position..end]
            .iter()
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let end_line = lines
            .get(end.saturating_sub(1))
            .map(|v| v.0)
            .unwrap_or(*start_line);
        rows.push(json!({"section":name,"start_line":start_line + line_offset,"end_line":end_line + line_offset,"text":text}));
    }
    Ok(rows)
}

fn source_artifact(note: &Path, metadata: &Value) -> Result<Value> {
    let file = metadata.get("source_file").and_then(Value::as_str);
    let expected = metadata.get("source_sha256").and_then(Value::as_str);
    match (file, expected) {
        (None, None) => Ok(Value::Null),
        (None, Some(expected)) => {
            if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return fail("source_sha256 must be a SHA-256 hex digest");
            }
            Ok(json!({"sha256":expected}))
        }
        (Some(file), Some(expected)) => {
            if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return fail("source_sha256 must be a SHA-256 hex digest");
            }
            let parent = note.parent().ok_or("Raw note has no package directory")?;
            let relative = safe_relative(file)?;
            let mut artifact = parent.to_path_buf();
            for part in relative.components() {
                artifact.push(part.as_os_str());
                let metadata = fs::symlink_metadata(&artifact)
                    .map_err(|e| format!("source_file unavailable: {file}: {e}"))?;
                if metadata.file_type().is_symlink() {
                    return fail(format!("source_file symlink refused: {file}"));
                }
            }
            let metadata = fs::metadata(&artifact)
                .map_err(|e| format!("source_file unavailable: {file}: {e}"))?;
            if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
                return fail(format!("invalid source_file: {file}"));
            }
            let actual = sha256(&fs::read(&artifact).map_err(|e| e.to_string())?);
            if actual != expected {
                return fail(format!("source_file hash mismatch: {file}"));
            }
            Ok(json!({"file":file,"sha256":expected}))
        }
        (Some(_), None) => fail("Raw source_file requires source_sha256"),
    }
}

fn doc_record(dossier: &str, path: &str, note: &Path, source: &[u8]) -> Result<Option<Value>> {
    let Some(kind) = kind_for(dossier, path) else {
        return Ok(None);
    };
    let text = std::str::from_utf8(source).map_err(|_| format!("Markdown is not UTF-8: {path}"))?;
    let (metadata, body, line_offset) = split_frontmatter(text)?;
    let link_targets = links(body);
    let wiki_target = metadata
        .get("wiki_target")
        .and_then(Value::as_str)
        .and_then(target_link);
    let raw_status = metadata
        .get("wiki_status")
        .and_then(Value::as_str)
        .unwrap_or("missing");
    let artifact = if kind == "raw" {
        source_artifact(note, &metadata)?
    } else {
        Value::Null
    };
    Ok(Some(
        json!({"path":path,"sha256":sha256(source),"kind":kind,"historical":historical(path,&metadata),"title":file_title(&visible_lines(body), Path::new(path).file_stem().and_then(|s|s.to_str()).unwrap_or("note")),"wiki_status":if kind=="raw" {raw_status}else{"n/a"},"wiki_target":wiki_target.map(|(note,anchor)|json!({"note":note,"section":anchor})).unwrap_or(Value::Null),"source_artifact":artifact,"links":link_targets,"sections":sections(path,body,line_offset)?}),
    ))
}

fn dossier_documents(vault: &Path, dossier: &str) -> Result<Vec<Value>> {
    let dossier_path = dossier_root(vault, dossier)?;
    let mut documents = Vec::new();
    for (path, full) in markdown_files(&dossier_path, vault)? {
        let bytes = fs::read(&full).map_err(|e| format!("cannot read {}: {e}", full.display()))?;
        if let Some(document) = doc_record(dossier, &path, &full, &bytes)? {
            documents.push(document);
        }
    }
    if documents.is_empty() {
        return fail("dossier has no Markdown beneath wiki or raw");
    }
    Ok(documents)
}

fn validate_index(value: &Value) -> Result<()> {
    let object = value.as_object().ok_or("index must be a JSON object")?;
    let allowed: BTreeSet<&str> = [
        "schema",
        "vault",
        "dossier",
        "documents",
        "index_sha256",
        "limits",
    ]
    .into_iter()
    .collect();
    if object.keys().any(|key| !allowed.contains(key.as_str())) || value["schema"] != SCHEMA {
        return fail("unsupported or malformed index schema");
    }
    let vault = value["vault"].as_str().ok_or("index vault missing")?;
    if !Path::new(vault).is_absolute() {
        return fail("index vault must be absolute");
    }
    safe_relative(value["dossier"].as_str().ok_or("index dossier missing")?)?;
    let docs = value["documents"]
        .as_array()
        .ok_or("index documents missing")?;
    if docs.is_empty() || docs.len() > MAX_FILES {
        return fail("index document count invalid");
    }
    let mut paths = BTreeSet::new();
    for doc in docs {
        let object = doc.as_object().ok_or("invalid index document")?;
        let allowed: BTreeSet<&str> = [
            "path",
            "sha256",
            "kind",
            "historical",
            "title",
            "wiki_status",
            "wiki_target",
            "source_artifact",
            "links",
            "sections",
        ]
        .into_iter()
        .collect();
        if object.keys().any(|key| !allowed.contains(key.as_str())) {
            return fail("unknown index document field");
        }
        let path = doc["path"].as_str().ok_or("document path missing")?;
        safe_relative(path)?;
        if !paths.insert(path)
            || !matches!(doc["kind"].as_str(), Some("wiki" | "raw"))
            || doc["sha256"]
                .as_str()
                .is_none_or(|v| v.len() != 64 || !v.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return fail("invalid index document identity");
        }
        let sections = doc["sections"]
            .as_array()
            .ok_or("document sections missing")?;
        if sections.len() > MAX_SECTIONS_PER_FILE {
            return fail("too many indexed sections");
        }
        for section in sections {
            if section["section"].as_str().is_none()
                || section["text"].as_str().is_none()
                || section["start_line"].as_u64().is_none()
                || section["end_line"].as_u64().is_none()
            {
                return fail("invalid indexed section");
            }
        }
        if !doc["source_artifact"].is_null() {
            let artifact = doc["source_artifact"]
                .as_object()
                .ok_or("invalid indexed source artifact")?;
            let allowed: BTreeSet<&str> = ["file", "sha256"].into_iter().collect();
            if artifact.keys().any(|key| !allowed.contains(key.as_str()))
                || artifact
                    .get("sha256")
                    .and_then(Value::as_str)
                    .is_none_or(|value| {
                        value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
                    })
                || artifact.get("file").is_some_and(|file| {
                    file.as_str()
                        .is_none_or(|file| safe_relative(file).is_err())
                })
            {
                return fail("invalid indexed source artifact");
            }
        }
    }
    let expected = value["index_sha256"].as_str().ok_or("index hash missing")?;
    let mut without_hash = value.clone();
    without_hash
        .as_object_mut()
        .expect("checked object")
        .remove("index_sha256");
    if sha256(&serde_json::to_vec(&without_hash).map_err(|e| e.to_string())?) != expected {
        return fail("index integrity hash mismatch");
    }
    Ok(())
}

fn external_index_path(index: &Path, vault: &Path) -> Result<PathBuf> {
    if !index.is_absolute() {
        return fail("--index must be an absolute path outside the vault and repository");
    }
    let parent = index.parent().ok_or("index needs a parent directory")?;
    let parent = absolute_without_symlinks(parent)?;
    let file = index
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or("index needs a filename")?;
    if file.starts_with('.') || file.is_empty() {
        return fail("hidden index filename refused");
    }
    let destination = parent.join(file);
    if destination.starts_with(vault) {
        return fail("index must be outside --vault");
    }
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or("repository root unavailable")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if destination.starts_with(repository) {
        return fail("index must be outside the repository");
    }
    if destination.exists()
        && fs::symlink_metadata(&destination)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
    {
        return fail("index destination is a symlink");
    }
    Ok(destination)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("index parent missing")?;
    let temporary = parent.join(format!(
        ".dossier-retrieval-{}-{}.tmp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    ));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    fs::rename(&temporary, path).map_err(|e| {
        let _ = fs::remove_file(&temporary);
        e.to_string()
    })
}

/// Build a disposable JSON index. This reads the vault and only writes `index`.
pub fn build_index(vault_arg: &Path, dossier: &str, index_arg: &Path) -> Result<Value> {
    let vault = absolute_without_symlinks(vault_arg)?;
    let index = external_index_path(index_arg, &vault)?;
    let documents = dossier_documents(&vault, dossier)?;
    let mut output = json!({"schema":SCHEMA,"vault":vault,"dossier":dossier,"documents":documents,"limits":{"max_files":MAX_FILES,"max_file_bytes":MAX_FILE_BYTES,"max_total_bytes":MAX_TOTAL_BYTES}});
    let digest = sha256(&serde_json::to_vec(&output).map_err(|e| e.to_string())?);
    output["index_sha256"] = json!(digest);
    validate_index(&output)?;
    atomic_write(
        &index,
        &serde_json::to_vec_pretty(&output).map_err(|e| e.to_string())?,
    )?;
    Ok(
        json!({"schema":"viggobsidian.dossier-retrieval-build-result.v1","status":"complete","index":index,"documents":output["documents"].as_array().map_or(0,Vec::len),"index_sha256":output["index_sha256"]}),
    )
}

fn source_state(raw: &Value, docs: &BTreeMap<String, &Value>) -> Vec<String> {
    let mut warnings = Vec::new();
    let status = raw["wiki_status"].as_str().unwrap_or("missing");
    if status != "processed" {
        warnings.push(format!("raw wiki_status is {status}"));
    }
    if raw["source_artifact"].is_object() && raw["source_artifact"]["file"].is_null() {
        warnings.push(
            "raw source artifact unresolved: declared source_sha256 has no source_file; artifact was not verified"
                .into(),
        );
    }
    let Some(target) = raw["wiki_target"].as_object() else {
        warnings.push("raw wiki_target missing or invalid".into());
        return warnings;
    };
    let note = target
        .get("note")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim_end_matches(".md");
    let mut wiki_candidates = docs.iter().filter_map(|(path, doc)| {
        (doc["kind"] == "wiki" && resolves_link_to(note, path.trim_end_matches(".md")))
            .then_some(*doc)
    });
    let Some(wiki) = wiki_candidates.next() else {
        warnings.push("wiki_target does not resolve to a Wiki note".into());
        return warnings;
    };
    if wiki_candidates.next().is_some() {
        warnings.push("wiki_target is ambiguous across Wiki notes".into());
        return warnings;
    };
    let anchor = target
        .get("section")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let anchor_count = wiki["sections"].as_array().map_or(0, |sections| {
        sections
            .iter()
            .filter(|section| section["section"].as_str() == Some(anchor))
            .count()
    });
    match anchor_count {
        0 => warnings.push("wiki_target points to a missing Wiki section".into()),
        1 => {}
        _ => warnings.push("wiki_target points to an ambiguous Wiki section".into()),
    }
    let raw_note = raw["path"]
        .as_str()
        .unwrap_or_default()
        .trim_end_matches(".md");
    let linked = wiki["links"].as_array().is_some_and(|links| {
        links
            .iter()
            .filter_map(Value::as_str)
            .any(|link| resolves_link_to(link, raw_note))
    });
    if !linked {
        warnings.push("Wiki does not link back to this Raw source".into());
    }
    warnings
}

fn resolves_link_to(link: &str, path_without_extension: &str) -> bool {
    let link = link.trim_start_matches('/').trim_end_matches(".md");
    let path = path_without_extension
        .trim_start_matches('/')
        .trim_end_matches(".md");
    link == path || path.ends_with(&format!("/{link}"))
}

fn wiki_links_raw(raw: &Value, docs: &[Value]) -> bool {
    let raw_path = raw["path"]
        .as_str()
        .unwrap_or_default()
        .trim_end_matches(".md");
    docs.iter().filter(|doc| doc["kind"] == "wiki").any(|wiki| {
        wiki["links"].as_array().is_some_and(|links| {
            links
                .iter()
                .filter_map(Value::as_str)
                .any(|link| resolves_link_to(link, raw_path))
        })
    })
}

fn searchable_text(text: &str) -> String {
    let mut output = String::new();
    // Link locators are provenance, not prose. Matching their directory names
    // otherwise outweighs the human statement and promotes processing logs.
    let mut prose = String::new();
    let mut remaining = text;
    while let Some(start) = remaining.find("[[") {
        prose.push_str(&remaining[..start]);
        let after = &remaining[start + 2..];
        let Some(end) = after.find("]]") else {
            prose.push_str(&remaining[start..]);
            remaining = "";
            break;
        };
        if let Some((_, label)) = after[..end].split_once('|') {
            prose.push_str(label);
        }
        prose.push(' ');
        remaining = &after[end + 2..];
    }
    prose.push_str(remaining);
    let mut rest = prose.as_str();
    loop {
        let http = rest.find("http://");
        let https = rest.find("https://");
        let start = match (http, https) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) | (None, Some(a)) => a,
            (None, None) => {
                output.push_str(rest);
                break;
            }
        };
        output.push_str(&rest[..start]);
        let after = &rest[start..];
        let end = after
            .find(|ch: char| ch.is_whitespace() || ch == ')')
            .unwrap_or(after.len());
        rest = &after[end..];
    }
    output
}

fn excerpt(section: &str, terms: &[String]) -> String {
    // Preserve complete short evidence, including its caveats and attribution.
    // A four-line window can otherwise hide the answer immediately beside a hit.
    if section.len() <= 1400 {
        return section.to_owned();
    }
    let lines: Vec<_> = section.lines().collect();
    let distinct_terms: BTreeSet<_> = terms.iter().collect();
    let position = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let words = normalized_words(&searchable_text(line));
            let matched = distinct_terms
                .iter()
                .filter(|term| words.iter().any(|word| word.as_str() == term.as_str()))
                .count();
            let occurrences = words
                .iter()
                .filter(|word| {
                    distinct_terms
                        .iter()
                        .any(|term| word.as_str() == term.as_str())
                })
                .count();
            (matched, occurrences, index)
        })
        .max_by_key(|(matched, occurrences, index)| (*matched, *occurrences, *index))
        .map(|(_, _, index)| index)
        .unwrap_or(0);
    let first = position.saturating_sub(1);
    let last = (position + 3).min(lines.len());
    let text = lines[first..last].join("\n");
    if text.len() > 1400 {
        let boundary = text
            .char_indices()
            .map(|(offset, _)| offset)
            .take_while(|offset| *offset <= 1400)
            .last()
            .unwrap_or(0);
        format!("{}…", &text[..boundary])
    } else {
        text
    }
}

fn excluded_section(section: &str, _text: &str) -> bool {
    let heading = normalized_words(section);
    if heading
        .iter()
        .any(|word| matches!(word.as_str(), "inhoud" | "subfolder"))
        || section.eq_ignore_ascii_case("activity")
        || section.eq_ignore_ascii_case("verwerking")
        || section.to_lowercase().contains("verwerking in wiki")
        || section.to_lowercase().contains("praktische links")
    {
        return true;
    }
    false
}

fn credential_request(query: &str) -> bool {
    let terms = normalized_words(query);
    terms.iter().any(|term| {
        matches!(
            term.as_str(),
            "wachtwoord"
                | "password"
                | "secret"
                | "geheim"
                | "token"
                | "credential"
                | "pincode"
                | "pin"
                | "bearer"
        )
    }) || (terms
        .iter()
        .any(|term| matches!(term.as_str(), "api" | "access" | "private"))
        && terms
            .iter()
            .any(|term| matches!(term.as_str(), "key" | "sleutel")))
}

/// Query an index only after validating its schema/hash and re-reading all sources.
pub fn query_index(
    vault_arg: &Path,
    dossier: &str,
    index_arg: &Path,
    query: &str,
    limit: usize,
) -> Result<Value> {
    if query.len() > MAX_QUERY_BYTES {
        return fail("query exceeds 4096 bytes");
    }
    let terms = normalized_words(query);
    if terms.is_empty() {
        return fail("query has no searchable terms after generic Dutch normalization");
    }
    let vault = absolute_without_symlinks(vault_arg)?;
    dossier_root(&vault, dossier)?;
    let index = external_index_path(index_arg, &vault)?;
    let raw_index = fs::read(&index).map_err(|e| format!("cannot read index: {e}"))?;
    let text_index = std::str::from_utf8(&raw_index).map_err(|_| "index is not UTF-8")?;
    let options =
        serde_saphyr::options! { duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error };
    let _: Value = serde_saphyr::from_str_with_options(text_index, options)
        .map_err(|_| "index has duplicate keys or invalid JSON")?;
    let index_value: Value =
        serde_json::from_slice(&raw_index).map_err(|_| "invalid index JSON")?;
    validate_index(&index_value)?;
    if index_value["vault"].as_str() != Some(vault.to_string_lossy().as_ref())
        || index_value["dossier"].as_str() != Some(dossier)
    {
        return fail("index does not belong to this --vault and --dossier");
    }
    let indexed_docs = index_value["documents"]
        .as_array()
        .expect("validated documents");
    let live_docs = dossier_documents(&vault, dossier)?;
    let indexed_by_path: BTreeMap<_, _> = indexed_docs
        .iter()
        .map(|document| (document["path"].as_str().expect("validated path"), document))
        .collect();
    let live_by_path: BTreeMap<_, _> = live_docs
        .iter()
        .map(|document| (document["path"].as_str().expect("generated path"), document))
        .collect();
    if indexed_by_path.keys().ne(live_by_path.keys()) {
        return fail("source inventory changed; rebuild the index");
    }
    for (path, live) in &live_by_path {
        if indexed_by_path
            .get(path)
            .is_some_and(|indexed| indexed["sha256"] != live["sha256"])
        {
            return fail(format!("source drift detected: {path}; rebuild the index"));
        }
        if indexed_by_path.get(path) != Some(live) {
            return fail(format!(
                "indexed document does not match re-read source: {path}; rebuild the index"
            ));
        }
    }
    let docs = &live_docs;
    let by_path: BTreeMap<String, &Value> = live_by_path
        .into_iter()
        .map(|(path, document)| (path.to_owned(), document))
        .collect();
    let provenance_warnings: Vec<Value> = docs
        .iter()
        .filter(|doc| doc["kind"] == "raw")
        .filter_map(|doc| {
            let warnings = source_state(doc, &by_path);
            (!warnings.is_empty()).then(|| json!({"path":doc["path"],"warnings":warnings}))
        })
        .collect();
    if credential_request(query) {
        return Ok(json!({
            "schema":"viggobsidian.dossier-retrieval-query-result.v1",
            "status":"refused",
            "query":query,
            "normalized_terms":terms,
            "provenance_warnings":provenance_warnings,
            "results":[],
            "refusal":"Credential-seeking queries are not returned by this read-only retriever.",
            "limitations":["Queries matching bounded common credential-seeking patterns are refused; unrelated excerpts are not credential-redacted.","Lexical retrieval only: ranked excerpts are evidence for a caller-grounded answer, not generated answers."]
        }));
    }
    let distinct_terms: BTreeSet<_> = terms.iter().collect();
    let required_matches = distinct_terms.len().min(2);
    let mut linked_from_matching_wiki = BTreeSet::new();
    for wiki in docs.iter().filter(|doc| doc["kind"] == "wiki") {
        for section in wiki["sections"].as_array().expect("validated sections") {
            let text = section["text"].as_str().expect("validated text");
            let name = section["section"].as_str().expect("validated section name");
            if excluded_section(name, text) {
                continue;
            }
            let words = normalized_words(&searchable_text(text));
            let heading_words = normalized_words(name);
            let matched = distinct_terms
                .iter()
                .filter(|term| {
                    words.iter().any(|word| word == **term)
                        || heading_words.iter().any(|word| word == **term)
                })
                .count();
            if matched >= required_matches {
                for link in links(text) {
                    for raw in docs.iter().filter(|doc| doc["kind"] == "raw") {
                        let raw_path = raw["path"]
                            .as_str()
                            .unwrap_or_default()
                            .trim_end_matches(".md");
                        if resolves_link_to(&link, raw_path) {
                            linked_from_matching_wiki.insert(raw_path.to_owned());
                        }
                    }
                }
            }
        }
    }
    let mut results = Vec::new();
    for doc in docs {
        let evidence_warnings = if doc["kind"] == "raw" {
            source_state(doc, &by_path)
        } else {
            Vec::new()
        };
        for section in doc["sections"].as_array().expect("validated sections") {
            let text = section["text"].as_str().expect("validated text");
            let section_name = section["section"].as_str().expect("validated section name");
            if excluded_section(section_name, text) {
                continue;
            }
            let words = normalized_words(&searchable_text(text));
            let heading_words = normalized_words(section_name);
            let matched: BTreeSet<_> = distinct_terms
                .iter()
                .filter(|term| {
                    words.iter().any(|word| word == **term)
                        || heading_words.iter().any(|word| word == **term)
                })
                .collect();
            let raw_path = doc["path"]
                .as_str()
                .unwrap_or_default()
                .trim_end_matches(".md");
            let linked_evidence =
                doc["kind"] == "raw" && linked_from_matching_wiki.contains(raw_path);
            if (matched.is_empty() || !linked_evidence) && matched.len() < required_matches {
                continue;
            }
            let mut score = 0f64;
            for term in &terms {
                let occurrences = words.iter().filter(|word| *word == term).count();
                if occurrences > 0 {
                    score += 4.0 + (occurrences.min(4) as f64);
                }
                if heading_words.iter().any(|word| word == term) {
                    score += 5.0;
                }
            }
            if doc["kind"] == "wiki" {
                score += 32.0;
            } else if linked_evidence {
                score += 18.0;
            } else if wiki_links_raw(doc, docs) {
                score += 3.0;
            }
            if doc["historical"] == true {
                score *= 0.25;
            }
            let excerpt = excerpt(text, &terms);
            let uncertainty_lines: Vec<_> = excerpt
                .lines()
                .filter(|line| uncertainty(line))
                .map(str::to_owned)
                .collect();
            results.push(json!({"score":score,"path":doc["path"],"sha256":doc["sha256"],"kind":doc["kind"],"historical":doc["historical"],"section":section["section"],"lines":{"start":section["start_line"],"end":section["end_line"]},"excerpt":excerpt,"uncertainty_evidence":uncertainty_lines,"source_warnings":evidence_warnings}));
        }
    }
    results.sort_by(|a, b| {
        b["score"]
            .as_f64()
            .partial_cmp(&a["score"].as_f64())
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a["path"].as_str().cmp(&b["path"].as_str()))
    });
    let mut deduped = Vec::new();
    let mut excerpts = BTreeSet::new();
    for result in results {
        let path = result["path"].as_str().unwrap_or_default();
        let start = result["lines"]["start"].as_u64().unwrap_or_default();
        let end = result["lines"]["end"].as_u64().unwrap_or_default();
        let text = result["excerpt"].as_str().unwrap_or_default();
        let overlaps = deduped.iter().any(|kept: &Value| {
            kept["path"].as_str() == Some(path)
                && start <= kept["lines"]["end"].as_u64().unwrap_or_default()
                && end >= kept["lines"]["start"].as_u64().unwrap_or_default()
        });
        if !overlaps && excerpts.insert(text.to_owned()) {
            deduped.push(result);
        }
    }
    let requested = limit.clamp(1, MAX_RESULTS);
    deduped.truncate(requested);
    Ok(
        json!({"schema":"viggobsidian.dossier-retrieval-query-result.v1","status":"complete","query":query,"normalized_terms":terms,"provenance_warnings":provenance_warnings,"results":deduped,"limitations":["Lexical retrieval only: ranked excerpts are evidence for a caller-grounded answer, not generated answers.","Uncertainty is quoted from matching text; it is not a semantic extraction or contradiction verdict.","Historical snapshots and change logs are retained but down-ranked.","The PoC verifies indexed source bytes at query time but is not a full vault or semantic consistency checker."]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    fn fixture() -> (PathBuf, PathBuf, String) {
        let root = std::env::temp_dir().join(format!(
            "dossier-retrieval-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let vault = root.join("vault");
        let dossier = "30-topics/example-dossier".to_owned();
        fs::create_dir_all(vault.join(&dossier).join("wiki")).unwrap();
        fs::create_dir_all(vault.join(&dossier).join("raw/20260101-source")).unwrap();
        fs::create_dir_all(vault.join(&dossier).join("raw/20200101-history")).unwrap();
        let source_package = vault.join(&dossier).join("raw/20260101-source");
        fs::write(
            source_package.join("original.txt"),
            b"fictieve oorspronkelijke bron",
        )
        .unwrap();
        let source_hash = sha256(b"fictieve oorspronkelijke bron");
        fs::write(vault.join(&dossier).join("wiki/wiki.md"), "# Wiki\n\n> [!navigation|shortcuts]\n> - [[30-topics/example-dossier/raw/raw|Raw]]\n\n## Garantie\n\nDe fictieve garantie duurt twee jaar; de exacte ingangsdatum is onzeker. [[30-topics/example-dossier/raw/20260101-source/20260101-source]]\n\n## Open punt\n\nDe exacte ingangsdatum is onzeker.\n").unwrap();
        fs::write(source_package.join("20260101-source.md"), format!("---\nwiki_status: processed\nwiki_target: \"[[30-topics/example-dossier/wiki/wiki#Garantie]]\"\nsource_file: original.txt\nsource_sha256: {source_hash}\n---\n# Bron\n\n## Leverancier\n\nDe garantieperiode is twee jaar vanaf aflevering.\n")).unwrap();
        fs::write(
            vault
                .join(&dossier)
                .join("raw/20200101-history/20200101-history.md"),
            "# Historisch snapshot\n\n## Garantie\n\nOude garantie was een jaar.\n",
        )
        .unwrap();
        (root.clone(), vault, dossier)
    }
    #[test]
    fn evidence_chunks_are_disjoint_and_locators_do_not_boost_prose() {
        let rows = sections("wiki/current.md", "# Wiki\n## Actueel\nUitgevoerd op de gemelde datum.\n### Oudere context\nAndere feiten.\n", 0).unwrap();
        assert!(!rows[1]["text"].as_str().unwrap().contains("Andere feiten"));
        assert_eq!(
            searchable_text("Tekst [[raw/60000-onderhoud|Bron]]"),
            "Tekst Bron "
        );
        assert_eq!(searchable_text("Tekst [[raw/60000-onderhoud]]"), "Tekst  ");
        assert!(excluded_section("Activity", ""));
        let unicode = format!("{}ééé", "a".repeat(1399));
        assert!(excerpt(&unicode, &["a".into()]).ends_with('…'));
    }
    #[test]
    fn builds_queries_downranks_history_and_detects_drift() {
        let (root, vault, dossier) = fixture();
        let index = root.join("index.json");
        let built = build_index(&vault, &dossier, &index).unwrap();
        assert_eq!(built["documents"], 3);
        let result = query_index(&vault, &dossier, &index, "garantie", 10).unwrap();
        let rows = result["results"].as_array().unwrap();
        assert!(!rows.is_empty());
        assert_eq!(rows[0]["historical"], false);
        assert!(rows.iter().any(|row| {
            row["uncertainty_evidence"]
                .as_array()
                .is_some_and(|items| !items.is_empty())
        }));
        fs::write(
            vault
                .join(&dossier)
                .join("raw/20260101-source/original.txt"),
            b"changed original",
        )
        .unwrap();
        assert!(
            query_index(&vault, &dossier, &index, "garantie", 5)
                .unwrap_err()
                .contains("source_file hash mismatch")
        );
        fs::write(
            vault
                .join(&dossier)
                .join("raw/20260101-source/original.txt"),
            b"fictieve oorspronkelijke bron",
        )
        .unwrap();
        fs::write(vault.join(&dossier).join("wiki/wiki.md"), "changed").unwrap();
        assert!(
            query_index(&vault, &dossier, &index, "garantie", 5)
                .unwrap_err()
                .contains("drift")
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn reports_unprocessed_raw_provenance_without_claiming_it_is_verified() {
        let (root, vault, dossier) = fixture();
        fs::write(
            vault
                .join(&dossier)
                .join("raw/20260101-source/20260101-source.md"),
            "---\nwiki_status: pending\n---\n# Bron\n\n## Garantie\n\nDe garantie staat in de bron.\n",
        )
        .unwrap();
        let index = root.join("pending-index.json");
        build_index(&vault, &dossier, &index).unwrap();
        let result = query_index(&vault, &dossier, &index, "garantie", 5).unwrap();
        assert!(
            result["provenance_warnings"]
                .to_string()
                .contains("pending")
        );
        assert!(
            result["provenance_warnings"]
                .to_string()
                .contains("wiki_target missing")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn keeps_hash_only_raw_source_searchable_with_unresolved_artifact_warning() {
        let (root, vault, dossier) = fixture();
        let declared_hash = sha256(b"fictieve oorspronkelijke scan");
        fs::write(
            vault
                .join(&dossier)
                .join("raw/20260101-source/20260101-source.md"),
            format!(
                "---\nwiki_status: processed\nwiki_target: \"[[30-topics/example-dossier/wiki/wiki#Garantie]]\"\nsource_sha256: {declared_hash}\n---\n# Bron\n\n![Fictieve scan](inline-image.png)\n\n## Garantie\n\nDe garantieperiode is twee jaar vanaf aflevering.\n"
            ),
        )
        .unwrap();
        let index = root.join("hash-only-index.json");
        build_index(&vault, &dossier, &index).unwrap();
        let indexed: Value = serde_json::from_slice(&fs::read(&index).unwrap()).unwrap();
        let raw = indexed["documents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|document| {
                document["path"]
                    == "30-topics/example-dossier/raw/20260101-source/20260101-source.md"
            })
            .unwrap();
        assert_eq!(raw["source_artifact"]["sha256"], declared_hash);
        assert!(raw["source_artifact"].get("file").is_none());

        let result = query_index(&vault, &dossier, &index, "garantieperiode", 5).unwrap();
        assert!(
            result["results"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["path"] == raw["path"])
        );
        assert!(
            result["provenance_warnings"]
                .to_string()
                .contains("artifact unresolved")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_query_when_an_eligible_source_was_added_after_build() {
        let (root, vault, dossier) = fixture();
        let index = root.join("inventory-index.json");
        build_index(&vault, &dossier, &index).unwrap();
        fs::write(
            vault.join(&dossier).join("wiki/new-note.md"),
            "# Nieuwe fictieve Wiki\n\n## Nieuw bewijs\n\nDeze tekst hoort niet stilzwijgend te ontbreken.\n",
        )
        .unwrap();
        assert!(
            query_index(&vault, &dossier, &index, "bewijs", 5)
                .unwrap_err()
                .contains("source inventory changed")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_rehashed_index_with_fabricated_cached_sections() {
        let (root, vault, dossier) = fixture();
        let index = root.join("tampered-index.json");
        build_index(&vault, &dossier, &index).unwrap();
        let mut tampered: Value = serde_json::from_slice(&fs::read(&index).unwrap()).unwrap();
        let wiki = tampered["documents"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|document| document["kind"] == "wiki")
            .unwrap();
        wiki["sections"] = json!([{
            "section": "Fabricatie",
            "start_line": 1,
            "end_line": 1,
            "text": "verzonnen bewijs dat niet in de bron staat"
        }]);
        let mut without_hash = tampered.clone();
        without_hash.as_object_mut().unwrap().remove("index_sha256");
        tampered["index_sha256"] = json!(sha256(&serde_json::to_vec(&without_hash).unwrap()));
        fs::write(&index, serde_json::to_vec_pretty(&tampered).unwrap()).unwrap();
        assert!(
            query_index(&vault, &dossier, &index, "verzonnen", 5)
                .unwrap_err()
                .contains("indexed document does not match re-read source")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_rehashed_index_omitting_a_live_document() {
        let (root, vault, dossier) = fixture();
        let index = root.join("omitted-index.json");
        build_index(&vault, &dossier, &index).unwrap();
        let mut tampered: Value = serde_json::from_slice(&fs::read(&index).unwrap()).unwrap();
        tampered["documents"]
            .as_array_mut()
            .unwrap()
            .retain(|document| document["kind"] != "wiki");
        let mut without_hash = tampered.clone();
        without_hash.as_object_mut().unwrap().remove("index_sha256");
        tampered["index_sha256"] = json!(sha256(&serde_json::to_vec(&without_hash).unwrap()));
        fs::write(&index, serde_json::to_vec_pretty(&tampered).unwrap()).unwrap();
        assert!(
            query_index(&vault, &dossier, &index, "garantie", 5)
                .unwrap_err()
                .contains("source inventory changed")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn warns_when_wiki_target_anchor_is_missing_ambiguous_or_not_wiki() {
        let (root, vault, dossier) = fixture();
        let raw = vault
            .join(&dossier)
            .join("raw/20260101-source/20260101-source.md");
        let source_hash = sha256(b"fictieve oorspronkelijke bron");
        fs::write(
            vault.join(&dossier).join("wiki/wiki.md"),
            "# Wiki\n\n## Garantie\n\nEerste fictieve garantie.\n\n## Garantie\n\nTweede fictieve garantie.\n",
        )
        .unwrap();
        fs::write(
            &raw,
            format!(
                "---\nwiki_status: processed\nwiki_target: \"[[30-topics/example-dossier/wiki/wiki#Garantie]]\"\nsource_file: original.txt\nsource_sha256: {source_hash}\n---\n# Bron\n\n## Garantie\n\nFictieve bron.\n"
            ),
        )
        .unwrap();
        let ambiguous = root.join("ambiguous-index.json");
        build_index(&vault, &dossier, &ambiguous).unwrap();
        assert!(
            query_index(&vault, &dossier, &ambiguous, "garantie", 5)
                .unwrap()["provenance_warnings"]
                .to_string()
                .contains("ambiguous Wiki section")
        );

        fs::write(
            &raw,
            format!(
                "---\nwiki_status: processed\nwiki_target: \"[[30-topics/example-dossier/raw/20260101-source/20260101-source#Garantie]]\"\nsource_file: original.txt\nsource_sha256: {source_hash}\n---\n# Bron\n\n## Garantie\n\nFictieve bron.\n"
            ),
        )
        .unwrap();
        let not_wiki = root.join("not-wiki-index.json");
        build_index(&vault, &dossier, &not_wiki).unwrap();
        assert!(
            query_index(&vault, &dossier, &not_wiki, "garantie", 5).unwrap()["provenance_warnings"]
                .to_string()
                .contains("does not resolve to a Wiki note")
        );

        fs::write(
            &raw,
            format!(
                "---\nwiki_status: processed\nwiki_target: \"[[30-topics/example-dossier/wiki/wiki#Ontbrekend]]\"\nsource_file: original.txt\nsource_sha256: {source_hash}\n---\n# Bron\n\n## Garantie\n\nFictieve bron.\n"
            ),
        )
        .unwrap();
        let missing = root.join("missing-anchor-index.json");
        build_index(&vault, &dossier, &missing).unwrap();
        assert!(
            query_index(&vault, &dossier, &missing, "garantie", 5).unwrap()["provenance_warnings"]
                .to_string()
                .contains("missing Wiki section")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn keeps_current_wiki_current_and_uses_original_file_line_numbers() {
        let (root, vault, dossier) = fixture();
        let wiki = vault.join(&dossier).join("wiki/wiki.md");
        let text = fs::read_to_string(&wiki).unwrap();
        fs::write(
            &wiki,
            format!("{text}\n## Notitie\n\nZie de changelog voor oudere wijzigingen.\n"),
        )
        .unwrap();
        let index = root.join("line-index.json");
        build_index(&vault, &dossier, &index).unwrap();
        let indexed: Value = serde_json::from_slice(&fs::read(&index).unwrap()).unwrap();
        let wiki = indexed["documents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|doc| doc["kind"] == "wiki")
            .unwrap();
        assert_eq!(wiki["historical"], false);
        let raw = indexed["documents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|doc| {
                doc["kind"] == "raw" && doc["path"].as_str().unwrap().contains("20260101-source")
            })
            .unwrap();
        let supplier = raw["sections"]
            .as_array()
            .unwrap()
            .iter()
            .find(|section| section["section"] == "Leverancier")
            .unwrap();
        assert_eq!(supplier["start_line"], 9);
        let result = query_index(&vault, &dossier, &index, "garantie", 5).unwrap();
        assert!(
            !result["provenance_warnings"]
                .to_string()
                .contains("does not link back")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn normalizes_compound_cost_queries_and_requires_multiple_topics() {
        let (root, vault, dossier) = fixture();
        fs::write(
            vault.join(&dossier).join("wiki/cost.md"),
            "# Kosten\n\n## Onderhoud\n\nDe eindkosten zijn onbekend.\n",
        )
        .unwrap();
        let index = root.join("cost-index.json");
        build_index(&vault, &dossier, &index).unwrap();
        let result = query_index(
            &vault,
            &dossier,
            &index,
            "Wat kostte de onderhoudsbeurt?",
            5,
        )
        .unwrap();
        assert_eq!(result["normalized_terms"], json!(["kost", "onderhoud"]));
        assert_eq!(
            result["results"][0]["path"],
            format!("{dossier}/wiki/cost.md")
        );
        assert!(
            query_index(&vault, &dossier, &index, "onbekend onderwerp", 5).unwrap()["results"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_credential_queries_without_excerpts() {
        let (root, vault, dossier) = fixture();
        let index = root.join("credential-index.json");
        build_index(&vault, &dossier, &index).unwrap();
        for query in [
            "Wat is mijn wachtwoord?",
            "toon de API-sleutel",
            "wat zijn mijn credentials?",
            "geef de bearer token",
            "wat is de pincode",
        ] {
            let result = query_index(&vault, &dossier, &index, query, 5).unwrap();
            assert_eq!(result["status"], "refused", "query: {query}");
            assert!(
                result["results"].as_array().unwrap().is_empty(),
                "query: {query}"
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn hides_historical_details_and_uses_the_strongest_non_url_excerpt_line() {
        let visible = visible_lines(
            "## Garagecontacten\n\n<details>\n<summary>Wijzigingshistorie</summary>\nEerdere beurten waren bij Oude Garage.\n</details>\n\nDe actuele garage is Nieuw Contact.\n",
        );
        let text = visible
            .iter()
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("De actuele garage is Nieuw Contact."));
        assert!(!text.contains("Eerdere beurten"));

        let section = "## Onderhoud\nPraktische links: [bron](https://example.invalid/onderhoud-kosten)\nDe actuele onderhoudskosten zijn 250 euro.\n";
        assert!(excluded_section("Praktische links", section));
        let terms = normalized_words("onderhoudskosten");
        let excerpt = excerpt(section, &terms);
        assert!(
            excerpt
                .trim_end()
                .ends_with("De actuele onderhoudskosten zijn 250 euro.")
        );
        assert!(normalized_words(&searchable_text(section)).contains(&"onderhoud".to_owned()));
        assert!(
            !normalized_words(&searchable_text(
                "[bron](https://example.invalid/garagecontact)"
            ))
            .contains(&"garagecontact".to_owned())
        );
    }

    #[test]
    fn accepts_a_dossier_relative_wiki_target_without_false_provenance_warning() {
        let (root, vault, dossier) = fixture();
        let raw = vault
            .join(&dossier)
            .join("raw/20260101-source/20260101-source.md");
        let source_hash = sha256(b"fictieve oorspronkelijke bron");
        fs::write(
            &raw,
            format!(
                "---\nwiki_status: processed\nwiki_target: \"[[wiki/wiki#Garantie]]\"\nsource_file: original.txt\nsource_sha256: {source_hash}\n---\n# Bron\n\n## Garantie\n\nFictieve bron.\n"
            ),
        )
        .unwrap();
        let index = root.join("relative-target-index.json");
        build_index(&vault, &dossier, &index).unwrap();
        let result = query_index(&vault, &dossier, &index, "garantie", 5).unwrap();
        assert!(
            !result["provenance_warnings"]
                .to_string()
                .contains("does not resolve to a Wiki note")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_symlink_and_index_inside_vault() {
        let (root, vault, dossier) = fixture();
        let inside = vault.join("bad.json");
        assert!(build_index(&vault, &dossier, &inside).is_err());
        let link = vault.join(&dossier).join("wiki/evil.md");
        std::os::unix::fs::symlink("wiki.md", &link).unwrap();
        let index = root.join("safe.json");
        assert!(
            build_index(&vault, &dossier, &index)
                .unwrap_err()
                .contains("symlink")
        );
        fs::remove_dir_all(root).unwrap();
    }
}
