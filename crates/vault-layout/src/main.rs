use serde_json::{Value, json};
use std::{
    cell::OnceCell,
    collections::{BTreeMap, BTreeSet},
    env, fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

mod activity_delivery;
mod activity_layout;
mod dossier_tables;
mod event_header;
mod index_navigation;
mod link_style;
mod mail_draft;
mod mail_manifest;
mod navigation_contract;
mod project_codes;
mod raw_processing;
mod raw_task;
mod shortcut_links;
mod source_inventory;
mod wiki_layout;
mod work_intake;
mod work_package;
mod write_set;

const USAGE: &str = "Usage: vault-layout [--json] [--vault ROOT] ([--strict] FOLDER | --tiles-only [FOLDER] | --indexes [--no-anchors] [--scope-policy FILE] [--paths NOTE ...] | --link-style | --fix-link-style | --resolve-code CODE [PROJECT] | --stage-raw-task RAW | --check-all | --check-work | --new-work-task RAW < INPUT.json)";
const SCHEMA: &str = "99-system/contracts/vault-schema.yaml";
const TILE_CSS: &str = ".obsidian/snippets/folder-note.css";
const TASK_SUMMARY: &str = "```vault-task-summary";
const OPEN_TASKS: &str = "![[vault-dossier.base#Open tasks]]";
const RAW_SUMMARY: &str = "```vault-raw-summary";
const RAW_SUMMARY_PLUGIN: &str = "vault-raw-summary";
const TEMPLATES_INDEX: &str = "90-templates/90-templates.md";
const TEMPLATES_BASE: &str = "99-system/vault/vault-navigation/standaard/wiki/vault-templates.base";
const TEMPLATE_FILTERS: &[&str] = &[
    r#"file.folder == "90-templates""#,
    r#"file.ext == "md""#,
    r#"file.name != "90-templates""#,
    r#"!file.name.contains(".sync-conflict-")"#,
];

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("vault-layout: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<u8, Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1).peekable();
    let mut vault = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut folder = None;
    let mut json_output = false;
    let mut tiles_only = false;
    let mut indexes = false;
    let mut paths = Vec::new();
    let mut anchors = true;
    let mut scope_policy = None;
    let mut resolve_code = None;
    let mut stage_raw_task = false;
    let mut strict = false;
    let mut check_work = false;
    let mut check_all = false;
    let mut new_work_task = false;
    let mut link_style = None;
    let mut write_set_mode = false;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--help" | "-h") => {
                println!("{USAGE}");
                return Ok(0);
            }
            Some("--json") => json_output = true,
            Some("--tiles-only") => tiles_only = true,
            Some("--strict") => strict = true,
            Some("--check-work") => check_work = true,
            Some("--check-all") => check_all = true,
            Some("--write-set") if !write_set_mode => write_set_mode = true,
            Some("--new-work-task") => new_work_task = true,
            Some("--link-style") if link_style.is_none() => link_style = Some(false),
            Some("--fix-link-style") if link_style.is_none() => link_style = Some(true),
            Some("--indexes") => indexes = true,
            Some("--stage-raw-task") if !stage_raw_task => stage_raw_task = true,
            Some("--no-anchors") => anchors = false,
            Some("--scope-policy") if scope_policy.is_none() => {
                scope_policy = Some(PathBuf::from(args.next().ok_or(USAGE)?));
            }
            Some("--paths") => {
                let start = paths.len();
                while let Some(path) = args.next_if(|arg| !arg.to_string_lossy().starts_with('-')) {
                    paths.push(PathBuf::from(path));
                }
                if paths.len() == start {
                    return Err(USAGE.into());
                }
            }
            Some("--resolve-code") if resolve_code.is_none() => {
                resolve_code = Some(args.next().ok_or(USAGE)?.into_string().map_err(|_| USAGE)?);
            }
            Some("--vault") => vault = args.next().ok_or(USAGE)?.into(),
            Some(option) if option.starts_with('-') => return Err(USAGE.into()),
            _ if folder.is_none() => folder = Some(PathBuf::from(arg)),
            _ => return Err(USAGE.into()),
        }
    }
    let vault = vault.canonicalize()?;
    if write_set_mode {
        if folder.is_some()
            || tiles_only
            || indexes
            || !paths.is_empty()
            || !anchors
            || scope_policy.is_some()
            || resolve_code.is_some()
            || stage_raw_task
            || strict
            || check_work
            || check_all
            || new_work_task
            || link_style.is_some()
        {
            return Err(USAGE.into());
        }
        return write_set::run(&vault, serde_json::from_reader(io::stdin().lock())?);
    }
    if let Some(apply) = link_style {
        if check_work
            || check_all
            || new_work_task
            || strict
            || tiles_only
            || indexes
            || !paths.is_empty()
            || resolve_code.is_some()
            || stage_raw_task
            || folder.is_some()
            || !anchors
            || scope_policy.is_some()
        {
            return Err(USAGE.into());
        }
        return link_style::run(&vault, apply, json_output);
    }
    if new_work_task {
        if check_work
            || check_all
            || strict
            || tiles_only
            || indexes
            || !paths.is_empty()
            || resolve_code.is_some()
            || stage_raw_task
            || !anchors
            || scope_policy.is_some()
        {
            return Err(USAGE.into());
        }
        let raw = folder.ok_or(USAGE)?;
        let input: Value = serde_json::from_reader(io::stdin().lock())?;
        println!("{}", work_intake::create(&vault, &raw, input)?);
        return Ok(0);
    }
    if check_work || check_all {
        if (check_work && check_all)
            || strict
            || tiles_only
            || indexes
            || !paths.is_empty()
            || resolve_code.is_some()
            || stage_raw_task
            || folder.is_some()
            || !anchors
            || scope_policy.is_some()
        {
            return Err(USAGE.into());
        }
        return if check_work {
            work_package::check(&vault)
        } else {
            work_package::check_all(&vault, false, false)
        };
    }
    if strict
        && (folder.is_none()
            || tiles_only
            || indexes
            || !paths.is_empty()
            || resolve_code.is_some()
            || stage_raw_task)
    {
        return Err(USAGE.into());
    }
    if stage_raw_task && (tiles_only || indexes || !paths.is_empty() || resolve_code.is_some()) {
        return Err(USAGE.into());
    }
    if indexes || !paths.is_empty() {
        if tiles_only || resolve_code.is_some() || folder.is_some() {
            return Err(USAGE.into());
        }
        return index_navigation::run(
            &vault,
            json_output,
            &paths,
            anchors,
            scope_policy.as_deref(),
        );
    }
    if !anchors || scope_policy.is_some() {
        return Err(USAGE.into());
    }
    if let Some(code) = resolve_code {
        if tiles_only {
            return Err(USAGE.into());
        }
        let entry = project_codes::resolve(&vault, folder.as_deref(), &code)?;
        if json_output {
            println!(
                "{}",
                json!({"id": entry.id, "key": entry.key, "path": entry.path})
            );
        } else {
            println!("{}\t{}\t{}", entry.key, entry.id, entry.path.display());
        }
        return Ok(0);
    }
    let input = match folder {
        Some(folder) => vault.join(folder),
        None if tiles_only => vault.clone(),
        None => return Err(USAGE.into()),
    };
    let folder = input.canonicalize()?;
    if (!tiles_only && folder == vault) || !folder.starts_with(&vault) || !folder.is_dir() {
        return Err("choose a directory inside the vault (root requires --tiles-only)".into());
    }
    if fs::symlink_metadata(&input)?.is_symlink() {
        return Err("the selected folder is a symbolic link".into());
    }
    let mut schema: Value = serde_saphyr::from_str(&fs::read_to_string(vault.join(SCHEMA))?)?;
    navigation_contract::apply(&vault, &mut schema)?;
    if !schema["validation_profiles"]["minimal"]["required"].is_array()
        || !schema["contracts"]["raw_package"]["required_fields"].is_array()
        || !schema["contracts"]["raw_package"]["wiki_status_values"].is_object()
        || !schema["common_fields"]["layer"]["allowed_values"].is_array()
    {
        return Err("vault schema lacks the layout contract".into());
    }

    let system_root = vault.join("99-system");
    if folder.starts_with(&system_root) {
        let path = system_root.join("contracts/system-layout.yaml");
        match fs::read_to_string(&path) {
            Ok(policy) => {
                let policy: Value = serde_saphyr::from_str(&policy)?;
                if !policy["collections"].is_array()
                    || !policy["fixed_files"].is_array()
                    || !policy["delegated"].is_object()
                {
                    return Err("invalid system layout policy".into());
                }
                schema["system_layout"] = policy;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound && !strict => {}
            Err(error) => return Err(format!("{}: {error}", path.display()).into()),
        }
    }

    let css = match fs::read_to_string(vault.join(TILE_CSS)) {
        Ok(css) => css,
        Err(error) if error.kind() == io::ErrorKind::NotFound && !tiles_only => String::new(),
        Err(error) => return Err(error.into()),
    };
    let css = without_css_comments(&css);
    let mut scan = Scan {
        vault: &vault,
        schema: &schema,
        tile_styles: tile_styles(&css),
        findings: Vec::new(),
        folders: 0,
        tiles: 0,
        shortcuts: 0,
        notes: OnceCell::new(),
        dossier_tables_checked: BTreeSet::new(),
        raw_summary_checked: false,
        strict,
    };
    if stage_raw_task {
        raw_task::stage(
            &mut scan,
            &folder,
            serde_json::from_reader(io::stdin().lock())?,
        )?;
    } else if tiles_only {
        scan.tile_folder(&folder)?;
    } else {
        let code_roots = &schema["work_project_code_roots"];
        if strict
            && strings(code_roots).any(|root| folder.starts_with(vault.join(root)))
            && let Err(message) = project_codes::validate_registry(&vault, code_roots)
        {
            scan.issue(&folder, "project-code-registry", message);
        }
        scan.folder(&folder)?;
    }
    let delegated: BTreeMap<_, _> = schema["system_layout"]["delegated"]
        .as_object()
        .into_iter()
        .flatten()
        .chain(
            schema["strict_layout_exceptions"]["daily_writer_collections"]
                .as_object()
                .into_iter()
                .flatten(),
        )
        .filter(|(path, _)| {
            let target = vault.join(path);
            !tiles_only && !stage_raw_task && target.starts_with(&folder) && target.is_dir()
        })
        .collect();
    if json_output {
        let findings: Vec<_> = scan.findings.iter().map(|finding| {
            json!({"path": finding.path, "code": finding.code, "message": finding.message})
        }).collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schema_version": 1, "folders_checked": scan.folders,
                "tiles_checked": scan.tiles, "shortcuts_checked": scan.shortcuts,
                "findings": findings, "delegated": delegated
            }))?
        );
    } else {
        for (path, reason) in delegated {
            println!("{path} [delegated] {}", reason.as_str().unwrap_or(""));
        }
        for finding in &scan.findings {
            println!(
                "{} [{}] {}",
                finding.path.display(),
                finding.code,
                finding.message
            );
        }
        println!(
            "{} folders checked, {} findings",
            scan.folders,
            scan.findings.len()
        );
        if tiles_only || scan.tiles > 0 {
            println!("{} dashboard tiles checked", scan.tiles);
        }
        if tiles_only || scan.shortcuts > 0 {
            println!("{} Wiki/Raw shortcut destinations checked", scan.shortcuts);
        }
    }
    Ok(u8::from(!scan.findings.is_empty()))
}

struct Finding {
    path: PathBuf,
    code: &'static str,
    message: String,
}

struct Scan<'a> {
    vault: &'a Path,
    schema: &'a Value,
    tile_styles: Vec<TileStyle<'a>>,
    findings: Vec<Finding>,
    folders: usize,
    tiles: usize,
    shortcuts: usize,
    notes: OnceCell<Vec<PathBuf>>,
    dossier_tables_checked: BTreeSet<&'static str>,
    raw_summary_checked: bool,
    strict: bool,
}

impl Scan<'_> {
    fn vault_collection(&self, path: &Path) -> bool {
        self.strict && path == self.vault.join("99-system/vault")
    }

    fn system_cache(&self, path: &Path) -> bool {
        path.strip_prefix(self.vault)
            .ok()
            .and_then(Path::to_str)
            .is_some_and(|path| {
                matches!(
                    path,
                    "99-system/agents/__pycache__" | "99-system/contracts/__pycache__"
                )
            })
            && self.system_path("delegated", path)
    }

    fn system_path(&self, key: &str, path: &Path) -> bool {
        let Ok(relative) = path.strip_prefix(self.vault) else {
            return false;
        };
        let relative = relative.to_string_lossy().replace('\\', "/");
        let entries = &self.schema["system_layout"][key];
        if key == "delegated" {
            return entries[relative.as_str()]
                .as_str()
                .is_some_and(|reason| !reason.trim().is_empty());
        }
        strings(entries).any(|entry| entry == relative)
    }

    fn strict_exception(&self, key: &str, path: &Path) -> bool {
        path.strip_prefix(self.vault)
            .ok()
            .and_then(Path::to_str)
            .and_then(|path| self.schema["strict_layout_exceptions"][key][path].as_str())
            .is_some_and(|reason| !reason.trim().is_empty())
    }

    fn check_raw_summary_plugin(&mut self) {
        if self.raw_summary_checked {
            return;
        }
        self.raw_summary_checked = true;
        let config = self.vault.join(".obsidian/community-plugins.json");
        let enabled = fs::read_to_string(&config)
            .ok()
            .and_then(|source| serde_json::from_str::<Vec<String>>(&source).ok())
            .is_some_and(|plugins| plugins.iter().any(|plugin| plugin == RAW_SUMMARY_PLUGIN));
        if !enabled {
            self.issue(
                &config,
                "raw-summary-plugin",
                "enable Vault Raw Summary for source and task counter blocks",
            );
        }
        let plugin = self
            .vault
            .join(".obsidian/plugins")
            .join(RAW_SUMMARY_PLUGIN);
        for file in ["manifest.json", "main.js", "styles.css"] {
            let path = plugin.join(file);
            if !path.is_file() {
                self.issue(
                    &path,
                    "raw-summary-plugin",
                    "missing Vault Raw Summary plugin file",
                );
            }
        }
    }

    fn check_dossier_tables(&mut self, dashboard: bool) {
        for (file, views) in dossier_tables::CONTRACTS {
            if (!dashboard && *file != dossier_tables::RAW_BASE)
                || !self.dossier_tables_checked.insert(file)
            {
                continue;
            }
            let path = self.vault.join(file);
            match fs::read_to_string(&path) {
                Ok(source) => {
                    for finding in dossier_tables::findings(&source, views) {
                        self.issue(&path, "dossier-table-layout", finding);
                    }
                }
                Err(error) => self.issue(&path, "dossier-table-layout", error.to_string()),
            }
        }
    }

    fn tile_folder(&mut self, folder: &Path) -> io::Result<()> {
        self.folders += 1;
        let mut entries = fs::read_dir(folder)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries {
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            let kind = entry.file_type()?;
            let path = entry.path();
            if kind.is_dir() {
                self.tile_folder(&path)?;
            } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                let source = fs::read_to_string(&path)?;
                if !source.contains("[!navigation]") && !source.contains("[!navigation|") {
                    continue;
                }
                if let Some((yaml, body)) = frontmatter(&source)
                    && let Ok(data) = serde_saphyr::from_str::<Value>(&yaml)
                {
                    self.check_tiles(&path, &data, &body)?;
                }
            }
        }
        Ok(())
    }

    fn check_tiles(&mut self, path: &Path, data: &Value, body: &str) -> io::Result<()> {
        let classes = &data["cssclasses"];
        let has_class = |class: &str| {
            strings(classes).any(|value| value == class)
                || classes
                    .as_str()
                    .is_some_and(|value| value.split_whitespace().any(|part| part == class))
        };
        let targets = || tiles(body, false).chain(tiles(body, true)).flatten();
        for target in tiles(body, true).flatten() {
            self.shortcuts += 1;
            if let Some(code) = shortcut_links::check(self.vault, &self.notes, path, target)? {
                self.issue(path, code, format!("invalid Wiki/Raw shortcut: {target}"));
            }
        }
        if !has_class("vault-dashboard") {
            if has_class("dashboard-variant-2-2") && targets().next().is_some() {
                self.issue(
                    path,
                    "missing-dashboard-class",
                    "HOME2.2 tiles require vault-dashboard",
                );
            }
            return Ok(());
        }
        for target in targets() {
            self.tiles += 1;
            if !self
                .tile_styles
                .iter()
                .any(|style| has_class(style.class) && style.matches(target))
            {
                self.issue(
                    path,
                    "default-tile-style",
                    format!("uses the default icon: {target}"),
                );
            }
        }
        let home_dashboard = has_class("dashboard-variant-2-2");
        let distinct_accents = home_dashboard || has_class("tile-distinct-accents");
        let distinct_icons = home_dashboard || has_class("tile-distinct-icons");
        if distinct_accents || distinct_icons {
            let mut accents = BTreeMap::new();
            let mut icons = BTreeMap::new();
            for target in tiles(body, false).flatten() {
                if let Some(style) = self
                    .tile_styles
                    .iter()
                    .rev()
                    .find(|style| has_class(style.class) && style.matches(target))
                {
                    let accent = style.accent.to_ascii_lowercase();
                    let repeated_accent = distinct_accents
                        .then(|| accents.insert(accent.clone(), target))
                        .flatten();
                    let repeated_icon = distinct_icons
                        .then(|| icons.insert(style.icon, target))
                        .flatten();
                    if let Some(previous) = repeated_accent {
                        self.issue(
                            path,
                            "duplicate-tile-accent",
                            format!("repeated accent {accent}: {previous} and {target}"),
                        );
                    }
                    if let Some(previous) = repeated_icon {
                        self.issue(
                            path,
                            "duplicate-tile-icon",
                            format!("repeated icon: {previous} and {target}"),
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn issue(&mut self, path: &Path, code: &'static str, message: impl Into<String>) {
        self.findings.push(Finding {
            path: path.strip_prefix(self.vault).unwrap_or(path).to_path_buf(),
            code,
            message: message.into(),
        });
    }

    fn folder(&mut self, folder: &Path) -> io::Result<()> {
        let under_raw = folder
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|name| name == "raw");
        self.folder_with_raw_parent(folder, under_raw)
    }

    fn folder_with_raw_parent(&mut self, folder: &Path, under_raw: bool) -> io::Result<()> {
        self.folder_with_depth(folder, under_raw, true)
    }

    fn folder_with_depth(
        &mut self,
        folder: &Path,
        under_raw: bool,
        recurse: bool,
    ) -> io::Result<()> {
        if self.strict
            && folder.starts_with(self.vault.join("99-system/vault"))
            && matches!(
                folder.file_name().and_then(|name| name.to_str()),
                Some("sources" | "bronnen")
            )
        {
            self.issue(
                folder,
                "vault-source-owner",
                "put source packages directly under Raw of their owning dossier",
            );
        }
        if self.system_path("delegated", folder) {
            if self.strict
                && !self.system_cache(folder)
                && let Some(entrance) = direct_wiki_note(folder)
                && !fs::symlink_metadata(&entrance).is_ok_and(|entry| entry.is_file())
            {
                self.issue(
                    &entrance,
                    "delegated-folder-note",
                    "a visible delegated folder needs its matching folder note",
                );
            }
            return Ok(());
        }
        let Some(name) = folder.file_name().and_then(|name| name.to_str()) else {
            self.issue(folder, "folder-name", "folder name must be UTF-8");
            return Ok(());
        };
        let entrance = folder.join(format!("{name}.md"));
        if self.strict_exception("daily_writer_collections", folder) {
            // These collections remain visible as tiles. Their writers own the contents.
            self.note(&entrance)?;
            return Ok(());
        }
        self.folders += 1;
        let daily_notes = folder == self.vault.join("10-daily/notes");
        let dated = date_prefix(name);
        let package = package_name(name);
        let inbox = self.vault.join("00-inbox");
        let direct_inbox_child = folder.parent() == Some(inbox.as_path());
        if self.strict
            && package
            && !under_raw
            && !direct_inbox_child
            && !self.strict_exception("producer_packages", folder)
        {
            self.issue(
                folder,
                "strict-package-location",
                "dated packages belong directly under Raw",
            );
        }
        if (under_raw || dated || (self.strict && direct_inbox_child)) && !package {
            self.issue(
                folder,
                "package-name",
                "expected YYYYMMDD-note-name with a valid date",
            );
        }

        let mut children = Vec::new();
        let mut entries = fs::read_dir(folder)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries {
            let path = entry.path();
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                self.issue(
                    &path,
                    "symlink",
                    "symbolic links are outside the layout contract",
                );
            } else if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            } else if kind.is_dir() {
                if !self.system_cache(&path) {
                    children.push(path);
                }
            } else if package && under_raw && path != entrance && kind.is_file() {
                self.raw_source_note(&path)?;
            } else if path != entrance
                && self.vault_collection(folder)
                && !(kind.is_file() && self.system_path("fixed_files", &path))
            {
                self.issue(
                    &path,
                    "loose-file",
                    "Vault collection allows only vault.md and child folders",
                );
            } else if !package
                && path != entrance
                && !(kind.is_file()
                    && (self.strict_exception("writer_files", &path)
                        || self.strict_exception("instruction_files", &path)))
                && !(daily_notes
                    && kind.is_file()
                    && (path == folder.join("vault-daily.base")
                        || (path.extension().is_some_and(|ext| ext == "md")
                            && path
                                .file_stem()
                                .and_then(|stem| stem.to_str())
                                .is_some_and(iso_date))))
                && !(folder == inbox
                    && kind.is_file()
                    && path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("eml")))
                && !(kind.is_file() && self.system_path("fixed_files", &path))
                && !(entrance == self.vault.join(TEMPLATES_INDEX)
                    && kind.is_file()
                    && self.strict_exception("template_files", &path))
            {
                self.issue(
                    &path,
                    "loose-file",
                    "outside a package, only the matching folder note belongs here",
                );
            }
        }

        if let Some((data, body)) = self.note(&entrance)? {
            self.metadata(&entrance, &data, &body, under_raw);
            if self.strict
                && text(&data, "type") == Some("project")
                && folder.parent().is_some_and(|parent| {
                    strings(&self.schema["work_project_code_roots"])
                        .any(|root| parent == self.vault.join(root))
                })
                && let Err(message) = project_codes::validate_layout(folder, &data)
            {
                self.issue(&entrance, "project-code-layout", message);
            }
            self.check_tiles(&entrance, &data, &body)?;
            if name == "raw" {
                self.check_component_tasks(folder, &children)?;
            }
            if package && under_raw {
                if self.strict {
                    for message in source_inventory::check(self.vault, folder, &data, &body)? {
                        self.issue(&entrance, "strict-source-inventory", message);
                    }
                }
                if self.strict
                    && let Err(message) = event_header::check(self.vault, folder, &data, &body)
                {
                    self.issue(&entrance, "strict-event-header", message);
                }
                if self.strict && text(&data, "type") == Some("task") {
                    let activity = if text(&data, "activity_format") == Some("chat-v1") {
                        Err("replace chat-v1 history with Event Note v1 Activity bubbles")
                    } else {
                        activity_layout::check_task_event(&body)
                    };
                    if let Err(message) = activity {
                        self.issue(&entrance, "strict-task-activity", message);
                    }
                }
                if collection_note(&data) {
                    self.issue(
                        &entrance,
                        "raw-package-wrapper",
                        "a dated Raw package must describe one source or event, not a collection index",
                    );
                }
            }
            if package
                && (under_raw
                    || (self.strict
                        && direct_inbox_child
                        && text(&data, "category") == Some("email")))
                && !package_event_date(&data, name)
            {
                self.issue(
                    &entrance,
                    "raw-package-date",
                    "event_date must match the package date, or be null for email with date_basis: ingested",
                );
            }
            if !package {
                self.layer(folder, &entrance, name, &data, &body, &children);
            }
            if self.strict && package && (under_raw || direct_inbox_child) {
                let expected = (text(&data, "category") == Some("email"))
                    .then(|| Path::new(entrance.file_name().expect("package note")));
                self.check_mail_package(folder, expected);
            }
        } else if self.strict && package && (under_raw || direct_inbox_child) {
            self.check_mail_package(folder, None);
        }
        if package && under_raw {
            for child in children {
                self.raw_package_child(&child)?;
            }
        } else if !package && recurse {
            for child in children {
                self.folder(&child)?;
            }
        }
        Ok(())
    }

    fn check_component_tasks(&mut self, raw: &Path, packages: &[PathBuf]) -> io::Result<()> {
        let Some(component) = raw.parent() else {
            return Ok(());
        };
        let Some(project) = component.parent() else {
            return Ok(());
        };
        let folder_note = |folder: &Path| -> Option<PathBuf> {
            Some(folder.join(format!("{}.md", folder.file_name()?.to_str()?)))
        };
        let Some(project_note) = folder_note(project) else {
            return Ok(());
        };
        let Some((project_data, _)) = read_metadata(&project_note)? else {
            return Ok(());
        };
        if text(&project_data, "type") != Some("project")
            || text(&project_data, "task_layout") != Some("component-raw")
        {
            return Ok(());
        }
        let Some(component_note) = folder_note(component) else {
            return Ok(());
        };
        let Some((component_data, body)) = read_metadata(&component_note)? else {
            return Ok(());
        };
        let Some(code) = text(&component_data, "code").filter(|code| {
            code.rsplit_once('-').is_some_and(|(prefix, number)| {
                number.len() == 2
                    && number.bytes().all(|byte| byte.is_ascii_digit())
                    && prefix.split('-').all(|part| {
                        !part.is_empty()
                            && part
                                .bytes()
                                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
                    })
            })
        }) else {
            self.issue(
                &component_note,
                "component-code",
                "expected the component code",
            );
            return Ok(());
        };
        let heading_prefix = format!("{code} · ");
        if !body
            .lines()
            .find_map(|line| line.strip_prefix("# "))
            .is_some_and(|heading| heading.starts_with(&heading_prefix))
        {
            self.issue(
                &component_note,
                "component-title",
                format!("start the heading with {heading_prefix}"),
            );
        }
        let mut seen = BTreeSet::new();
        for path in packages.iter().filter_map(|package| folder_note(package)) {
            let Some((data, _)) = read_metadata(&path)? else {
                continue;
            };
            if text(&data, "type") != Some("task") {
                continue;
            }
            let task_prefix = format!("{code}-T");
            let task_code = text(&data, "task-code").unwrap_or_default();
            let number = task_code
                .strip_prefix(&task_prefix)
                .and_then(|suffix| suffix.parse::<u64>().ok());
            if !number.is_some_and(|number| {
                number > 0 && task_code == format!("{task_prefix}{number:02}")
            }) {
                self.issue(
                    &path,
                    "component-task-code",
                    format!("expected a task-code like {code}-T01"),
                );
            } else {
                if !seen.insert(task_code.to_owned()) {
                    self.issue(
                        &path,
                        "duplicate-component-task-code",
                        format!("repeated task-code {task_code}"),
                    );
                }
                if !text(&data, "name")
                    .is_some_and(|name| name.starts_with(&format!("{task_code} · ")))
                {
                    self.issue(
                        &path,
                        "component-task-name",
                        format!("start the task name with {task_code} · "),
                    );
                }
            }
        }
        Ok(())
    }

    fn raw_package_child(&mut self, folder: &Path) -> io::Result<()> {
        if let Some(name) = folder.file_name().and_then(|name| name.to_str()) {
            let note = folder.join(format!("{name}.md"));
            if fs::symlink_metadata(&note).is_ok_and(|metadata| metadata.is_file()) {
                let independent = package_name(name)
                    || frontmatter(&fs::read_to_string(&note)?)
                        .and_then(|(yaml, _)| serde_saphyr::from_str::<Value>(&yaml).ok())
                        .is_some_and(|data| collection_note(&data));
                if independent {
                    self.issue(
                        folder,
                        "raw-package-wrapper",
                        "a Raw package contains a separate collection or dated package; review the sources and place each independent package with its owner",
                    );
                    return Ok(());
                }
            }
        }
        if self.strict {
            self.check_mail_package(folder, None);
        }
        let mut entries = fs::read_dir(folder)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries {
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            let kind = entry.file_type()?;
            if kind.is_symlink() && self.strict {
                self.issue(
                    &entry.path(),
                    "symlink",
                    "symbolic links are outside the layout contract",
                );
            } else if kind.is_dir() {
                self.raw_package_child(&entry.path())?;
            } else if kind.is_file() {
                self.raw_source_note(&entry.path())?;
            }
        }
        Ok(())
    }

    fn check_mail_package(&mut self, folder: &Path, expected_note: Option<&Path>) {
        let path = folder.join(mail_manifest::MANIFEST);
        let result = (|| {
            match fs::symlink_metadata(&path) {
                Ok(meta) if meta.is_file() => {}
                Err(error)
                    if error.kind() == io::ErrorKind::NotFound && expected_note.is_none() =>
                {
                    return Ok(());
                }
                Ok(_) => return Err("mail manifest must be a regular file".into()),
                Err(error) => return Err(error.into()),
            }
            let bytes = mail_manifest::read_manifest(&path)?;
            mail_manifest::verify_manifest(
                &bytes,
                Path::new(mail_manifest::MANIFEST),
                expected_note,
                |member| {
                    let target = folder.join(member);
                    for ancestor in target.ancestors().take_while(|path| *path != folder) {
                        if fs::symlink_metadata(ancestor)?.is_symlink() {
                            return Err(
                                format!("symbolic mail member: {}", member.display()).into()
                            );
                        }
                    }
                    if !fs::symlink_metadata(&target)?.is_file() {
                        return Err(format!(
                            "expected a regular mail member: {}",
                            member.display()
                        )
                        .into());
                    }
                    mail_manifest::hash_file(&target).map(Some)
                },
            )
        })();
        if let Err(error) = result {
            self.issue(&path, "mail-manifest", error.to_string());
        }
    }

    fn raw_source_note(&mut self, path: &Path) -> io::Result<()> {
        if !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            return Ok(());
        }
        let data = read_metadata(path).ok().flatten().map(|(data, _)| data);
        let source = data.as_ref().is_some_and(|data| {
            text(data, "category") == Some("source") || data.get("wiki_status").is_some()
        });
        if source && let Some((data, body)) = self.note(path)? {
            self.metadata(path, &data, &body, true);
        } else if self.strict && !source && !self.strict_exception("supporting_markdown", path) {
            self.issue(
                path,
                "strict-raw-markdown",
                "secondary Markdown needs source metadata or an exact supporting-file exception",
            );
        }
        Ok(())
    }

    fn note(&mut self, path: &Path) -> io::Result<Option<(Value, String)>> {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() => {}
            Ok(_) => {
                self.issue(
                    path,
                    "folder-note",
                    "entrance must be a regular Markdown file",
                );
                return Ok(None);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.issue(path, "folder-note", "missing matching folder note");
                return Ok(None);
            }
            Err(error) => return Err(error),
        }
        let text = fs::read_to_string(path)?;
        let Some((yaml, body)) = frontmatter(&text) else {
            self.issue(path, "frontmatter", "expected a YAML frontmatter block");
            return Ok(None);
        };
        if self.strict {
            for message in mail_draft::findings(&body) {
                self.issue(path, "strict-mail-draft", message);
            }
            for message in activity_delivery::findings(&body) {
                self.issue(path, "strict-activity-delivery", message);
            }
        }
        for line in table_wikilink_lines(&body) {
            let line = line + yaml.split('\n').count() + 2;
            self.issue(
                path,
                "table-wikilink-pipe",
                format!(
                    r"line {line}: escape the alias separator inside the table: [[note\|label]]"
                ),
            );
        }
        let options = serde_saphyr::options! {
            duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
        };
        match serde_saphyr::from_str_with_options::<Value>(&yaml, options) {
            Ok(data) if data.is_object() => {
                if self.strict
                    && let Some(contract) = data.get("activity_contract")
                {
                    if contract.as_str() != Some("provenance-v1") {
                        self.issue(
                            path,
                            "strict-activity-provenance",
                            "unknown activity_contract",
                        );
                    } else {
                        for message in activity_delivery::provenance_findings(&body) {
                            self.issue(path, "strict-activity-provenance", message);
                        }
                    }
                }
                if crate::text(&data, "activity_format") == Some("chat-v1") {
                    for message in activity_layout::findings(&body) {
                        self.issue(path, "activity-message", message);
                    }
                }
                Ok(Some((data, body)))
            }
            _ => {
                self.issue(
                    path,
                    "frontmatter",
                    "frontmatter must be a valid YAML mapping with unique keys",
                );
                Ok(None)
            }
        }
    }

    fn metadata(&mut self, path: &Path, data: &Value, body: &str, under_raw: bool) {
        let kind = text(data, "type").unwrap_or("");
        let profiles = &self.schema["validation_profiles"];
        let contract = &self.schema["contracts"][kind];
        let profile = profiles.get(kind).unwrap_or(&profiles["minimal"]);
        let mut required = strings(&profile["required"]).collect::<BTreeSet<_>>();
        required.extend(strings(&contract["required_fields"]));
        if under_raw {
            required.extend(strings(
                &self.schema["contracts"]["raw_package"]["required_fields"],
            ));
        }
        for field in required {
            if text(data, field).is_none() {
                self.issue(path, "metadata", format!("missing or empty {field}"));
            }
        }
        for (field, allowed) in [("status", "status_values"), ("category", "category_values")] {
            if let Some(value) = text(data, field)
                && contract[allowed].is_array()
                && !strings(&contract[allowed]).any(|item| item == value)
            {
                self.issue(
                    path,
                    "metadata",
                    format!("{field} is outside the {kind} contract"),
                );
            }
        }
        for field in ["created", "updated", "wiki_processed_on"] {
            if let Some(value) = data.get(field)
                && !value.as_str().is_some_and(iso_date)
            {
                self.issue(
                    path,
                    "metadata",
                    format!("{field} must be a valid YYYY-MM-DD date"),
                );
            }
        }
        if self.strict {
            for field in ["due", "work_date", "review_after", "reviewed_on"] {
                if let Some(value) = data.get(field)
                    && !value.is_null()
                    && !value.as_str().is_some_and(iso_date)
                {
                    self.issue(
                        path,
                        "metadata",
                        format!("{field} must be a valid YYYY-MM-DD date"),
                    );
                }
            }
        }

        if let Some(status) = data.get("wiki_status") {
            let statuses = &self.schema["contracts"]["raw_package"]["wiki_status_values"];
            let status = status.as_str().unwrap_or("");
            if statuses.get(status).is_none() {
                self.issue(path, "wiki-status", "expected pending or processed");
            }
            if status == "processed" {
                if under_raw
                    && let (Some(updated), Some(processed)) =
                        (text(data, "updated"), text(data, "wiki_processed_on"))
                    && iso_date(updated)
                    && iso_date(processed)
                    && updated > processed
                {
                    self.issue(
                        path,
                        "wiki-status-stale",
                        "updated is newer than wiki_processed_on; set wiki_status to pending",
                    );
                }
                if text(data, "wiki_processed_on").is_none() {
                    self.issue(path, "wiki-status", "processed requires wiki_processed_on");
                }
                let wiki = if under_raw {
                    path.ancestors()
                        .take_while(|ancestor| *ancestor != self.vault)
                        .find(|ancestor| ancestor.file_name().is_some_and(|name| name == "raw"))
                        .and_then(Path::parent)
                } else {
                    path.parent().and_then(Path::parent).and_then(Path::parent)
                };
                let target = text(data, "wiki_target");
                if !target.is_some_and(|target| self.wiki_target(target, wiki)) {
                    self.issue(
                        path,
                        "wiki-status",
                        "processed requires a link to an existing note in the dossier Wiki",
                    );
                } else if under_raw {
                    let result = match target.and_then(raw_processing::target) {
                        Some((note, anchor)) => {
                            let mut destination = self.vault.join(note);
                            if !note.ends_with(".md") {
                                destination.as_mut_os_string().push(".md");
                            }
                            match fs::read_to_string(destination) {
                                Ok(wiki)
                                    if self.strict
                                        || strings(&data["cssclasses"])
                                            .any(|class| class == "vault-event-v1-1") =>
                                {
                                    raw_processing::check_event(body, note, anchor, &wiki)
                                }
                                Ok(wiki) => raw_processing::check(body, note, anchor, &wiki),
                                Err(_) => {
                                    Err("cannot read the Wiki note referenced by wiki_target")
                                }
                            }
                        }
                        None => Err("wiki_target must include a Wiki heading"),
                    };
                    if let Err(message) = result {
                        self.issue(path, "wiki-processing", message);
                    }
                }
            }
        }
    }

    fn wiki_target(&self, link: &str, dossier: Option<&Path>) -> bool {
        let Some(target) = link
            .strip_prefix("[[")
            .and_then(|link| link.strip_suffix("]]"))
        else {
            return false;
        };
        let target = target.split(['#', '|']).next().unwrap_or("");
        let path = Path::new(target);
        if path
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return false;
        }
        let mut destination = self.vault.join(path);
        if destination.extension().and_then(|ext| ext.to_str()) != Some("md") {
            destination.as_mut_os_string().push(".md");
        }
        dossier.is_some_and(|root| {
            let wiki = root.join("wiki");
            let direct = direct_wiki_note(root);
            destination.canonicalize().is_ok_and(|resolved| {
                resolved.is_file()
                    && (resolved.starts_with(wiki)
                        || direct
                            .as_ref()
                            .is_some_and(|note| resolved == *note && is_direct_wiki_note(note)))
            })
        })
    }

    fn layer(
        &mut self,
        folder: &Path,
        path: &Path,
        name: &str,
        data: &Value,
        body: &str,
        children: &[PathBuf],
    ) {
        let allowed = &self.schema["common_fields"]["layer"]["allowed_values"];
        let layer = text(data, "layer").unwrap_or("");
        if !strings(allowed).any(|value| value == layer) {
            self.issue(path, "layer", "expected layer: navigation or layer: wiki");
        }
        let daily_navigation =
            folder == self.vault.join("10-daily") || folder == self.vault.join("10-daily/notes");
        if self.system_path("collections", folder) || (self.strict && daily_navigation) {
            let relative = folder
                .strip_prefix(self.vault)
                .expect("system folder inside vault");
            let parent = relative
                .parent()
                .filter(|path| !path.as_os_str().is_empty());
            let back = parent
                .and_then(|path| path.file_name().map(|name| path.join(name)))
                .unwrap_or_else(|| PathBuf::from("HOME"));
            let expected_back = format!("> - [[{}|Back]]", back.display());
            let expected_finder = format!("> - [Finder](file://{})", folder.display());
            let mut lines = body.lines().filter(|line| !line.trim().is_empty());
            if layer != "navigation"
                || !lines.next().is_some_and(|line| line.starts_with("# "))
                || lines.next() != Some("> [!navigation|shortcuts]")
                || lines.next() != Some(&expected_back)
                || lines.next() != Some(&expected_finder)
            {
                self.issue(
                    path,
                    if daily_navigation {
                        "daily-navigation"
                    } else {
                        "system-navigation"
                    },
                    "use the title, Back to the parent and Finder to this collection",
                );
            }
        }

        let in_wiki = folder
            .strip_prefix(self.vault)
            .is_ok_and(|relative| relative.components().any(|part| part.as_os_str() == "wiki"));
        let has_dossier_children = children.iter().any(|child| {
            matches!(
                child.file_name().and_then(|name| name.to_str()),
                Some("raw" | "wiki")
            )
        });
        let direct = strings(&data["cssclasses"]).any(|class| class == "vault-wiki-direct");
        let dashboard =
            strings(&data["cssclasses"]).any(|class| class == "vault-dossier-dashboard");
        let dossier = !in_wiki
            && (has_dossier_children || (layer == "wiki" && (self.strict || !children.is_empty())));
        if self.strict {
            if dossier && (!dashboard || direct) {
                self.issue(
                    path,
                    "strict-dossier-dashboard",
                    "use the current Wiki/Raw dossier dashboard",
                );
            }
            if layer == "navigation"
                && name != "raw"
                && path != self.vault.join("00-inbox/00-inbox.md")
                && path != self.vault.join(TEMPLATES_INDEX)
                && path != self.vault.join("10-daily/notes/notes.md")
            {
                if !dossier_entrance_body(body)
                    || body.contains("![[vault-navigation.base#Dossier recent]]")
                {
                    self.issue(
                        path,
                        "strict-navigation-content",
                        "keep navigation pages to their title, shortcuts and child tiles",
                    );
                } else if !self.system_path("collections", folder) && !daily_navigation {
                    let parent = folder.parent().expect("collection inside vault");
                    let back = if parent == self.vault {
                        self.vault.join("HOME.md")
                    } else {
                        direct_wiki_note(parent).expect("parent folder has a name")
                    };
                    if !collection_shortcuts(body, folder, |line| {
                        self.shortcut_points_to(path, line, &back, &["Back", "Terug"])
                    }) {
                        let back = back.strip_prefix(self.vault).expect("Back inside vault");
                        self.issue(
                            path,
                            "strict-navigation-shortcuts",
                            format!(
                                "use Back to [[{}]] and Finder to this collection",
                                back.with_extension("").display()
                            ),
                        );
                    }
                }
            }
        }
        if direct
            && (in_wiki || layer != "wiki" || children.len() != 1 || !children[0].ends_with("raw"))
        {
            self.issue(
                path,
                "wiki-direct",
                "a direct Wiki has layer: wiki and only one child folder, raw/",
            );
        }
        let delegated_children = children
            .iter()
            .filter(|child| self.system_path("delegated", child))
            .count();
        if dashboard && (!dossier || direct || !dossier_dashboard_body(body, delegated_children)) {
            self.issue(
                path,
                "dossier-dashboard",
                "use the Back/Finder row, Wiki/Raw tiles, and a vault-task-summary block above one Open tasks Base embed",
            );
        }
        if dashboard {
            self.check_raw_summary_plugin();
        }
        if dashboard || name == "raw" {
            self.check_dossier_tables(dashboard);
        }
        if dossier && !direct {
            if has_dossier_children && !dashboard && !dossier_entrance_body(body) {
                self.issue(
                    path,
                    "dossier-entrance-content",
                    "keep only the title, navigation and Dossier recent Base; put knowledge in Wiki and history in Raw",
                );
            }
            if layer != "wiki" {
                self.issue(
                    path,
                    "layer",
                    "a dossier entrance with Raw and Wiki uses layer: wiki",
                );
            }
            if children.len() != 2 + delegated_children
                || !children.iter().any(|p| p.ends_with("raw"))
                || !children.iter().any(|p| p.ends_with("wiki"))
            {
                self.issue(
                    folder,
                    "dossier-folders",
                    "a dossier entrance has raw/ and wiki/ plus any registered delegated system folders",
                );
            }
        } else if name == "raw" && layer != "navigation" {
            self.issue(path, "layer", "Raw entrance uses layer: navigation");
        } else if in_wiki && layer != "wiki" {
            self.issue(path, "layer", "Wiki entrances use layer: wiki");
        }

        let direct_parent = name == "raw"
            && folder
                .parent()
                .and_then(direct_wiki_note)
                .is_some_and(|note| is_direct_wiki_note(&note));
        let dashboard_child = matches!(name, "raw" | "wiki")
            && folder
                .parent()
                .and_then(direct_wiki_note)
                .and_then(|note| read_metadata(&note).ok().flatten())
                .is_some_and(|(parent, _)| {
                    strings(&parent["cssclasses"]).any(|class| class == "vault-dossier-dashboard")
                });
        if dashboard || dashboard_child {
            self.check_dossier_profile(folder, path, name, data, body, dashboard);
        }
        if name == "raw" {
            if !raw_hub_body(body, direct_parent, dashboard_child) {
                self.issue(
                    path,
                    "raw-hub-content",
                    "use the title, Back/Wiki/Finder shortcuts (Back/Finder for a direct Wiki), and the Latest sources Base; dossier dashboards require a vault-raw-summary block directly above the Base",
                );
            } else if self.strict && !dashboard_child && !direct_parent {
                self.check_dossier_shortcuts(folder, path, name, body);
            }
        }
        if name == "raw"
            && (dashboard_child || body.lines().any(|line| line.trim_end() == RAW_SUMMARY))
        {
            self.check_raw_summary_plugin();
        }

        if direct || (name == "wiki" && in_wiki) {
            if layer == "wiki" && !strings(&data["cssclasses"]).any(|class| class == "vault-wiki") {
                self.issue(
                    path,
                    "wiki-style",
                    "add vault-wiki for the shared Wiki reading and link style",
                );
            }
            let (has_fact_cards, wiki_findings) = wiki_layout::findings(body);
            if self.strict
                && !dashboard_child
                && !direct
                && !wiki_findings
                    .iter()
                    .any(|(code, _)| *code == "wiki-preamble")
            {
                self.check_dossier_shortcuts(folder, path, name, body);
            }
            for (code, message) in wiki_findings {
                self.issue(path, code, message);
            }
            if (self.strict || dashboard_child)
                && has_fact_cards
                && !strings(&data["cssclasses"]).any(|class| class == "vault-wiki-overview")
            {
                self.issue(
                    path,
                    "dossier-dashboard-style",
                    "add vault-wiki-overview for compact Wiki fact cards",
                );
            }
        }

        if name != "raw" && !direct && (dossier || layer == "navigation") {
            self.navigation(path, body, children);
        }
    }

    fn shortcut_points_to(
        &self,
        source: &Path,
        line: &str,
        expected: &Path,
        labels: &[&str],
    ) -> bool {
        line.strip_prefix("> - [[")
            .and_then(|link| link.strip_suffix("]]"))
            .and_then(|link| link.split_once('|'))
            .is_some_and(|(target, label)| {
                labels.contains(&label)
                    && shortcut_links::resolve_note(self.vault, &self.notes, source, target)
                        .is_ok_and(|result| result.is_ok_and(|note| note == expected))
            })
    }

    fn check_dossier_shortcuts(&mut self, folder: &Path, path: &Path, name: &str, body: &str) {
        let parent = folder.parent().expect("Wiki is inside a dossier");
        let back = direct_wiki_note(parent).expect("dossier has a name");
        let sibling = if name == "wiki" {
            "raw/raw.md"
        } else {
            "wiki/wiki.md"
        };
        let sibling = parent.join(sibling);
        let label = if name == "wiki" { "Raw" } else { "Wiki" };
        let mut lines = body
            .lines()
            .map(str::trim_end)
            .skip_while(|line| *line != "> [!navigation|shortcuts]")
            .skip(1);
        if !lines
            .next()
            .is_some_and(|line| self.shortcut_points_to(path, line, &back, &["Back", "Terug"]))
            || !lines
                .next()
                .is_some_and(|line| self.shortcut_points_to(path, line, &sibling, &[label]))
            || !lines
                .next()
                .is_some_and(|line| finder_targets(line, folder))
        {
            self.issue(
                path,
                "strict-dossier-shortcuts",
                "point Back to the dossier, Wiki/Raw to its sibling, and Finder to this folder",
            );
        }
    }

    fn check_dossier_profile(
        &mut self,
        folder: &Path,
        path: &Path,
        name: &str,
        data: &Value,
        body: &str,
        dashboard: bool,
    ) {
        let Some(parent) = folder.parent().filter(|parent| *parent != self.vault) else {
            self.issue(
                path,
                "dossier-dashboard-back",
                "a dossier needs a parent entrance for Back",
            );
            return;
        };
        let vault = self.vault;
        let relative = |target: &Path| {
            target
                .strip_prefix(vault)
                .expect("dossier inside vault")
                .with_extension("")
                .to_string_lossy()
                .replace('\\', "/")
        };
        let mut shortcuts = body
            .lines()
            .map(str::trim_end)
            .skip_while(|line| *line != "> [!navigation|shortcuts]")
            .skip(1);
        let back = direct_wiki_note(parent).expect("parent folder has a name");
        if shortcuts.next() != Some(format!("> - [[{}|Back]]", relative(&back)).as_str()) {
            self.issue(
                path,
                "dossier-dashboard-back",
                format!("point Back to [[{}]]", relative(&back)),
            );
        }
        if !dashboard {
            let sibling = if name == "wiki" { "raw" } else { "wiki" };
            let target = parent.join(sibling).join(format!("{sibling}.md"));
            let label = if name == "wiki" { "Raw" } else { "Wiki" };
            if shortcuts.next() != Some(format!("> - [[{}|{label}]]", relative(&target)).as_str()) {
                self.issue(
                    path,
                    "dossier-dashboard-shortcut",
                    format!("point {label} to [[{}]]", relative(&target)),
                );
            }
        }
        if self.strict
            && !shortcuts
                .next()
                .is_some_and(|line| finder_targets(line, folder))
        {
            self.issue(
                path,
                "dossier-dashboard-finder",
                "point Finder to the folder containing this note",
            );
        }

        let has_class = |class| strings(&data["cssclasses"]).any(|value| value == class);
        let page_classes: &[&str] = match (dashboard, name) {
            (true, _) => &["folder-note", "dashboard-option-2", "dashboard-variant-2-2"],
            (false, "wiki") => &["vault-wiki"],
            (false, _) => &[
                "folder-note",
                "dashboard-option-2",
                "dashboard-variant-2-2",
                "raw-hub-table",
            ],
        };
        for class in ["vault-navigation", "vault-dashboard"]
            .into_iter()
            .chain(page_classes.iter().copied())
        {
            if !has_class(class) {
                self.issue(
                    path,
                    "dossier-dashboard-style",
                    format!("add the {class} CSS class"),
                );
            }
        }
    }

    fn navigation(&mut self, path: &Path, body: &str, children: &[PathBuf]) {
        if path == self.vault.join("10-daily/notes/notes.md") {
            let base_valid = fs::read_to_string(self.vault.join("10-daily/notes/vault-daily.base"))
                .ok()
                .and_then(|source| serde_saphyr::from_str::<Value>(&source).ok())
                .is_some_and(|base| {
                    let views = base["views"].as_array().map(Vec::as_slice);
                    matches!(views, Some([view])
                        if text(view, "name") == Some("Dagnotities")
                        && text(view, "type") == Some("table")
                        && strings(&view["order"]).eq(["formula.day"])
                        && matches!(view["sort"].as_array().map(Vec::as_slice), Some([sort])
                            if text(sort, "property") == Some("file.name")
                            && text(sort, "direction") == Some("DESC")))
                        && text(&base["formulas"], "day") == Some("link(file.path, file.name)")
                        && strings(&base["filters"]["and"]).eq([
                            r#"file.folder == "10-daily/notes""#,
                            r#"file.ext == "md""#,
                            r#"file.name != "notes""#,
                            r#"!file.name.contains(".sync-conflict-")"#,
                        ])
                });
            let expected = format!(
                "# Dagnotities\n\n> [!navigation|shortcuts]\n> - [[10-daily/10-daily|Back]]\n> - [Finder](file://{}/10-daily/notes)\n\n![[vault-daily.base#Dagnotities]]",
                self.vault.display()
            );
            if !children.is_empty() || !base_valid || body.trim() != expected {
                self.issue(
                    path,
                    "daily-navigation",
                    "use the Dagnotities Base for direct date notes",
                );
            }
            return;
        }
        if path == self.vault.join(TEMPLATES_INDEX) {
            if let Some(templates) =
                self.schema["strict_layout_exceptions"]["template_files"].as_object()
            {
                for (template, reason) in templates {
                    let file = self.vault.join(template);
                    if file.parent() != path.parent()
                        || file.extension().is_none_or(|extension| extension != "md")
                        || reason
                            .as_str()
                            .is_none_or(|reason| reason.trim().is_empty())
                        || !fs::symlink_metadata(&file).is_ok_and(|entry| entry.is_file())
                    {
                        self.issue(
                            &file,
                            "templates-file",
                            "the registered template must be a direct Markdown file with a reason",
                        );
                    }
                }
            } else {
                self.issue(
                    path,
                    "templates-file",
                    "register the template paths in vault-schema.yaml",
                );
            }
            let expected = format!(
                "# Templates\n\n> [!navigation|shortcuts]\n> - [[HOME|Back]]\n> - [Finder](file://{}/90-templates)\n> - [[99-system/vault/vault-navigation/vault-navigation-handleiding|Info]]\n\n```vault-template-summary\n```\n\n![[vault-templates.base#Templates]]",
                self.vault.display()
            );
            if body.trim() != expected || !children.is_empty() {
                self.issue(
                    path,
                    "templates-navigation",
                    "use the Templates shortcuts, counters and Base without child folders",
                );
            }
            let base_valid = fs::read_to_string(self.vault.join(TEMPLATES_BASE))
                .ok()
                .and_then(|source| serde_saphyr::from_str::<Value>(&source).ok())
                .is_some_and(|base| {
                    let views = base["views"].as_array().map(Vec::as_slice);
                    matches!(views, Some([view])
                        if text(view, "name") == Some("Templates")
                        && text(view, "type") == Some("table")
                        && strings(&view["order"]).eq(["formula.template_type", "formula.modified", "formula.template"]))
                        && strings(&base["filters"]["and"]).eq(TEMPLATE_FILTERS.iter().copied())
                });
            if !base_valid {
                self.issue(path, "templates-base", "use the shared Templates Base with the registered folder filters and Type, Gewijzigd, Template columns");
            }
            self.check_raw_summary_plugin();
            return;
        }
        if path == self.vault.join("00-inbox/00-inbox.md") {
            let inbox_view = "Inbox packages";
            let base_valid = fs::read_to_string(
                self.vault
                    .join("99-system/vault/vault-navigation/standaard/wiki/vault-inbox.base"),
            )
            .ok()
            .and_then(|source| serde_saphyr::from_str::<Value>(&source).ok())
            .is_some_and(|base| {
                index_navigation::base_count(&base, inbox_view) == 1
                    && base["views"].as_array().into_iter().flatten().any(|view| {
                        text(view, "name") == Some(inbox_view)
                            && text(view, "type") == Some("table")
                            && strings(&view["order"]).eq([
                                "formula.status",
                                "formula.added_day",
                                "formula.package",
                            ])
                    })
                    && text(&base["formulas"], "package") == Some("link(file.path, file.name)")
                    && strings(&base["filters"]["and"]).eq([
                        r#"file.ext == "md""#,
                        r#"file.folder == "00-inbox/" + file.name"#,
                        r#"!file.name.contains(".sync-conflict-")"#,
                    ])
            });
            let expected = format!(
                "# Inbox\n\n> [!navigation|shortcuts]\n> - [[HOME|Back]]\n> - [Finder](file://{}/00-inbox)\n> - [[vault-inbox-proces|Info]]\n\n```vault-inbox-summary\n```\n\n![[vault-inbox.base#Inbox packages]]",
                self.vault.display()
            );
            if body
                .lines()
                .filter(|line| line.trim() == "![[vault-inbox.base#Inbox packages]]")
                .count()
                != 1
                || tiles(body, false).next().is_some()
                || !base_valid
                || (self.strict && body.trim() != expected)
            {
                self.issue(path, "inbox-navigation", "use the Inbox packages Base view");
            }
            if self.strict {
                self.check_raw_summary_plugin();
            }
            return;
        }

        let expected: BTreeSet<_> = children
            .iter()
            .map(|child| {
                let mut target = child
                    .strip_prefix(self.vault)
                    .expect("child inside vault")
                    .to_path_buf();
                target.push(child.file_name().expect("child folder name"));
                target
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .collect();
        let mut seen = BTreeSet::new();
        for tile in navigation_links(body, false) {
            let Ok(target) = tile else {
                self.issue(
                    path,
                    "tile-target",
                    "content tiles must be vault-relative wikilinks",
                );
                continue;
            };
            let (target, label) = target.split_once('|').unwrap_or((target, ""));
            let target = target.strip_suffix(".md").unwrap_or(target);
            if self.vault_collection(path.parent().expect("navigation note has a parent"))
                && !vault_tile_label_matches(target, label)
            {
                self.issue(
                    path,
                    "vault-tile-label",
                    format!("use the physical child folder name as the label for {target}"),
                );
            }
            if !expected.contains(target) {
                self.issue(
                    path,
                    "tile-target",
                    format!("{target} is not an exact direct child entrance"),
                );
            }
            if !seen.insert(target) {
                self.issue(
                    path,
                    "duplicate-tile",
                    format!("duplicate tile for {target}"),
                );
            }
        }
        for target in expected {
            if !seen.contains(target.as_str()) {
                self.issue(path, "missing-tile", format!("missing tile for {target}"));
            }
        }
    }
}

fn dossier_entrance_body(body: &str) -> bool {
    let mut lines = body.lines().map(str::trim_end);
    if lines
        .find(|line| !line.is_empty())
        .and_then(|line| line.strip_prefix("# "))
        .is_none_or(|title| title.is_empty())
    {
        return false;
    }

    let mut navigation = false;
    let mut recent_seen = false;
    for line in lines {
        match line {
            "" => navigation = false,
            "> [!navigation]" | "> [!navigation|back]" | "> [!navigation|shortcuts]" => {
                navigation = true;
            }
            "![[vault-navigation.base#Dossier recent]]" if !recent_seen => {
                navigation = false;
                recent_seen = true;
            }
            _ if navigation
                && (line
                    .strip_prefix("> - [[")
                    .and_then(|link| link.strip_suffix("]]"))
                    .is_some_and(|link| !link.is_empty() && !link.contains(['[', ']']))
                    || line
                        .strip_prefix("> - [Finder](file:///")
                        .and_then(|link| link.strip_suffix(')'))
                        .is_some_and(|link| !link.is_empty() && !link.contains(['(', ')']))) => {}
            _ => return false,
        }
    }
    true
}

fn dossier_dashboard_body(body: &str, delegated_children: usize) -> bool {
    let lines: Vec<_> = body.lines().map(str::trim_end).collect();
    let mut blocks = lines
        .split(|line| line.is_empty())
        .filter(|block| !block.is_empty());
    let Some([title]) = blocks.next() else {
        return false;
    };
    let Some(["> [!navigation|shortcuts]", back, finder]) = blocks.next() else {
        return false;
    };
    let Some(["> [!navigation]", wiki, raw, delegated @ ..]) = blocks.next() else {
        return false;
    };
    title
        .strip_prefix("# ")
        .is_some_and(|title| !title.trim().is_empty())
        && wiki_layout::shortcut(back, "Back")
        && finder
            .strip_prefix("> - [Finder](file:///")
            .and_then(|target| target.strip_suffix(')'))
            .is_some_and(|target| !target.trim().is_empty())
        && wiki_layout::shortcut(wiki, "Wiki")
        && wiki_layout::shortcut(raw, "Raw")
        && delegated.len() == delegated_children
        && blocks.next() == Some(&[TASK_SUMMARY, "```"][..])
        && blocks.next() == Some(&[OPEN_TASKS][..])
        && blocks.next().is_none()
}

fn direct_wiki_note(folder: &Path) -> Option<PathBuf> {
    let mut name = folder.file_name()?.to_os_string();
    name.push(".md");
    Some(folder.join(name))
}

fn is_direct_wiki_note(note: &Path) -> bool {
    read_metadata(note).ok().flatten().is_some_and(|(data, _)| {
        text(&data, "layer") == Some("wiki")
            && strings(&data["cssclasses"]).any(|class| class == "vault-wiki-direct")
    })
}

fn raw_hub_body(body: &str, direct_wiki: bool, summary_required: bool) -> bool {
    let body_lines: Vec<_> = body.lines().map(str::trim_end).collect();
    let mut lines = body_lines
        .iter()
        .copied()
        .filter(|line| !line.is_empty())
        .peekable();
    let shortcut = |line: &str, label: &str| {
        line.strip_prefix("> - [[")
            .and_then(|link| link.split_once('|'))
            .is_some_and(|(target, alias)| {
                !target.trim().is_empty()
                    && !target.contains(['[', ']'])
                    && alias.strip_suffix("]]") == Some(label)
            })
    };
    lines.next().is_some_and(|line| {
        line.strip_prefix("# ")
            .is_some_and(|title| !title.trim().is_empty() && !title.contains(['[', ']', '<', '>']))
    }) && lines.next() == Some("> [!navigation|shortcuts]")
        && lines
            .next()
            .is_some_and(|line| shortcut(line, "Back") || shortcut(line, "Terug"))
        && (direct_wiki || lines.next().is_some_and(|line| shortcut(line, "Wiki")))
        && lines.next().is_some_and(|line| {
            line.strip_prefix("> - [Finder](file:///")
                .and_then(|link| link.strip_suffix(')'))
                .is_some_and(|target| {
                    !target.is_empty() && !target.contains(['(', ')', '[', ']', '<', '>'])
                })
        })
        && if lines.next_if_eq(&RAW_SUMMARY).is_some() {
            lines.next() == Some("```")
                && body_lines
                    .windows(4)
                    .any(|block| block == ["", RAW_SUMMARY, "```", ""])
        } else {
            !summary_required
        }
        && lines.next() == Some("![[vault-raw.base#Latest sources]]")
        && lines.next().is_none()
}

fn collection_note(data: &Value) -> bool {
    text(data, "type") == Some("collection") || text(data, "category") == Some("index")
}

fn text<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
    data.get(key)?
        .as_str()
        .filter(|text| !text.trim().is_empty())
}

fn strings(value: &Value) -> impl Iterator<Item = &str> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

fn read_metadata(path: &Path) -> io::Result<Option<(Value, String)>> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() => {}
        Ok(_) => return Ok(None),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    }
    Ok(frontmatter(&fs::read_to_string(path)?)
        .and_then(|(yaml, body)| serde_saphyr::from_str(&yaml).ok().map(|data| (data, body))))
}

fn frontmatter(note: &str) -> Option<(String, String)> {
    let mut lines = note.trim_start_matches('\u{feff}').lines();
    if lines.next()? != "---" {
        return None;
    }
    let mut yaml = Vec::new();
    loop {
        let line = lines.next()?;
        if line == "---" {
            return Some((yaml.join("\n"), lines.collect::<Vec<_>>().join("\n")));
        }
        yaml.push(line);
    }
}

fn date_prefix(name: &str) -> bool {
    name.as_bytes()
        .get(..8)
        .is_some_and(|prefix| prefix.iter().all(u8::is_ascii_digit))
}

fn finder_targets(line: &str, folder: &Path) -> bool {
    line.strip_prefix("> - [Finder](file://")
        .and_then(|target| target.strip_suffix(')'))
        .is_some_and(|target| Path::new(&shortcut_links::decode_percent(target)) == folder)
}

fn collection_shortcuts(body: &str, folder: &Path, back: impl FnOnce(&str) -> bool) -> bool {
    let mut lines = body
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty());
    lines.next().is_some()
        && matches!(
            lines.next(),
            Some("> [!navigation|shortcuts]" | "> [!navigation|back]")
        )
        && lines.next().is_some_and(back)
        && lines
            .next()
            .is_some_and(|line| finder_targets(line, folder))
}

fn package_event_date(data: &Value, name: &str) -> bool {
    let date = data.get("event_date");
    let matches_package = |date: &str| {
        iso_date(date)
            && date
                .bytes()
                .filter(|&byte| byte != b'-')
                .eq(name.bytes().take(8))
    };
    if text(data, "category") == Some("email") {
        match text(data, "date_basis") {
            Some("ingested") => {
                return date.is_some_and(Value::is_null)
                    && text(data, "ingested_at")
                        .and_then(|timestamp| timestamp.split_once('T'))
                        .is_some_and(|(date, _)| matches_package(date));
            }
            Some("message-date") if date.is_none() => return false,
            _ => {}
        }
    }
    date.is_none_or(|value| value.as_str().is_some_and(matches_package))
}

fn package_name(name: &str) -> bool {
    let Some((date, slug)) = name.split_once('-') else {
        return false;
    };
    date.len() == 8
        && date.bytes().all(|byte| byte.is_ascii_digit())
        && valid_date(&date[..4], &date[4..6], &date[6..])
        && slug.split('-').all(|word| {
            !word.is_empty()
                && word
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

fn iso_date(date: &str) -> bool {
    let parts: Vec<_> = date.split('-').collect();
    matches!(parts.as_slice(), [year, month, day] if year.len() == 4 && month.len() == 2 && day.len() == 2 && valid_date(year, month, day))
}

fn valid_date(year: &str, month: &str, day: &str) -> bool {
    if [year, month, day]
        .iter()
        .any(|part| !part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        year.parse::<u32>(),
        month.parse::<u32>(),
        day.parse::<u32>(),
    ) else {
        return false;
    };
    let max = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        _ => 0,
    };
    year > 0 && (1..=max).contains(&day)
}

fn table_wikilink_lines(body: &str) -> impl Iterator<Item = usize> + '_ {
    let mut lines = body.lines().enumerate().peekable();
    let mut in_table = false;
    let mut fence = None;
    std::iter::from_fn(move || {
        while let Some((index, line)) = lines.next() {
            let line = line.trim();
            if let Some(marker) = fence {
                if line.starts_with(marker) {
                    fence = None;
                }
                continue;
            }
            if line.starts_with("```") || line.starts_with("~~~") {
                fence = Some(&line[..3]);
                in_table = false;
                continue;
            }
            if !line.contains('|') {
                in_table = false;
                continue;
            }
            if lines.peek().is_some_and(|(_, next)| table_separator(next)) {
                lines.next();
                in_table = true;
            }
            if in_table
                && line.split("[[").skip(1).any(|tail| {
                    tail.split_once("]]").is_some_and(|(link, _)| {
                        link.split_once('|').is_some_and(|(target, _)| {
                            target
                                .bytes()
                                .rev()
                                .take_while(|&byte| byte == b'\\')
                                .count()
                                % 2
                                == 0
                        })
                    })
                })
            {
                return Some(index + 1);
            }
        }
        None
    })
}

fn table_separator(line: &str) -> bool {
    let line = line.trim();
    line.contains('|')
        && line.trim_matches('|').split('|').all(|cell| {
            let dashes = cell.trim().trim_matches(':');
            !dashes.is_empty() && dashes.bytes().all(|byte| byte == b'-')
        })
}

fn vault_tile_label_matches(target: &str, label: &str) -> bool {
    let Some((folder, _)) = target.rsplit_once('/') else {
        return false;
    };
    let folder_name = folder.rsplit('/').next().unwrap_or(folder);
    label
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        == folder_name
}

fn tiles(body: &str, shortcuts: bool) -> impl Iterator<Item = Result<&str, ()>> {
    navigation_links(body, shortcuts).map(move |link| {
        link.map(|link| {
            if shortcuts {
                link
            } else {
                link.split('|').next().unwrap_or(link)
            }
        })
    })
}

fn navigation_links(body: &str, shortcuts: bool) -> impl Iterator<Item = Result<&str, ()>> {
    let mut active = false;
    let mut fence = None;
    let mut comment = false;
    body.lines().filter_map(move |line| {
        let trimmed = line.trim_start();
        if let Some(marker) = fence {
            if trimmed.starts_with(marker) {
                fence = None;
            }
            return None;
        }
        if comment || trimmed.starts_with("<!--") {
            comment = !trimmed.contains("-->");
            active = false;
            return None;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fence = Some(&trimmed[..3]);
            active = false;
            return None;
        }
        if trimmed.starts_with("> [!") {
            let heading = trimmed.trim_end();
            active = if shortcuts {
                matches!(
                    heading,
                    "> [!navigation|shortcuts]" | "> [!navigation|back]"
                )
            } else {
                heading == "> [!navigation]"
            };
            return None;
        }
        if !trimmed.starts_with('>') {
            active = false;
            return None;
        }
        if !active {
            return None;
        }
        let item = trimmed.strip_prefix('>')?.trim_start();
        let item = item
            .strip_prefix("- ")
            .or_else(|| item.strip_prefix("* "))
            .or_else(|| item.strip_prefix("+ "))?;
        let link = item
            .trim()
            .strip_prefix("[[")
            .and_then(|link| link.strip_suffix("]]"));
        if shortcuts {
            let link = link?;
            let (target, label) = link.split_once('|').unwrap_or((link, ""));
            let note = target.split('#').next().unwrap_or(target);
            let note = note.strip_suffix(".md").unwrap_or(note);
            let destination = note.rsplit('/').next().unwrap_or(note);
            if !matches!(label, "Back" | "Terug")
                && (matches!(label, "Wiki" | "Raw") || matches!(destination, "wiki" | "raw"))
            {
                return Some(Ok(target));
            }
            return None;
        }
        Some(link.ok_or(()))
    })
}

struct TileStyle<'a> {
    class: &'a str,
    operator: &'a str,
    value: &'a str,
    accent: &'a str,
    icon: &'a str,
}

impl TileStyle<'_> {
    fn matches(&self, target: &str) -> bool {
        match self.operator {
            "" => target == self.value,
            "^" => target.starts_with(self.value),
            "$" => target.ends_with(self.value),
            "*" => target.contains(self.value),
            _ => false,
        }
    }
}

fn without_css_comments(mut css: &str) -> String {
    let mut clean = String::with_capacity(css.len());
    while let Some((before, comment)) = css.split_once("/*") {
        clean.push_str(before);
        clean.push(' ');
        css = comment.split_once("*/").map_or("", |(_, after)| after);
    }
    clean.push_str(css);
    clean
}

// Destination overrides use simple, class-scoped attribute selectors.
// The shared fallback has no data-href selector and never counts as an override.
fn tile_styles(css: &str) -> Vec<TileStyle<'_>> {
    css.split('}')
        .filter_map(|rule| rule.rsplit_once('{'))
        .filter_map(|(selectors, declarations)| {
            let value = |property| {
                declarations
                    .split(';')
                    .rev()
                    .find_map(|declaration| {
                        let (name, value) = declaration.split_once(':')?;
                        (name.trim() == property).then(|| value.trim())
                    })
                    .filter(|value| !matches!(*value, "" | "none"))
            };
            Some((
                selectors,
                value("--dashboard-accent")?,
                value("--dashboard-icon")?,
            ))
        })
        .flat_map(|(selectors, accent, icon)| {
            selectors
                .split(',')
                .map(move |selector| (selector, accent, icon))
        })
        .filter_map(|(selector, accent, icon)| {
            let (class, attribute) = selector
                .trim()
                .strip_prefix('.')?
                .split_once(" a[data-href")?;
            let (operator, value) = attribute.split_once('=')?;
            let value = value.strip_prefix('"')?.strip_suffix("\"]")?;
            Some(TileStyle {
                class,
                operator,
                value,
                accent,
                icon,
            })
        })
        .collect()
}

#[cfg(test)]
fn test_root() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::create_dir_all(root.join(".tmp")).unwrap();
    root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_shortcuts_require_the_parent_and_current_folder() {
        let body = "# Collection\n\n> [!navigation|shortcuts]\n> - [[topics/topics|Terug]]\n> - [Finder](file:///vault/topics/a%20folder)\n";
        let back = |line: &str| line == "> - [[topics/topics|Terug]]";
        let current = Path::new("/vault/topics/a folder");
        assert!(collection_shortcuts(body, current, back));
        assert!(!collection_shortcuts(
            body,
            Path::new("/vault/topics"),
            back
        ));
        assert!(collection_shortcuts(
            &body.replace("|shortcuts]", "|back]"),
            current,
            back
        ));
        assert!(!collection_shortcuts(body, current, |_| false));
        assert!(!collection_shortcuts("# Title", current, back));
    }

    #[test]
    fn mail_can_use_an_ingestion_date_without_claiming_an_event_date() {
        let folder = "20261108-example";
        assert!(package_event_date(
            &json!({
                "category": "email", "date_basis": "ingested", "event_date": null,
                "ingested_at": "2026-11-08T12:00:00+01:00"
            }),
            folder
        ));
        assert!(!package_event_date(
            &json!({
                "category": "email", "date_basis": "ingested", "event_date": null,
                "ingested_at": "2026-11-09T12:00:00+01:00"
            }),
            folder
        ));
        assert!(!package_event_date(
            &json!({
                "category": "email", "date_basis": "ingested", "event_date": "2026-11-08"
            }),
            folder
        ));
        assert!(!package_event_date(
            &json!({"category": "note", "event_date": null}),
            folder
        ));
        assert!(!package_event_date(
            &json!({"event_date": "2026-11-09"}),
            folder
        ));
        assert!(package_event_date(
            &json!({"event_date": "2026-11-08"}),
            folder
        ));
    }

    #[test]
    fn strict_system_navigation_allows_info_but_rejects_prose() {
        let schema = json!({
            "system_layout": {"collections": ["99-system"]},
            "common_fields": {"layer": {"allowed_values": ["navigation"]}}
        });
        let mut scan = Scan {
            vault: Path::new("/vault"),
            schema: &schema,
            tile_styles: Vec::new(),
            findings: Vec::new(),
            folders: 0,
            tiles: 0,
            shortcuts: 0,
            notes: OnceCell::new(),
            dossier_tables_checked: BTreeSet::new(),
            raw_summary_checked: false,
            strict: true,
        };
        let folder = Path::new("/vault/99-system");
        let note = folder.join("99-system.md");
        let data = json!({"layer": "navigation"});
        let body = "# System\n\n> [!navigation|shortcuts]\n> - [[HOME|Back]]\n> - [Finder](file:///vault/99-system)\n> - [[99-system/agents/99-system-info|Info]]";
        scan.layer(folder, &note, "99-system", &data, body, &[]);
        assert!(scan.findings.is_empty());
        scan.layer(
            folder,
            &note,
            "99-system",
            &data,
            &format!("{body}\n\n## Werkwijze\n\nLosse uitleg."),
            &[],
        );
        assert_eq!(scan.findings.len(), 1);
        assert_eq!(scan.findings[0].code, "strict-navigation-content");
    }

    #[test]
    fn home_dashboard_checks_repeated_tile_styles_by_default() {
        let css = r#"
            .vault-dashboard a[data-href="a"] { --dashboard-accent: red; --dashboard-icon: url(a); }
            .vault-dashboard a[data-href="b"] { --dashboard-accent: RED; --dashboard-icon: url(b); }
            .vault-dashboard a[data-href="c"] { --dashboard-accent: blue; --dashboard-icon: url(b); }
        "#;
        let mut scan = Scan {
            vault: Path::new("."),
            schema: &Value::Null,
            tile_styles: tile_styles(css),
            findings: Vec::new(),
            folders: 0,
            tiles: 0,
            shortcuts: 0,
            notes: OnceCell::new(),
            dossier_tables_checked: BTreeSet::new(),
            raw_summary_checked: false,
            strict: false,
        };
        let body = "> [!navigation]\n> - [[a]]\n> - [[b]]\n> - [[c]]";
        let path = Path::new("index.md");
        scan.check_tiles(path, &json!({"cssclasses": ["vault-dashboard"]}), body)
            .unwrap();
        assert!(scan.findings.is_empty());
        scan.check_tiles(
            path,
            &json!({"cssclasses": ["vault-dashboard", "dashboard-variant-2-2"]}),
            body,
        )
        .unwrap();
        assert_eq!(
            scan.findings
                .iter()
                .map(|finding| finding.code)
                .collect::<Vec<_>>(),
            ["duplicate-tile-accent", "duplicate-tile-icon"]
        );
    }

    #[test]
    fn raw_package_checks_source_notes_at_every_depth() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let vault =
            crate::test_root().join(format!(".tmp/raw-source-{}-{nonce}", std::process::id()));
        let package = vault.join("dossier/raw/20261108-example");
        fs::create_dir_all(package.join("details")).unwrap();
        fs::create_dir_all(vault.join("dossier/wiki")).unwrap();
        fs::write(vault.join("dossier/wiki/wiki.md"), "# Wiki\n").unwrap();
        for (name, category, fields) in [
            ("20261108-example.md", "source", "wiki_status: pending\n"),
            ("missing.md", "source", ""),
            ("details/invalid.md", "general", "wiki_status: done\n"),
            ("details/history.md", "change-history", ""),
            (
                "details/valid.md",
                "source",
                "wiki_status: processed\nupdated: 2026-11-08\nwiki_processed_on: 2026-11-08\nwiki_target: \"[[dossier/wiki/wiki#Wiki]]\"\n",
            ),
            (
                "details/stale.md",
                "source",
                "wiki_status: processed\nupdated: 2026-11-08\nwiki_processed_on: 2026-11-07\nwiki_target: \"[[dossier/wiki/wiki#Wiki]]\"\n",
            ),
            (
                "details/missing-processing.md",
                "general",
                "wiki_status: processed\nwiki_processed_on: 2026-11-08\nwiki_target: \"[[dossier/wiki/wiki#Wiki]]\"\n",
            ),
        ] {
            let processing = if matches!(name, "details/valid.md" | "details/stale.md") {
                "\n## Verwerking in Wiki\n\nBronfeit samengevat in [[dossier/wiki/wiki#Wiki]].\n"
            } else {
                ""
            };
            fs::write(
                package.join(name),
                format!("---\ntype: reference\ncategory: {category}\n{fields}---\n\n# Example\n{processing}"),
            )
            .unwrap();
        }
        let schema =
            serde_saphyr::from_str(include_str!("../fixtures/layout-contract.yaml")).unwrap();
        let mut scan = Scan {
            vault: &vault,
            schema: &schema,
            tile_styles: Vec::new(),
            findings: Vec::new(),
            folders: 0,
            tiles: 0,
            shortcuts: 0,
            notes: OnceCell::new(),
            dossier_tables_checked: BTreeSet::new(),
            raw_summary_checked: false,
            strict: false,
        };
        let root = vault.join("dossier");
        let direct = root.join("dossier.md");
        fs::write(&direct, "# Ordinary folder note\n").unwrap();
        assert!(!scan.wiki_target("[[dossier/dossier]]", Some(&root)));
        fs::write(
            &direct,
            "---\nlayer: wiki\ncssclasses: [vault-wiki, vault-wiki-direct]\n---\n\n# Wiki\n",
        )
        .unwrap();
        assert!(scan.wiki_target("[[dossier/dossier#Wiki]]", Some(&root)));
        scan.folder(&package).unwrap();
        let findings: Vec<_> = scan
            .findings
            .iter()
            .map(|finding| {
                (
                    finding.path.file_name().unwrap().to_str().unwrap(),
                    finding.code,
                )
            })
            .collect();
        assert_eq!(
            findings,
            [
                ("missing.md", "metadata"),
                ("invalid.md", "wiki-status"),
                ("missing-processing.md", "wiki-processing"),
                ("stale.md", "wiki-status-stale"),
            ]
        );
        scan.findings.clear();
        scan.strict = true;
        scan.folder(&package).unwrap();
        assert!(
            scan.findings
                .iter()
                .any(|finding| finding.code == "strict-raw-markdown"
                    && finding.path.ends_with("history.md"))
        );
        assert!(
            scan.findings
                .iter()
                .any(|finding| finding.code == "wiki-processing"
                    && finding.path.ends_with("valid.md"))
        );

        let outside = root.join("20261108-outside");
        fs::create_dir(&outside).unwrap();
        fs::write(
            outside.join("20261108-outside.md"),
            "---\ntype: note\ncategory: general\n---\n# Buiten Raw\n",
        )
        .unwrap();
        scan.folder(&outside).unwrap();
        assert!(
            scan.findings
                .iter()
                .any(|finding| finding.code == "strict-package-location")
        );
        scan.layer(
            &root,
            &direct,
            "dossier",
            &json!({"layer":"navigation"}),
            "# Index\n\nLosse inhoud.",
            &[],
        );
        assert!(
            scan.findings
                .iter()
                .any(|finding| finding.code == "strict-navigation-content")
        );
        scan.layer(
            &root,
            &direct,
            "dossier",
            &json!({"layer":"wiki", "cssclasses":["vault-wiki-direct"]}),
            "# Wiki",
            &[root.join("raw")],
        );
        assert!(
            scan.findings
                .iter()
                .any(|finding| finding.code == "strict-dossier-dashboard")
        );
        fs::remove_dir_all(&vault).unwrap();
    }

    #[test]
    fn dossier_entrance_rejects_knowledge_text() {
        let valid = "# Volvo EX30\n\n> [!navigation|shortcuts]\n> - [[cars|Terug]]\n> - [Finder](file:///vault/cars/volvo)\n\n> [!navigation]\n> - [[volvo/wiki/wiki|Wiki]]\n> - [[volvo/raw/raw|Raw]]\n\n![[vault-navigation.base#Dossier recent]]";
        assert!(dossier_entrance_body(valid));
        assert!(!dossier_entrance_body(&format!(
            "{valid}\n\n## Lease\n41 maanden."
        )));
        assert!(!dossier_entrance_body(
            &valid.replace("|Wiki]]", "|Wiki]] extra tekst")
        ));
    }

    #[test]
    fn dossier_dashboard_requires_the_shared_blocks_in_order() {
        let navigation = "# Example Employer\n\n> [!navigation|shortcuts]\n> - [[employers|Back]]\n> - [Finder](file:///vault/example-employer)\n\n> [!navigation]\n> - [[example-employer/wiki/wiki|Wiki]]\n> - [[example-employer/raw/raw|Raw]]\n\n";
        let valid = format!("{navigation}{TASK_SUMMARY}\n```\n\n{OPEN_TASKS}");
        let system = valid.replace(
            "|Raw]]",
            "|Raw]]\n> - [[example-employer/archive/archive|Archive]]",
        );
        assert!(dossier_dashboard_body(&system, 1));
        assert!(!dossier_dashboard_body(&system, 0));
        assert!(!dossier_dashboard_body(&valid, 1));
        let dossier_dashboard_body = |body: &str| self::dossier_dashboard_body(body, 0);
        assert!(dossier_dashboard_body(&valid));
        assert!(!dossier_dashboard_body(&valid.replace("|Back", "|Terug")));
        assert!(!dossier_dashboard_body(
            &valid.replace("\n\n> [!navigation]", "\n> [!navigation]")
        ));
        assert!(!dossier_dashboard_body(&valid.replace(
            "![[vault-dossier.base#Open tasks]]",
            "## Open tasks\n\n![[vault-dossier.base#Open tasks]]"
        )));
        assert!(!dossier_dashboard_body(
            &valid.replace("![[vault-dossier.base#Open tasks]]", "")
        ));
        assert!(!dossier_dashboard_body(&format!(
            "{navigation}{OPEN_TASKS}"
        )));
        assert!(!dossier_dashboard_body(&format!(
            "{navigation}{OPEN_TASKS}\n\n{TASK_SUMMARY}\n```"
        )));
        assert!(!dossier_dashboard_body(&format!(
            "{valid}\n\n![[vault-raw.base#Sources to process]]"
        )));
        assert!(!dossier_dashboard_body(&format!("{valid}\n\nExtra text.")));
    }

    #[test]
    fn raw_hub_body_is_navigation_only() {
        let valid = "# Applications · Raw\n\n> [!navigation|shortcuts]\n> - [[applications|Terug]]\n> - [[applications/wiki/wiki|Wiki]]\n> - [Finder](file:///vault/applications/raw)\n\n![[vault-raw.base#Latest sources]]\n";
        assert!(raw_hub_body(valid, false, false));
        assert!(raw_hub_body(&valid.replace('\n', "\r\n"), false, false));
        let direct = valid.replace("> - [[applications/wiki/wiki|Wiki]]\n", "");
        assert!(raw_hub_body(&direct, true, false));
        assert!(!raw_hub_body(&direct, false, false));
        assert!(!raw_hub_body(valid, true, false));
        assert!(!raw_hub_body(valid, false, true));
        let with_summary = valid.replace(
            "![[vault-raw.base",
            "```vault-raw-summary\n```\n\n![[vault-raw.base",
        );
        assert!(raw_hub_body(&with_summary, false, true));
        assert!(!raw_hub_body(
            &with_summary.replace("```\n\n![[", "```\n![["),
            false,
            true
        ));
        for invalid in [
            format!("{valid}\n## Bestaande bronnen\n\n- [[source]]\n"),
            valid.replace("\n![[", "\nEen bronverwijzing\n\n![["),
            valid.replace("> - [Finder]", "> - [[source]]\n> - [Finder]"),
            valid.replace("/raw)", "/raw) Extra tekst [[source]] (slot)"),
            valid.replace("Applications · Raw", "Applications · Raw [[source]]"),
            valid.replace("![[vault-raw.base#Latest sources]]", ""),
        ] {
            assert!(!raw_hub_body(&invalid, false, false), "{invalid}");
        }
    }

    #[test]
    fn table_alias_separator_must_be_escaped() {
        let body = r"[[outside|label]]

| Type | Example |
| --- | :--- |
| Note | [[broken|label]] |
| Note | [[valid\|label]] |
| Note | [[valid]] |

~~~
| Type | Example |
| --- | --- |
| Note | [[example|label]] |
~~~";
        assert_eq!(table_wikilink_lines(body).collect::<Vec<_>>(), [5]);
    }

    #[test]
    fn package_dates_are_calendar_dates_and_slugs() {
        assert!(package_name("20240229-a-log"));
        for name in [
            "20260229-a",
            "20261301-a",
            "20261101-A",
            "20261101-a--b",
            "20261101-",
        ] {
            assert!(!package_name(name), "{name}");
        }
    }

    #[test]
    fn vault_tiles_use_the_physical_folder_name() {
        assert!(vault_tile_label_matches(
            "99-system/vault/structuur-en-opslag/structuur-en-opslag",
            "Structuur en opslag"
        ));
        assert!(!vault_tile_label_matches(
            "99-system/vault/sources/sources",
            "Bronnen"
        ));
    }

    #[test]
    fn content_tiles_exclude_shortcuts_and_examples() {
        let body = "```md\n> [!navigation]\n> - [[example]]\n```\n> [!navigation|back]\n> - [[parent]]\n\n> [!navigation]\n> - [[a/b/b|B]]\n> - [[a/c/c.md]]";
        assert_eq!(
            tiles(body, false).collect::<Vec<_>>(),
            [Ok("a/b/b"), Ok("a/c/c.md")]
        );
        assert_eq!(
            tiles(
                "> [!navigation]\n> * [External](https://example.org)\n> - [[a/b/b.md.md]]",
                false
            )
            .collect::<Vec<_>>(),
            [Err(()), Ok("a/b/b.md.md")]
        );
    }

    #[test]
    fn wiki_and_raw_shortcuts_keep_heading_anchors() {
        let body = "> [!navigation|shortcuts]\n> - [[parent|Terug]]\n> - [[a/wiki/wiki#Stand|Wiki]]\n> - [[a/raw/raw#Bronnen|Raw]]\n> - [Finder](file:///a)\n\n> [!navigation]\n> - [[a/child/child|Child]]";
        assert_eq!(
            tiles(body, true).collect::<Vec<_>>(),
            [Ok("a/wiki/wiki#Stand"), Ok("a/raw/raw#Bronnen")]
        );
        assert_eq!(
            tiles(body, false).collect::<Vec<_>>(),
            [Ok("a/child/child")]
        );
    }

    #[test]
    fn destination_styles_require_a_matching_selector_icon_and_tint() {
        let css = without_css_comments(
            r#"
          :where(.vault-dashboard a) { --dashboard-icon: url(folder); --dashboard-accent: blue; }
          .vault-dashboard a[data-href="books"],
          .residence-navigation a[data-href^="residence/"],
          .vault-dashboard a[data-href$="/wiki/wiki"],
          .vault-dashboard a[data-href*="/wiki/wiki#"],
          .vault-dashboard a[data-href*="/raw/raw#"] {
            --dashboard-icon: url(book); --dashboard-accent: cyan;
          }
          .vault-dashboard a[data-href="incomplete"] { --dashboard-icon: url(book); }
          /*
          .vault-dashboard a[data-href="disabled-one"] { --dashboard-icon: url(book); --dashboard-accent: red; }
          .vault-dashboard a[data-href="disabled-two"] { --dashboard-icon: url(book); --dashboard-accent: red; }
          */
        "#,
        );
        let styles = tile_styles(&css);
        assert_eq!(styles.len(), 5);
        assert!(styles[0].matches("books"));
        assert!(!styles[0].matches("books.md"));
        assert_eq!(styles[1].class, "residence-navigation");
        assert!(styles[1].matches("residence/house"));
        assert!(styles[2].matches("dossier/wiki/wiki"));
        assert!(styles[3].matches("dossier/wiki/wiki#Section"));
        assert!(styles[4].matches("dossier/raw/raw#Section"));
        assert!(!styles.iter().any(|style| style.matches("incomplete")));
        assert!(!styles.iter().any(|style| style.matches("unknown")));
    }
}
