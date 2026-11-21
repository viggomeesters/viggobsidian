//! Pure native-source postimages. No filesystem, registry, clock or write authority.
use activity_core::activity_delivery::provenance_findings;
use activity_core::provenance::{self, RenderedProvenance, SourceKind};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_NOTE_BYTES: usize = 1024 * 1024;
pub const FORMAT: &str = "source-capture-v1";
pub const PROVENANCE_FORMAT: &str = "source-capture-v2";
pub const AUTHORED_ACTIVITY_FORMAT: &str = "authored-activity-v1";

type Result<T> = std::result::Result<T, &'static str>;

pub fn hash(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

fn field<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key).and_then(Value::as_str).ok_or("required_string")
}

fn date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(i, b)| i != 4 && i != 7 && !b.is_ascii_digit())
    {
        return false;
    }
    let Ok(year) = value[..4].parse::<u32>() else {
        return false;
    };
    let Ok(month) = value[5..7].parse::<usize>() else {
        return false;
    };
    let Ok(day) = value[8..].parse::<u32>() else {
        return false;
    };
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    year > 0 && (1..=12).contains(&month) && day > 0 && day <= days[month - 1]
}

fn relative(value: &str) -> bool {
    !value.is_empty()
        && !value.contains(['\\', '\r', '\n', '\0', '[', ']', '|', '#', '<', '>'])
        && value
            .split('/')
            .all(|part| !part.is_empty() && !part.starts_with('.'))
}

fn root(value: &str) -> bool {
    value.starts_with('/')
        && value.len() > 1
        && !value.contains(['\\', '\r', '\n', '\0'])
        && value[1..]
            .split('/')
            .all(|part| !part.is_empty() && !matches!(part, "." | ".."))
}

fn url(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
            output.push(char::from(byte));
        } else {
            use std::fmt::Write;
            write!(output, "%{byte:02X}").expect("write to String");
        }
    }
    output
}

fn note<'a>(text: &'a str, format: &str) -> Result<(&'a str, &'a str, Value)> {
    if text.len() > MAX_NOTE_BYTES || text.contains('\0') {
        return Err("note_limit_or_encoding");
    }
    let (header, body) = text.split_once("\n---\n").ok_or("frontmatter_required")?;
    let yaml = header.strip_prefix("---\n").ok_or("frontmatter_required")?;
    // Enforce merge semantics at the parser, including nested/flow/tagged keys.
    // Quoted or explicitly string-tagged << keys remain ordinary source data.
    let options = serde_saphyr::options! {
        duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
        merge_keys: serde_saphyr::MergeKeyPolicy::Error
    };
    let metadata: Value = serde_saphyr::from_str_with_options(yaml, options).map_err(|error| {
        if matches!(error, serde_saphyr::Error::MergeKeyNotAllowed { .. }) {
            "metadata_merge_refused"
        } else {
            "metadata_invalid"
        }
    })?;
    if format == AUTHORED_ACTIVITY_FORMAT {
        if !metadata.is_object()
            || metadata["type"] != "note"
            || metadata["category"] != "general"
            || metadata["activity_contract"] != "provenance-v1"
            || [
                "task-id",
                "task_id",
                "component-id",
                "component_id",
                "project_id",
                "layer",
                "source_archives",
            ]
            .iter()
            .any(|key| metadata.get(key).is_some())
        {
            return Err("authored_activity_note_required");
        }
    } else if !metadata.is_object()
        || metadata["type"] != "note"
        || metadata["category"] != "source"
    {
        return Err("plain_note_source_required");
    }
    Ok((header, body, metadata))
}

fn authored_sections<'a>(body: &'a str, day: &str) -> Result<(&'a str, String, &'a str)> {
    let (intro, activity) = body
        .split_once("\n## Activity\n")
        .ok_or("authored_activity_required")?;
    if !provenance_findings(body).is_empty() {
        return Err("invalid_authored_activity_provenance");
    }
    let mut first_in_block = true;
    for line in activity.lines() {
        if first_in_block
            && let Some((existing_day, _)) = line
                .strip_prefix("> [!activity-day|")
                .and_then(|rest| rest.split_once("] "))
            && existing_day > day
        {
            return Err("authored_activity_after_preparation_date");
        }
        first_in_block = line.trim().is_empty();
    }
    let (title, context) = intro
        .trim_start_matches('\n')
        .split_once("\n\n")
        .ok_or("authored_activity_title_and_context_required")?;
    let title = title
        .strip_prefix("# ")
        .ok_or("authored_activity_title_and_context_required")?;
    let context = context.trim_end_matches('\n');
    if title.trim().is_empty()
        || title.contains(['\r', '\n'])
        || context.is_empty()
        || context.lines().any(|line| {
            line.trim().is_empty()
                || line.starts_with(['#', '>', ' ', '\t', '<', '`', '~', '=', '|', '-'])
                || line.starts_with("* ")
                || line.starts_with("+ ")
                || line.contains('\r')
        })
    {
        return Err("authored_activity_title_and_context_required");
    }
    let quoted_context = context
        .lines()
        .map(|line| format!("> {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    Ok((title, quoted_context, activity))
}

fn quoted(body: &str) -> String {
    let mut out = String::new();
    for line in body.split_inclusive('\n') {
        out.push_str("> > ");
        out.push_str(line);
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn contribution<'a>(value: Option<&'a Value>, day: &str) -> Result<Option<RenderedProvenance<'a>>> {
    let rendered = value.map(provenance::render).transpose()?;
    if rendered
        .as_ref()
        .is_some_and(|event| event.kind != SourceKind::VaultContribution || event.day != day)
    {
        return Err("vault_contribution_kind_or_day_mismatch");
    }
    Ok(rendered)
}

fn provenance_pin(key: &str, value: Option<&Value>) -> String {
    value.map_or_else(String::new, |value| {
        format!("\n{key}: \"{}\"", hash(&value.to_string()))
    })
}

fn transform(v: &Value) -> Result<Value> {
    let object = v.as_object().ok_or("request_object_required")?;
    let allowed = [
        "schema",
        "format",
        "operation",
        "source_note",
        "date",
        "source_path",
        "destination_owner",
        "vault_root",
        "original_note",
        "wiki_target",
        "summary",
        "preparation_date",
        "provenance",
        "preparation_provenance",
    ];
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("unknown_field");
    }
    let format = field(v, "format")?;
    if v["schema"] != "viggobsidian.source-preparation.v1"
        || !matches!(
            format,
            FORMAT | PROVENANCE_FORMAT | AUTHORED_ACTIVITY_FORMAT
        )
    {
        return Err("schema_or_format");
    }
    if format == FORMAT
        && (v.get("provenance").is_some() || v.get("preparation_provenance").is_some())
    {
        return Err("provenance_requires_source_capture_v2");
    }
    let authored = format == AUTHORED_ACTIVITY_FORMAT;
    if format != FORMAT && v.get("provenance").is_none() {
        return Err("vault_contribution_provenance_required");
    }
    let operation = field(v, "operation")?;
    if !["prepare", "mark-wiki-processed"].contains(&operation) {
        return Err("operation");
    }
    let day = field(v, "date")?;
    if !date(day) {
        return Err("invalid_date");
    }
    let event = contribution(v.get("provenance"), day)?;
    let owner = field(v, "destination_owner")?;
    let source = field(v, "source_path")?;
    let vault = field(v, "vault_root")?;
    if !relative(owner) || !relative(source) || !root(vault) {
        return Err("unsafe_path");
    }
    let (dossier, owner_file) = owner.rsplit_once('/').ok_or("dossier_owner_required")?;
    let label = dossier.rsplit('/').next().ok_or("dossier_owner_required")?;
    if owner_file != format!("{label}.md") {
        return Err("dossier_owner_required");
    }
    let (package_path, filename) = source.rsplit_once('/').ok_or("package_source_required")?;
    let package = package_path
        .rsplit('/')
        .next()
        .ok_or("package_source_required")?;
    let bytes = package.as_bytes();
    if bytes.len() < 10
        || bytes[8] != b'-'
        || !bytes[..8].iter().all(u8::is_ascii_digit)
        || !bytes[9].is_ascii_lowercase() && !bytes[9].is_ascii_digit()
        || !bytes[9..]
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
        || filename != format!("{package}.md")
    {
        return Err("dated_primary_package_required");
    }
    let package_day = format!("{}-{}-{}", &package[..4], &package[4..6], &package[6..8]);
    if !date(&package_day) {
        return Err("dated_primary_package_required");
    }
    let raw = format!("{dossier}/raw");
    let prepared_path = format!("{raw}/{package}/{filename}");
    if (operation == "prepare" && source != format!("00-inbox/{package}/{filename}"))
        || (operation == "mark-wiki-processed" && source != prepared_path)
    {
        return Err("source_destination_mismatch");
    }
    let before = field(v, "source_note")?;
    let (header, body, metadata) = note(before, format)?;
    let classes = json!([
        "vault-navigation",
        "vault-dashboard",
        "vault-event",
        "vault-event-v1-1"
    ]);
    let after;
    let original_hash;
    if operation == "prepare" {
        original_hash = hash(before);
        if [
            "wiki_status",
            "cssclasses",
            "activity_format",
            "raw_preparation_format",
            "wiki_processed_on",
            "wiki_target",
            "raw_preparation_provenance_sha256",
            "wiki_processing_provenance_sha256",
        ]
        .iter()
        .any(|key| metadata.get(key).is_some())
            || (authored && metadata.get("raw_preparation_original_sha256").is_some())
            || (!authored
                && (metadata.get("activity_contract").is_some()
                    || body.lines().any(|line| line == "## Activity")))
            || [
                "original_note",
                "wiki_target",
                "summary",
                "preparation_date",
                "preparation_provenance",
            ]
            .iter()
            .any(|key| v.get(key).is_some())
        {
            return Err("unprepared_source_required");
        }
        let finder = url(&format!("{vault}/{raw}/{package}"));
        let entrance = |title: &str, context: &str| {
            format!(
                "# {title}\n\n> [!navigation|shortcuts]\n> - [[{raw}/raw|Back]]\n> - [Finder](file://{finder})\n\n> [!wiki-facts|dossier]\n> - Dossier **[[{dossier}/{label}|{label}]]**\n\n```vault-event-detail\n```\n\n> [!event-context] Context\n{context}\n\n## Activity\n"
            )
        };
        let pin = provenance_pin("raw_preparation_provenance_sha256", v.get("provenance"));
        after = if authored {
            let (title, context, activity) = authored_sections(body, day)?;
            let entrance = entrance(title, &context);
            format!(
                "{header}\nwiki_status: pending\ncssclasses: [vault-navigation, vault-dashboard, vault-event, vault-event-v1-1]\nraw_preparation_format: {format}\nraw_preparation_original_sha256: \"{original_hash}\"{pin}\n---\n\n{entrance}{activity}"
            )
        } else {
            let title = body
                .lines()
                .find_map(|line| line.strip_prefix("# "))
                .unwrap_or(package);
            let entrance = entrance(title, &format!("> {title}"));
            let actor = event.as_ref().map_or("Agent", |event| event.author);
            let legacy = if event.is_none() {
                "> `— · Agent · vault-process`\n"
            } else {
                ""
            };
            let footer = event
                .as_ref()
                .map_or_else(String::new, RenderedProvenance::footer);
            format!(
                "{header}\nwiki_status: pending\ncssclasses: [vault-navigation, vault-dashboard, vault-event, vault-event-v1-1]\nraw_preparation_format: {format}\nactivity_contract: provenance-v1{pin}\n---\n\n{entrance}\n> [!activity-day|{day}] {day}\n\n> [!activity-message|agent] {actor}\n{legacy}> <!-- viggobsidian:activity:v1 kind=result goal=raw-preparation -->\n> De oorspronkelijke inhoud is hieronder behouden voor plaatsing bij dit dossier.\n>\n> > [!info]- Meer\n{}{footer}",
                quoted(body)
            )
        };
    } else {
        let preparation_format = metadata["raw_preparation_format"]
            .as_str()
            .filter(|&prepared| prepared == format || (!authored && prepared == FORMAT))
            .ok_or("prepared_pending_source_required")?;
        if metadata["wiki_status"] != "pending"
            || metadata["activity_contract"] != "provenance-v1"
            || metadata["cssclasses"] != classes
            || metadata.get("wiki_processed_on").is_some()
            || metadata.get("wiki_target").is_some()
            || metadata.get("wiki_processing_provenance_sha256").is_some()
        {
            return Err("prepared_pending_source_required");
        }
        let original = field(v, "original_note")?;
        let preparation_date = field(v, "preparation_date")?;
        if !date(preparation_date) || preparation_date > day {
            return Err("invalid_preparation_date");
        }
        let (_, original_body, _) = note(original, format)?;
        let preparation = v.get("preparation_provenance");
        if (preparation_format != FORMAT) != preparation.is_some() {
            return Err("preparation_provenance_does_not_match_format");
        }
        let previous = contribution(preparation, preparation_date)?;
        let previous_footer = previous
            .as_ref()
            .map_or_else(String::new, RenderedProvenance::footer);
        if !authored
            && !body
                .strip_suffix(&previous_footer)
                .is_some_and(|body| body.ends_with(&quoted(original_body)))
        {
            return Err("original_body_changed");
        }
        let mut prepare = json!({"schema":"viggobsidian.source-preparation.v1", "format":preparation_format,
            "operation":"prepare", "source_note":original, "date":preparation_date,
            "source_path":format!("00-inbox/{package}/{filename}"), "destination_owner":owner, "vault_root":vault});
        if let Some(preparation) = preparation {
            prepare["provenance"] = preparation.clone();
        }
        let prepared = transform(&prepare)?;
        if prepared["postimage"].as_str() != Some(before) {
            return Err("prepared_postimage_changed");
        }
        let target = field(v, "wiki_target")?;
        let target_prefix = format!("[[{dossier}/wiki/wiki#");
        let anchor = target
            .strip_prefix(&target_prefix)
            .and_then(|s| s.strip_suffix("]]"))
            .ok_or("wiki_target_required")?;
        if anchor.trim().is_empty() || anchor.contains(['\n', '\r', '[', ']', '|']) {
            return Err("wiki_target_required");
        }
        let summary = field(v, "summary")?;
        if summary.trim().is_empty()
            || summary.len() > 16384
            || summary.contains(['\n', '\r', '\0'])
        {
            return Err("processing_summary");
        }
        if header
            .lines()
            .filter(|line| *line == "wiki_status: pending")
            .count()
            != 1
        {
            return Err("ambiguous_status_field");
        }
        let changed_header =
            header.replacen("\nwiki_status: pending\n", "\nwiki_status: processed\n", 1);
        if changed_header == header {
            return Err("ambiguous_status_field");
        }
        let target_json = serde_json::to_string(target).map_err(|_| "wiki_target_required")?;
        let actor = event.as_ref().map_or("Agent", |event| event.author);
        let footer = event.as_ref().map_or_else(
            || provenance::legacy_unknown_footer(actor, "vault-process"),
            RenderedProvenance::footer,
        );
        let pin = provenance_pin("wiki_processing_provenance_sha256", v.get("provenance"));
        // Keep the Wiki link as direct text. A long or Markdown-control
        // summary goes under Details without truncating or activating it.
        let summary_text = summary.trim();
        let unsafe_leading = [">", "!", "*", "[!", "```", "~~~", "<!--"]
            .iter()
            .any(|prefix| summary_text.starts_with(prefix));
        let processing_text =
            if format != FORMAT && (summary_text.len() + target.len() > 900 || unsafe_leading) {
                format!(
                    "> Samenvatting van de Wiki-verwerking: {target}\n{}",
                    provenance::folded_verbatim(summary)
                )
            } else {
                format!("> {summary_text} {target}\n")
            };
        let separator = if authored && !body.ends_with('\n') {
            "\n\n"
        } else {
            "\n"
        };
        after = format!(
            "{changed_header}\nwiki_processed_on: {day}\nwiki_target: {target_json}{pin}\n---\n{body}{separator}> [!activity-day|{day}] {day}\n\n> [!activity-message|agent] {actor}\n> <!-- viggobsidian:activity:v1 kind=result goal=wiki-processing -->\n> **Verwerking in Wiki**\n{processing_text}{footer}",
        );
        original_hash = hash(original);
    }
    if after.len() > 2 * MAX_NOTE_BYTES {
        return Err("postimage_limit");
    }
    if format != FORMAT
        && !provenance_findings(
            after
                .split_once("\n---\n")
                .expect("postimage has metadata")
                .1,
        )
        .is_empty()
    {
        return Err("invalid_rendered_activity_provenance");
    }
    Ok(
        json!({"schema":"viggobsidian.source-preparation-result.v1", "status":"complete",
        "engine":"source-preparation-core", "engine_version":env!("CARGO_PKG_VERSION"), "format":format,
        "operation":operation, "source_path":source, "prepared_path":prepared_path,
        "before_sha256":hash(before), "after_sha256":hash(&after), "original_sha256":original_hash, "postimage":after}),
    )
}

pub fn respond(input: &[u8]) -> Value {
    let result = if input.len() > MAX_REQUEST_BYTES {
        Err("request_limit")
    } else {
        serde_json::from_slice::<Value>(input).map_err(|_| "invalid_json").and_then(|v| {
            let options = serde_saphyr::options! { duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error };
            let text = std::str::from_utf8(input).map_err(|_| "invalid_json")?;
            let _: Value = serde_saphyr::from_str_with_options(text, options).map_err(|_| "duplicate_or_invalid_json")?;
            transform(&v)
        })
    };
    result.unwrap_or_else(|reason| json!({"schema":"viggobsidian.source-preparation-result.v1", "status":"incomplete", "reason":reason}))
}
