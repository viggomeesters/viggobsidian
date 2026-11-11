use crate::shortcut_links::{anchor_count, decode_percent};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Component, Path, PathBuf},
};

const SCOPE: &str = "99-system/contracts/navigation-scope.yaml";
const DESCRIPTION: &str =
    "existing folder-note entries and Home; archives are not a completeness gate";

pub fn run(
    root: &Path,
    json_output: bool,
    paths: &[PathBuf],
    anchors: bool,
    policy_path: Option<&Path>,
) -> Result<u8, Box<dyn std::error::Error>> {
    let report = audit(root, paths, anchors, policy_path)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "Checked {} entries: {} errors, {} warnings",
            report["checked"], report["errors"], report["warnings"]
        );
        for issue in items(&report["issues"]) {
            println!(
                "{} {}: {} — {}",
                string(&issue["severity"]).to_uppercase(),
                string(&issue["path"]),
                string(&issue["kind"]),
                string(&issue["target"])
            );
        }
    }
    Ok(u8::from(report["errors"] != 0))
}

fn audit(
    root: &Path,
    paths: &[PathBuf],
    anchors: bool,
    policy_path: Option<&Path>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let default_policy = root.join(SCOPE);
    let policy: Value =
        serde_saphyr::from_str(&fs::read_to_string(policy_path.unwrap_or(&default_policy))?)?;
    if !policy["technical"].is_array() || !policy["readme_collections"].is_array() {
        return Err("invalid navigation scope policy".into());
    }
    for rule in items(&policy["technical"]) {
        if !rule["parent"].is_null() && rule["name_pattern"] != "[0-9a-f]{64}" {
            return Err(format!(
                "unsupported technical scope pattern: {}",
                rule["name_pattern"]
            )
            .into());
        }
    }
    let mut scan = Scan {
        root,
        policy: &policy,
        inventory: None,
        anchors,
        checked: 0,
        issues: Vec::new(),
        excluded: Vec::new(),
    };
    let selected = if paths.is_empty() {
        scan.load_inventory()?;
        scan.inventory
            .as_ref()
            .unwrap()
            .files
            .iter()
            .filter(|file| scan.is_index(file))
            .cloned()
            .collect::<BTreeSet<_>>()
    } else {
        let mut selected = BTreeSet::new();
        for path in paths {
            let path = lexical(&root.join(path));
            let existing = path
                .ancestors()
                .find_map(|ancestor| ancestor.canonicalize().ok());
            if !path.starts_with(root)
                || !existing.is_some_and(|ancestor| ancestor.starts_with(root))
            {
                return Err("--paths must stay inside the vault".into());
            }
            selected.insert(path);
        }
        selected
    };
    for file in selected {
        if let Some(exception) = scan.technical(&file) {
            scan.excluded.push(exception);
        } else {
            scan.check(&file)?;
        }
    }
    let errors = scan
        .issues
        .iter()
        .filter(|issue| issue["severity"] == "error")
        .count();
    let warnings = scan.issues.len() - errors;
    Ok(json!({
        "scope": DESCRIPTION, "checked": scan.checked, "issues": scan.issues,
        "excluded": scan.excluded, "errors": errors, "warnings": warnings,
    }))
}

struct Scan<'a> {
    root: &'a Path,
    policy: &'a Value,
    inventory: Option<Inventory>,
    anchors: bool,
    checked: usize,
    issues: Vec<Value>,
    excluded: Vec<Value>,
}

struct Inventory {
    files: Vec<PathBuf>,
    visible: BTreeSet<PathBuf>,
    names: BTreeMap<std::ffi::OsString, Vec<usize>>,
    filenames: BTreeMap<std::ffi::OsString, Vec<usize>>,
}

impl Scan<'_> {
    fn load_inventory(&mut self) -> io::Result<()> {
        if self.inventory.is_some() {
            return Ok(());
        }
        let mut files = Vec::new();
        inventory(self.root, &mut files)?;
        files.sort();
        let visible = files
            .iter()
            .filter_map(|path| path.canonicalize().ok())
            .collect();
        let mut names = BTreeMap::<_, Vec<_>>::new();
        let mut filenames = BTreeMap::<_, Vec<_>>::new();
        for (index, file) in files.iter().enumerate() {
            filenames
                .entry(file.file_name().unwrap_or_default().to_owned())
                .or_default()
                .push(index);
            if markdown(file) && !source_asset(self.relative(file)) {
                names
                    .entry(file.file_stem().unwrap_or_default().to_owned())
                    .or_default()
                    .push(index);
            }
        }
        self.inventory = Some(Inventory {
            files,
            visible,
            names,
            filenames,
        });
        Ok(())
    }

    fn relative<'a>(&self, path: &'a Path) -> &'a Path {
        path.strip_prefix(self.root).unwrap_or(path)
    }

    fn technical(&self, path: &Path) -> Option<Value> {
        if !regular(self.root, path) {
            return None;
        }
        let relative = self.relative(path);
        if source_asset(relative) {
            return Some(exclusion(
                relative,
                "technical-attachments",
                "explicit preserved source assets; not canonical notes",
                Value::Null,
            ));
        }
        for rule in items(&self.policy["technical"]) {
            let owner = self.root.join(string(&rule["owner"]));
            if !owner.is_file() || !regular(self.root, &owner) {
                continue;
            }
            let exact = items(&rule["paths"]).any(|value| relative.starts_with(string(value)));
            // The maintained policy currently uses one pattern for pinned SHA-256 bundles.
            let pinned = rule["name_pattern"] == "[0-9a-f]{64}"
                && rule["parent"]
                    .as_str()
                    .and_then(|parent| relative.strip_prefix(parent).ok())
                    .and_then(|suffix| suffix.components().next())
                    .is_some_and(|part| {
                        let name = part.as_os_str().to_string_lossy();
                        name.len() == 64
                            && name
                                .bytes()
                                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    });
            if exact || pinned {
                return Some(exclusion(
                    relative,
                    "technical-storage",
                    string(&rule["reason"]),
                    rule["owner"].clone(),
                ));
            }
        }
        None
    }

    fn is_index(&self, file: &Path) -> bool {
        if !markdown(file) || self.technical(file).is_some() {
            return false;
        }
        let relative = self.relative(file);
        let depth = relative.components().count();
        let parent = file.parent().unwrap_or(self.root);
        if file.ends_with("README.md")
            && items(&self.policy["readme_collections"]).any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|p| self.relative(parent) == Path::new(p))
            })
            && fs::symlink_metadata(parent.join(format!(
                "{}.md",
                parent.file_name().unwrap_or_default().to_string_lossy()
            )))
            .is_err()
            && regular(self.root, file)
        {
            return true;
        }
        (depth == 1
            && file
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case("home.md")))
            || (depth <= 4
                && file.file_stem() == parent.file_name()
                && !(depth > 3 && relative.starts_with("60-raw"))
                && !relative.components().any(|part| {
                    matches!(
                        part.as_os_str().to_str(),
                        Some("imported" | "paperclip-bron")
                    )
                }))
    }

    fn wiki(&self, file: &Path, target: &str) -> Vec<PathBuf> {
        let inventory = self
            .inventory
            .as_ref()
            .expect("wikilinks require the inventory");
        let target = target.strip_suffix(".md").unwrap_or(target);
        if !target.contains('/') {
            let name = std::ffi::OsStr::new(target);
            return inventory
                .names
                .get(name)
                .or_else(|| inventory.filenames.get(name))
                .into_iter()
                .flatten()
                .map(|&index| inventory.files[index].clone())
                .collect();
        }
        let filename = format!("{target}.md");
        let mut exact = BTreeSet::new();
        for base in [self.root, file.parent().unwrap_or(self.root)] {
            for path in [base.join(target), base.join(&filename)] {
                if let Ok(path) = path.canonicalize()
                    && inventory.visible.contains(&path)
                {
                    exact.insert(path);
                }
            }
        }
        if !exact.is_empty() {
            return exact.into_iter().collect();
        }
        inventory
            .files
            .iter()
            .filter(|path| path.ends_with(target) || path.ends_with(&filename))
            .cloned()
            .collect()
    }

    fn issue(&mut self, file: &Path, kind: &str, target: &str, severity: &str) {
        self.issues.push(json!({"path": self.relative(file), "kind": kind, "target": target, "severity": severity}));
    }

    fn check(&mut self, file: &Path) -> io::Result<()> {
        self.checked += 1;
        if !regular(self.root, file) {
            self.issue(
                file,
                "symlink-entry",
                &self.relative(file).to_string_lossy(),
                "error",
            );
            return Ok(());
        }
        if !file.is_file() {
            self.issue(
                file,
                "missing-index",
                &self.relative(file).to_string_lossy(),
                "error",
            );
            return Ok(());
        }
        let source = fs::read_to_string(file)?;
        if source.trim().is_empty() {
            self.issue(
                file,
                "empty-entry",
                &file.file_stem().unwrap_or_default().to_string_lossy(),
                "warning",
            );
            return Ok(());
        }
        let body = visible_body(&source);
        let mut linked = BTreeSet::new();
        for raw in wiki_links(&body) {
            let (target, anchor) = wiki_target(raw);
            let candidates = if target.is_empty() {
                vec![file.to_path_buf()]
            } else {
                self.load_inventory()?;
                self.wiki(file, target)
            };
            let candidate = match candidates.as_slice() {
                [] => {
                    self.issue(file, "broken-wikilink", raw, "error");
                    continue;
                }
                [candidate] => candidate,
                _ => {
                    self.issue(file, "ambiguous-wikilink", raw, "error");
                    continue;
                }
            };
            if let Some(anchor) = anchor.filter(|_| self.anchors) {
                let count = match candidate.extension().and_then(|ext| ext.to_str()) {
                    Some("md") => Some(anchor_count(&fs::read_to_string(candidate)?, anchor)),
                    Some("base") => {
                        match serde_saphyr::from_str::<Value>(&fs::read_to_string(candidate)?) {
                            Ok(base) => Some(base_count(&base, anchor)),
                            Err(_) => {
                                self.issue(file, "invalid-base-view-source", raw, "error");
                                continue;
                            }
                        }
                    }
                    _ => {
                        self.issue(file, "unchecked-non-markdown-anchor", raw, "warning");
                        None
                    }
                };
                if let Some(count) = count
                    && count != 1
                {
                    self.issue(
                        file,
                        if count == 0 {
                            "missing-anchor"
                        } else {
                            "ambiguous-anchor"
                        },
                        raw,
                        "error",
                    );
                    continue;
                }
            }
            linked.insert(resolved(candidate));
        }
        for url in local_links(&body) {
            let Some(destination) = local_destination(self.root, file, url) else {
                continue;
            };
            let canonical = resolved(&destination);
            if url.starts_with("obsidian:") && canonical.strip_prefix(self.root).is_ok_and(hidden) {
                self.issue(file, "hidden-obsidian-target", url, "error");
            } else if destination.starts_with(self.root) && !regular(self.root, &destination) {
                self.issue(file, "symlink-local-link", url, "error");
            } else if !destination.exists() {
                self.issue(file, "broken-local-link", url, "error");
            } else if !url.contains('#') {
                linked.insert(canonical);
            }
        }
        self.child_rows(file, &body, &linked)
    }

    fn child_rows(
        &mut self,
        file: &Path,
        body: &str,
        linked: &BTreeSet<PathBuf>,
    ) -> io::Result<()> {
        // A preserved README inside a dated Raw package documents the source.
        // The matching foldernote owns navigation. Link validation has already run.
        if file.ends_with("README.md")
            && let Some(package) = file.parent()
            && package
                .parent()
                .is_some_and(|parent| parent.ends_with("raw"))
            && let Some(name) = package.file_name().and_then(|name| name.to_str())
            && crate::package_name(name)
            && regular(self.root, &package.join(format!("{name}.md")))
        {
            return Ok(());
        }
        let mut sections = body.split_inclusive('\n').peekable();
        let mut has_contents = false;
        let mut interrupted = false;
        while let Some(line) = sections.next() {
            if matches!(
                line.trim_end(),
                "## Contents" | "## Subfolders" | "## Inhoud" | "## Technisch onderhoud"
            ) {
                has_contents = true;
                let mut section = String::new();
                while let Some(next) = sections
                    .next_if(|line| !line.starts_with("## ") && !line.starts_with("> [!agents]"))
                {
                    section.push_str(next);
                }
                interrupted |= tables(&section).1;
            }
        }
        if !has_contents {
            return Ok(());
        }
        if interrupted {
            self.issue(
                file,
                "interrupted-child-table",
                "rows after a blank line lack a table header",
                "error",
            );
        }
        let rows = tables(body).0;
        let mut row_targets: BTreeSet<_> = local_links(&rows)
            .filter_map(|url| local_destination(self.root, file, url))
            .map(|path| resolved(&path))
            .collect();
        for raw in wiki_links(&rows) {
            let target = wiki_target(raw).0;
            if !target.is_empty()
                && let [candidate] = self.wiki(file, target).as_slice()
            {
                row_targets.insert(resolved(candidate));
            }
        }
        let parent = file.parent().unwrap_or(self.root);
        let mut children = fs::read_dir(parent)?.collect::<Result<Vec<_>, _>>()?;
        children.sort_by_key(fs::DirEntry::file_name);
        for entry in children {
            let child = entry.path();
            if hidden(Path::new(&entry.file_name())) {
                continue;
            }
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                self.issue(
                    file,
                    "symlink-entry",
                    &self.relative(&child).to_string_lossy(),
                    "error",
                );
                continue;
            }
            if !kind.is_dir() {
                continue;
            }
            if let Some(exception) = self.technical(&child) {
                self.excluded.push(exception);
                continue;
            }
            let name = entry.file_name();
            let note = child.join(format!("{}.md", name.to_string_lossy()));
            if [&child, &note, &child.join("README.md")]
                .iter()
                .any(|path| row_targets.contains(&resolved(path)))
            {
                continue;
            }
            if let Some(exception) = self.child_exception(file, &child, linked)? {
                self.excluded.push(exception);
            } else {
                self.issue(
                    file,
                    "missing-child-row",
                    &self.relative(&child).to_string_lossy(),
                    "warning",
                );
            }
        }
        Ok(())
    }

    fn child_exception(
        &self,
        index: &Path,
        child: &Path,
        linked: &BTreeSet<PathBuf>,
    ) -> io::Result<Option<Value>> {
        let rule = &self.policy["child_rows"][self.relative(index).to_string_lossy().as_ref()];
        let relative = self.relative(child);
        for route in items(&rule["routes"]) {
            let destination = index.parent().unwrap_or(self.root).join(string(route));
            if destination.parent() == Some(child)
                && markdown(&destination)
                && destination.is_file()
                && regular(self.root, &destination)
                && linked.contains(&resolved(&destination))
            {
                return Ok(Some(exclusion(
                    relative,
                    "alternate-child-route",
                    "Existing link to the designated child note.",
                    json!(self.relative(&destination)),
                )));
            }
        }
        let name = child.file_name().unwrap_or_default().to_string_lossy();
        let owner = &rule["storage"][name.as_ref()];
        if let Some(owner_path) = owner.as_str() {
            let path = self.root.join(owner_path);
            if path.is_file() && regular(self.root, &path) {
                return Ok(Some(exclusion(
                    relative,
                    "child-storage",
                    "Designated storage or archive with a documented owner.",
                    owner.clone(),
                )));
            }
        }
        if items(&rule["empty"]).any(|item| item.as_str() == Some(&name))
            && fs::read_dir(child)?.next().is_none()
        {
            return Ok(Some(exclusion(
                relative,
                "empty-legacy-child",
                "Designated legacy folder contains no files or subfolders.",
                json!(self.relative(index)),
            )));
        }
        Ok(None)
    }
}

fn exclusion(path: &Path, profile: &str, reason: &str, owner: Value) -> Value {
    json!({"path": path, "profile": profile, "reason": reason, "owner": owner})
}

fn items(value: &Value) -> impl Iterator<Item = &Value> {
    value.as_array().into_iter().flatten()
}
fn string(value: &Value) -> &str {
    value.as_str().unwrap_or("")
}
fn markdown(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "md")
}
fn source_asset(path: &Path) -> bool {
    path.components()
        .any(|part| part.as_os_str() == "_source-assets")
}
fn hidden(path: &Path) -> bool {
    path.components()
        .any(|part| part.as_os_str().to_string_lossy().starts_with('.'))
}

fn regular(root: &Path, path: &Path) -> bool {
    path.starts_with(root)
        && path
            .ancestors()
            .take_while(|parent| *parent != root)
            .all(|part| !fs::symlink_metadata(part).is_ok_and(|meta| meta.is_symlink()))
}

fn resolved(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| lexical(path))
}

fn lexical(path: &Path) -> PathBuf {
    let mut normal = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                normal.pop();
            }
            Component::CurDir => {}
            _ => normal.push(component),
        }
    }
    normal
}

fn inventory(folder: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            if !hidden(Path::new(&entry.file_name())) {
                inventory(&entry.path(), files)?;
            }
        } else if !kind.is_symlink() || !entry.path().is_dir() {
            files.push(entry.path());
        }
    }
    Ok(())
}

pub(crate) fn base_count(base: &Value, anchor: &str) -> usize {
    let name = decode_percent(anchor);
    items(&base["views"])
        .filter(|view| view["name"].as_str() == Some(&name))
        .count()
}

fn wiki_target(raw: &str) -> (&str, Option<&str>) {
    let target = raw.split('|').next().unwrap_or(raw).trim_end_matches('\\');
    target
        .split_once('#')
        .map_or((target, None), |(note, anchor)| (note, Some(anchor)))
}

pub(crate) fn wiki_links(body: &str) -> impl Iterator<Item = &str> {
    body.split("[[").skip(1).filter_map(|tail| {
        let (link, _) = tail.split_once("]]")?;
        (!link.is_empty() && !link.contains(']')).then_some(link)
    })
}

fn local_links(body: &str) -> impl Iterator<Item = &str> {
    body.match_indices("](").filter_map(move |(start, _)| {
        let before = &body[..start];
        let bracket = before.rfind(['[', ']', '\n'])?;
        if before.as_bytes()[bracket] != b'[' {
            return None;
        }
        let tail = &body[start + 2..];
        let (url, rest) = if let Some(angle) = tail.strip_prefix('<') {
            angle.split_once('>')?
        } else {
            let end = tail.find(|c: char| c == ')' || c.is_whitespace())?;
            tail.split_at(end)
        };
        let title_closed = rest.starts_with(char::is_whitespace)
            && rest
                .trim_start()
                .strip_prefix('"')
                .and_then(|title| title.split_once('"'))
                .is_some_and(|(_, end)| end.starts_with(')'));
        (!url.is_empty() && !url.contains('\n') && (rest.starts_with(')') || title_closed))
            .then_some(url)
    })
}

fn local_destination(root: &Path, file: &Path, url: &str) -> Option<PathBuf> {
    let url = url.split('#').next()?;
    let (path, query) = url.split_once('?').unwrap_or((url, ""));
    if path == "obsidian://open" || path == "obsidian://open/" {
        let mut values = BTreeMap::new();
        for (key, value) in query
            .split('&')
            .filter_map(|part| part.split_once('='))
            .filter(|(_, value)| !value.is_empty())
        {
            values
                .entry(key)
                .or_insert_with(|| decode_percent(&value.replace('+', " ")));
        }
        if let Some(path) = values.get("path") {
            return Some(PathBuf::from(path));
        }
        if values.get("vault").map(String::as_str).unwrap_or("vault")
            != root.file_name()?.to_str()?
        {
            return None;
        }
        let mut path = root.join(values.get("file")?);
        if path.extension().is_none() {
            path.set_extension("md");
        }
        return Some(path);
    }
    if let Some(rest) = path.strip_prefix("file:") {
        let path = if let Some(authority) = rest.strip_prefix("//") {
            &authority[authority.find('/')?..]
        } else {
            rest
        };
        return Some(PathBuf::from(decode_percent(path)));
    }
    if path.is_empty() || path.split('/').next()?.contains(':') || path.starts_with("//") {
        return None;
    }
    Some(file.parent()?.join(decode_percent(path)))
}

pub(crate) fn quote_content(mut line: &str) -> &str {
    loop {
        let trimmed = line.trim_start_matches(' ');
        if line.len() - trimmed.len() > 3 {
            return line;
        }
        let Some(rest) = trimmed.strip_prefix('>') else {
            return line;
        };
        line = rest.strip_prefix(' ').unwrap_or(rest);
    }
}

pub(crate) fn visible_blocks(source: &str) -> String {
    let source = source
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n").map(|(_, body)| body))
        .unwrap_or(source);
    let mut result = String::new();
    let mut fence = None;
    for line in source.lines() {
        let content = quote_content(line);
        let trimmed = content.trim_start_matches(' ');
        let marker = (content.len() - trimmed.len() <= 3)
            .then(|| trimmed.chars().next())
            .flatten()
            .filter(|ch| matches!(ch, '`' | '~'));
        let count = marker.map_or(0, |marker| {
            trimmed.chars().take_while(|ch| *ch == marker).count()
        });
        if let Some((opening, length)) = fence {
            if marker == Some(opening) && count >= length && trimmed[count..].trim().is_empty() {
                fence = None;
            }
            continue;
        }
        if count >= 3 {
            fence = marker.map(|marker| (marker, count));
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }
    let mut rest = result.as_str();
    let mut clean = String::new();
    while let Some((before, after)) = rest.split_once("<!--") {
        clean.push_str(before);
        let Some((_, after)) = after.split_once("-->") else {
            return clean;
        };
        rest = after;
    }
    clean.push_str(rest);
    clean
}

pub(crate) fn visible_body(source: &str) -> String {
    let clean = visible_blocks(source);
    let mut body = String::new();
    for line in clean.lines() {
        let mut rest = line;
        while let Some((before, after)) = rest.split_once('`') {
            body.push_str(before);
            let length = 1 + after.bytes().take_while(|&byte| byte == b'`').count();
            let marker = "`".repeat(length);
            if let Some((_, tail)) = after[length - 1..].split_once(&marker) {
                rest = tail;
            } else {
                body.push_str(&marker);
                rest = &after[length - 1..];
            }
        }
        body.push_str(rest);
        body.push('\n');
    }
    body
}

fn tables(body: &str) -> (String, bool) {
    let mut rows = String::new();
    let mut orphan = false;
    let mut lines = body.lines().peekable();
    while let Some(line) = lines.next() {
        let row =
            |line: &str| line.starts_with('|') && line.trim_end().ends_with('|') && line.len() > 2;
        if !row(line) {
            continue;
        }
        let mut block = vec![line];
        while let Some(next) = lines.next_if(|next| row(next)) {
            block.push(next);
        }
        if block.get(1).is_some_and(|delimiter| {
            delimiter
                .trim()
                .trim_matches('|')
                .bytes()
                .all(|byte| matches!(byte, b' ' | b':' | b'|' | b'-'))
        }) {
            for line in block {
                rows.push_str(line);
                rows.push('\n');
            }
        } else {
            orphan = true;
        }
    }
    let interrupted = orphan && !rows.is_empty();
    (rows, interrupted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);
            let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = crate::test_root();
            let path = root.join(format!(
                ".tmp/index-nav-{}-{nonce}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn write(&self, name: &str, content: &str) {
            let path = self.0.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn base_views_and_child_exceptions_keep_real_failures_visible() {
        let fixture = Fixture::new();
        fixture.write(SCOPE, "readme_collections: []\ntechnical: []\nchild_rows:\n  collection/collection.md:\n    routes: [child/start.md]\n    storage: {store: owner.md}\n    empty: [old]\n");
        fixture.write("owner.md", "# Owner\n");
        fixture.write("collection/child/start.md", "# Live\n");
        fixture.write("collection/store/data.json", "{}");
        fs::create_dir(fixture.0.join("collection/old")).unwrap();
        fixture.write(
            "views.base",
            "views:\n- name: Inbox packages\n- name: Repeat\n- name: Repeat\n",
        );
        fixture.write("collection/collection.md", "## Contents\n\n| Link |\n|---|\n\n## Route\n\n[[collection/child/start#Live]] ![[views.base#Inbox%20packages]]\n");
        let clean = audit(&fixture.0, &[], true, None).unwrap();
        assert_eq!(clean["checked"], 1);
        assert_eq!(clean["issues"], json!([]));
        assert_eq!(
            items(&clean["excluded"])
                .map(|item| string(&item["profile"]))
                .collect::<Vec<_>>(),
            [
                "alternate-child-route",
                "empty-legacy-child",
                "child-storage"
            ],
        );

        fixture.write("collection/collection.md", "## Contents\n\n| Link |\n|---|\n\n## Route\n\n[[collection/child/start#Missing]] ![[views.base#Missing]] ![[views.base#Repeat]]\n[Missing file](missing.pdf) [Malformed](ignored file\n");
        fixture.write("collection/old/new.txt", "no longer empty");
        fs::remove_file(fixture.0.join("owner.md")).unwrap();
        let broken = audit(&fixture.0, &[], true, None).unwrap();
        assert_eq!(broken["errors"], 4);
        assert_eq!(broken["warnings"], 3);
        assert_eq!(broken["excluded"], json!([]));
        assert_eq!(
            items(&broken["issues"])
                .map(|item| string(&item["kind"]))
                .collect::<Vec<_>>(),
            [
                "missing-anchor",
                "missing-anchor",
                "ambiguous-anchor",
                "broken-local-link",
                "missing-child-row",
                "missing-child-row",
                "missing-child-row"
            ],
        );
    }

    #[test]
    fn raw_package_readme_checks_links_without_requiring_navigation_rows() {
        let fixture = Fixture::new();
        fixture.write(
            SCOPE,
            "readme_collections: []\ntechnical: []\nchild_rows: {}\n",
        );
        let folder = "dossier/raw/20261108-example";
        let readme = format!("{folder}/README.md");
        fixture.write(&format!("{folder}/20261108-example.md"), "# Package\n");
        fixture.write(&format!("{folder}/photos/file.txt"), "source");
        fixture.write(
            &readme,
            "## Inhoud\n\n- Bronfoto’s\n- [Ongeldige link](absent.pdf)\n",
        );
        let report = audit(&fixture.0, &[PathBuf::from(&readme)], true, None).unwrap();
        assert_eq!(report["warnings"], 0);
        assert_eq!(report["errors"], 1);
        assert_eq!(report["issues"][0]["kind"], "broken-local-link");
    }

    #[test]
    fn parses_live_local_links_without_unterminated_examples() {
        let quoted = visible_body(
            "> > [!info]- Meer\n> > ````text\n> > [[legacy-missing]]\n> > ```\n> > [[still-in-source]]\n> > ````\n> [[real-target]]\n",
        );
        assert_eq!(wiki_links(&quoted).collect::<Vec<_>>(), ["real-target"]);
        assert_eq!(
            crate::shortcut_links::anchor_count(
                "> ### Bronregister\n> > ```text\n> > ## Bronregister\n> > ```\n",
                "Bronregister"
            ),
            1
        );
        let body = visible_body(
            "[Plain](plain.md) [Angle](<space name.md>) [Title](another.md \"Title\")\n[Incomplete](no closing delimiter\n[Angle](<no closing delimiter>\n`[Example](example.md)`\n<!-- [Comment](comment.md) -->\n```md\n[Fence](fence.md)\n```\n",
        );
        assert_eq!(
            local_links(&body).collect::<Vec<_>>(),
            ["plain.md", "space name.md", "another.md"]
        );
    }

    #[test]
    fn default_scope_keeps_home_and_owned_indexes_outside_archives() {
        let fixture = Fixture::new();
        fixture.write(
            SCOPE,
            "readme_collections: [legacy.dotted]\ntechnical: []\n",
        );
        for name in [
            "HOME.md",
            "legacy.dotted/README.md",
            "regular/regular.md",
            "one/two/three/three.md",
            "one/two/three/four/four.md",
            ".raw/hidden/hidden.md",
            "30-topics/imported/archive/archive.md",
            "60-raw/old/history/history.md",
            "10-daily/tasks/generated-hub.md",
        ] {
            fixture.write(name, "[[missing]]");
        }
        let report = audit(&fixture.0, &[], true, None).unwrap();
        let paths: Vec<_> = items(&report["issues"])
            .map(|issue| string(&issue["path"]))
            .collect();
        assert_eq!(
            paths,
            [
                "HOME.md",
                "legacy.dotted/README.md",
                "one/two/three/three.md",
                "regular/regular.md"
            ]
        );
        fixture.write("legacy.dotted/legacy.dotted.md", "[[missing]]");
        let report = audit(&fixture.0, &[], true, None).unwrap();
        assert!(
            items(&report["issues"]).any(|issue| issue["path"] == "legacy.dotted/legacy.dotted.md")
        );
        assert!(!items(&report["issues"]).any(|issue| issue["path"] == "legacy.dotted/README.md"));
    }

    #[test]
    fn explicit_paths_deduplicate_report_missing_and_control_anchor_checks() {
        let fixture = Fixture::new();
        fixture.write(SCOPE, "readme_collections: []\ntechnical: []\n");
        fixture.write("unrelated/unrelated.md", "[[missing]]");
        fixture.write("deep/one/two/three/entry.md", "[[#Missing]]");
        let note = "deep/one/two/three/entry.md";
        let paths = [
            PathBuf::from(note),
            fixture.0.join(note),
            PathBuf::from("missing.md"),
        ];
        let report = audit(&fixture.0, &paths, true, None).unwrap();
        assert_eq!(report["checked"], 2);
        assert_eq!(
            items(&report["issues"])
                .map(|item| string(&item["kind"]))
                .collect::<Vec<_>>(),
            ["missing-anchor", "missing-index"]
        );
        assert_eq!(audit(&fixture.0, &paths, false, None).unwrap()["errors"], 1);
        assert!(audit(&fixture.0, &[PathBuf::from("../outside.md")], true, None).is_err());
    }
}
