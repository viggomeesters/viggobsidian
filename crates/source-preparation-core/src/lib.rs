//! Pure native-source postimages. No filesystem, registry, clock or write authority.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_NOTE_BYTES: usize = 1024 * 1024;
pub const FORMAT: &str = "source-capture-v1";

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

fn note(text: &str) -> Result<(&str, &str, Value)> {
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
    if !metadata.is_object() || metadata["type"] != "note" || metadata["category"] != "source" {
        return Err("plain_note_source_required");
    }
    Ok((header, body, metadata))
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
    ];
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("unknown_field");
    }
    if v["schema"] != "viggobsidian.source-preparation.v1" || v["format"] != FORMAT {
        return Err("schema_or_format");
    }
    let operation = field(v, "operation")?;
    if !["prepare", "mark-wiki-processed"].contains(&operation) {
        return Err("operation");
    }
    let day = field(v, "date")?;
    if !date(day) {
        return Err("invalid_date");
    }
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
    let (header, body, metadata) = note(before)?;
    let classes = json!([
        "vault-navigation",
        "vault-dashboard",
        "vault-event",
        "vault-event-v1-1"
    ]);
    let after;
    let original_hash;
    if operation == "prepare" {
        if [
            "wiki_status",
            "cssclasses",
            "activity_format",
            "raw_preparation_format",
            "activity_contract",
            "wiki_processed_on",
            "wiki_target",
        ]
        .iter()
        .any(|key| metadata.get(key).is_some())
            || body.lines().any(|line| line == "## Activity")
            || [
                "original_note",
                "wiki_target",
                "summary",
                "preparation_date",
            ]
            .iter()
            .any(|key| v.get(key).is_some())
        {
            return Err("unprepared_source_required");
        }
        let title = body
            .lines()
            .find_map(|line| line.strip_prefix("# "))
            .unwrap_or(package);
        let finder = url(&format!("{vault}/{raw}/{package}"));
        after = format!(
            "{header}\nwiki_status: pending\ncssclasses: [vault-navigation, vault-dashboard, vault-event, vault-event-v1-1]\nraw_preparation_format: source-capture-v1\nactivity_contract: provenance-v1\n---\n\n# {title}\n\n> [!navigation|shortcuts]\n> - [[{raw}/raw|Back]]\n> - [Finder](file://{finder})\n\n> [!wiki-facts|dossier]\n> - Dossier **[[{dossier}/{label}|{label}]]**\n\n```vault-event-detail\n```\n\n> [!event-context] Context\n> {title}\n\n## Activity\n\n> [!activity-day|{day}] {day}\n\n> [!activity-message|agent] Agent\n> `— · Agent · vault-process`\n> <!-- viggobsidian:activity:v1 kind=result goal=raw-preparation -->\n> De oorspronkelijke inhoud is hieronder behouden voor plaatsing bij dit dossier.\n>\n> > [!info]- Meer\n{}",
            quoted(body)
        );
        original_hash = hash(before);
    } else {
        if metadata["wiki_status"] != "pending"
            || metadata["raw_preparation_format"] != FORMAT
            || metadata["activity_contract"] != "provenance-v1"
            || metadata["cssclasses"] != classes
            || metadata.get("wiki_processed_on").is_some()
            || metadata.get("wiki_target").is_some()
        {
            return Err("prepared_pending_source_required");
        }
        let original = field(v, "original_note")?;
        let preparation_date = field(v, "preparation_date")?;
        if !date(preparation_date) || preparation_date > day {
            return Err("invalid_preparation_date");
        }
        let (_, original_body, _) = note(original)?;
        if !body.ends_with(&quoted(original_body)) {
            return Err("original_body_changed");
        }
        let prepared = transform(
            &json!({"schema":"viggobsidian.source-preparation.v1", "format":FORMAT,
            "operation":"prepare", "source_note":original, "date":preparation_date,
            "source_path":format!("00-inbox/{package}/{filename}"), "destination_owner":owner, "vault_root":vault}),
        )?;
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
        after = format!(
            "{changed_header}\nwiki_processed_on: {day}\nwiki_target: {target_json}\n---\n{body}\n> [!activity-day|{day}] {day}\n\n> [!activity-message|agent] Agent\n> <!-- viggobsidian:activity:v1 kind=result goal=wiki-processing -->\n> **Verwerking in Wiki**\n> {} {target}\n>\n> *— · Agent · vault-process*\n",
            summary.trim()
        );
        original_hash = hash(original);
    }
    if after.len() > 2 * MAX_NOTE_BYTES {
        return Err("postimage_limit");
    }
    Ok(
        json!({"schema":"viggobsidian.source-preparation-result.v1", "status":"complete",
        "engine":"source-preparation-core", "engine_version":env!("CARGO_PKG_VERSION"), "format":FORMAT,
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
