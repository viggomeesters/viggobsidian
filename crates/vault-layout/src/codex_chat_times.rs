//! Explicit local evidence check; ordinary vault scans never read rollout files.
use activity_core::{activity_delivery::codex_chat_footers, provenance};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    path::Path,
};

const MAX_REQUEST: u64 = 64 * 1024;
const MAX_NOTE: u64 = 256 * 1024;
const MAX_ROLLOUT_PREFIX: u64 = 64 * 1024 * 1024;
const MAX_BOUND_RECORD: usize = 2 * 1024 * 1024;
const SCHEMA: &str = "viggobsidian.codex-chat-time-bindings.v1";

pub fn run(
    note: &Path,
    rollout: &Path,
    input: impl Read,
) -> Result<u8, Box<dyn std::error::Error>> {
    let request = json_object(&bounded(input, MAX_REQUEST)?)?;
    let note = bounded(open_local(note)?, MAX_NOTE)?;
    let (header, body) =
        crate::frontmatter(&note).ok_or("expected an Event Note with frontmatter")?;
    let options = serde_saphyr::options! {
        duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
        merge_keys: serde_saphyr::MergeKeyPolicy::Error
    };
    let metadata: Value = serde_saphyr::from_str_with_options(&header, options)
        .map_err(|_| "invalid Event Note frontmatter")?;
    if metadata["activity_contract"] != "provenance-v1"
        || !metadata["cssclasses"]
            .as_array()
            .is_some_and(|classes| classes.iter().any(|class| class == "vault-event-v1-1"))
    {
        return Err("expected an Event Note v1.1 with provenance-v1".into());
    }
    let count = check(
        &body,
        BufReader::new(open_local(rollout)?.take(MAX_ROLLOUT_PREFIX)),
        &request,
    )?;
    if count == 0 {
        println!("Codex chat timestamps verified against the supplied message bindings");
    }
    Ok(u8::from(count != 0))
}

fn open_local(path: &Path) -> Result<File, &'static str> {
    if !path.is_absolute()
        || !fs::symlink_metadata(path).is_ok_and(|info| info.file_type().is_file())
    {
        return Err("expected two explicitly selected absolute regular file paths");
    }
    let file = File::open(path).map_err(|_| "selected input file could not be opened")?;
    if !file.metadata().is_ok_and(|info| info.is_file()) {
        return Err("selected input is not a regular file");
    }
    Ok(file)
}

fn bounded(input: impl Read, limit: u64) -> Result<String, &'static str> {
    let mut text = String::new();
    input
        .take(limit + 1)
        .read_to_string(&mut text)
        .map_err(|_| "input could not be read as UTF-8")?;
    if text.len() as u64 > limit {
        return Err("input exceeds the source check limit");
    }
    Ok(text)
}

fn json_object(input: &str) -> Result<Value, &'static str> {
    // Require JSON syntax, and reject duplicate keys rather than choosing a value.
    let value: Value = serde_json::from_str(input).map_err(|_| "invalid source check JSON")?;
    let options = serde_saphyr::options! {
        duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
        merge_keys: serde_saphyr::MergeKeyPolicy::Error
    };
    let _: Value = serde_saphyr::from_str_with_options(input, options)
        .map_err(|_| "duplicate or invalid source check JSON fields")?;
    Ok(value)
}

fn check(body: &str, mut source: impl BufRead, request: &Value) -> Result<usize, String> {
    if body.lines().count() > 4_000 {
        return Err("Event Note exceeds the source check line limit".into());
    }
    let footers = codex_chat_footers(body)?;
    if request.as_object().is_none_or(|o| o.len() != 4) || request["schema"] != SCHEMA {
        return Err("invalid Codex chat bindings schema".into());
    }
    let session = request["session_id"]
        .as_str()
        .filter(|id| {
            id.len() == 36
                && id.bytes().enumerate().all(|(i, c)| {
                    if matches!(i, 8 | 13 | 18 | 23) {
                        c == b'-'
                    } else {
                        c.is_ascii_hexdigit()
                    }
                })
        })
        .ok_or("expected the original Codex session UUID")?;
    let timezone = request["display_timezone"]
        .as_str()
        .ok_or("expected display timezone")?;
    let lines = request["message_lines"]
        .as_array()
        .filter(|lines| lines.len() == footers.len())
        .ok_or("bind exactly one original message line to every Activity bubble")?;
    let mut line = Vec::new();
    let mut row = 0;
    read_through(&mut source, &mut line, &mut row, 1)?;
    let header = record(&line, row)?;
    if header["type"] != "session_meta" || header["payload"]["id"] != session {
        return Err("selected rollout does not start with the bound Codex session".into());
    }

    let mut mismatches = Vec::new();
    for (index, (footer, binding)) in footers.iter().zip(lines).enumerate() {
        let number = index + 1;
        let target = binding.as_u64().filter(|target| *target > row).ok_or(
            "message lines must be strictly increasing physical line numbers after session_meta",
        )?;
        read_through(&mut source, &mut line, &mut row, target)?;
        let record = record(&line, row)?;
        let payload = &record["payload"];
        let (role, content_type) = if footer.role == "human" {
            ("user", "input_text")
        } else {
            ("assistant", "output_text")
        };
        if record["type"] != "response_item"
            || payload["type"] != "message"
            || payload["role"] != role
            || (role == "assistant" && payload["phase"] != "final_answer")
            || matches!(
                payload["channel"].as_str(),
                Some("analysis" | "commentary" | "justify" | "confidence")
            )
            || !payload["content"].as_array().is_some_and(|content| {
                content.iter().any(|part| {
                    part["type"] == content_type
                        && part["text"]
                            .as_str()
                            .is_some_and(|text| !text.trim().is_empty())
                })
            })
        {
            return Err(format!(
                "bubble {number}: bound row is not an original message of the expected role"
            ));
        }
        let timestamp = record["timestamp"]
            .as_str()
            .ok_or_else(|| format!("bubble {number}: original message time unavailable"))?;
        let facts = json!({
            "kind": "chat-message",
            "timestamp": timestamp,
            "author": footer.author,
            "channel": "Codex-chat",
            "display_timezone": timezone
        });
        let expected = provenance::render(&facts).map_err(|reason| {
            format!("bubble {number}: source time cannot be verified ({reason})")
        })?;
        if footer.day != expected.day || footer.stamp != expected.stamp {
            let time = expected.stamp.split_once(" · ").unwrap().0;
            mismatches.push(format!(
                "bubble {number}: expected Activity day {} and time {time}",
                expected.day
            ));
        }
    }
    for mismatch in &mismatches {
        eprintln!("vault-layout: {mismatch}");
    }
    Ok(mismatches.len())
}

fn read_through(
    source: &mut impl BufRead,
    line: &mut Vec<u8>,
    row: &mut u64,
    target: u64,
) -> Result<(), String> {
    while *row < target {
        line.clear();
        if source
            .read_until(b'\n', line)
            .map_err(|_| "rollout could not be read")?
            == 0
            || line.last() != Some(&b'\n')
        {
            return Err(format!(
                "selected rollout line {target} is unavailable or incomplete within the input limit"
            ));
        }
        *row += 1;
    }
    Ok(())
}

fn record(line: &[u8], row: u64) -> Result<Value, String> {
    if line.len() > MAX_BOUND_RECORD {
        return Err(format!(
            "bound rollout record at line {row} exceeds the input limit"
        ));
    }
    std::str::from_utf8(line)
        .ok()
        .and_then(|line| json_object(line.trim_end_matches(['\r', '\n'])).ok())
        .ok_or_else(|| format!("invalid or ambiguous rollout record at line {row}"))
}
