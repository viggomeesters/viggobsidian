//! Format adapter-supplied source facts. This module has no clock, vault, or write authority.
use jiff::{Timestamp, tz::TimeZone};
use serde_json::{Value, json};

pub const MAX_REQUEST_BYTES: usize = 8 * 1024;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SourceKind {
    ChatMessage,
    LauncherAction,
    VaultContribution,
}

impl SourceKind {
    fn parse(input: &Value) -> Result<Self, &'static str> {
        match input.as_str() {
            Some("chat-message") => Ok(Self::ChatMessage),
            Some("launcher-action") => Ok(Self::LauncherAction),
            Some("vault-contribution") => Ok(Self::VaultContribution),
            _ => Err("invalid_provenance_kind"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RenderedProvenance<'a> {
    pub kind: SourceKind,
    pub day: String,
    pub author: &'a str,
    pub stamp: String,
}

impl RenderedProvenance<'_> {
    /// Append after all body text (including any folded Details), not in the title.
    pub fn footer(&self) -> String {
        footer(&self.stamp)
    }
}

fn footer(stamp: &str) -> String {
    format!(">\n> *{stamp}*\n")
}

/// For legacy adapter requests with a known day but no structured clock input.
/// The caller owns the legacy author and channel validation.
pub fn legacy_unknown_footer(author: &str, channel: &str) -> String {
    footer(&format!("— · {author} · {channel}"))
}

/// Keep literal source inside a folded block, never as live Activity syntax.
/// Quote prefixes and one terminal newline form the Markdown envelope.
pub fn folded_verbatim(text: &str) -> String {
    let width = text
        .split(|c| c != '`')
        .map(str::len)
        .max()
        .unwrap_or(0)
        .max(2)
        + 1;
    let fence = "`".repeat(width);
    let mut result = format!(">\n> > [!info]- Meer\n> > {fence}text\n");
    for line in text.split_inclusive('\n') {
        result.push_str("> > ");
        result.push_str(line);
        if !line.ends_with('\n') {
            result.push('\n');
        }
    }
    result.push_str(&format!("> > {fence}\n"));
    result
}

fn label<'a>(value: &'a Value, key: &str) -> Result<&'a str, &'static str> {
    value[key]
        .as_str()
        .filter(|v| {
            !v.is_empty()
                && v.len() <= 256
                && *v == v.trim()
                && !v.contains(" · ")
                && !v.contains(['<', '>', '&', '`', '*', '_', '[', ']', '\\', '~', '|'])
                && !v
                    .chars()
                    .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
        })
        .ok_or("invalid_provenance_label")
}

fn from_rfc3339(input: &str) -> Result<Timestamp, &'static str> {
    // Jiff also accepts the wider Temporal syntax. This boundary accepts an
    // RFC 3339 civil datetime with seconds and an explicit source offset only.
    let civil = if let Some(civil) = input.strip_suffix(['Z', 'z']) {
        civil
    } else {
        let split = input
            .len()
            .checked_sub(6)
            .filter(|&i| input.is_char_boundary(i))
            .ok_or("invalid_provenance_timestamp")?;
        let (civil, offset) = input.split_at(split);
        let b = offset.as_bytes();
        if !matches!(b[0], b'+' | b'-')
            || b[3] != b':'
            || ![b[1], b[2], b[4], b[5]].iter().all(u8::is_ascii_digit)
            || &b[1..3] > b"23"
            || &b[4..6] > b"59"
        {
            return Err("invalid_provenance_timestamp");
        }
        civil
    };
    let b = civil.as_bytes();
    if b.len() < 19
        || b.len() > 29
        || !civil.get(..10).is_some_and(crate::iso_date)
        || !matches!(b[10], b'T' | b't')
        || b[13] != b':'
        || b[16] != b':'
        || ![b[11], b[12], b[14], b[15], b[17], b[18]]
            .iter()
            .all(u8::is_ascii_digit)
        || (b.len() > 19
            && (b[19] != b'.' || b.len() == 20 || !b[20..].iter().all(u8::is_ascii_digit)))
        || &b[17..19] > b"59"
    {
        return Err("invalid_provenance_timestamp");
    }
    input.parse().map_err(|_| "invalid_provenance_timestamp")
}

/// Known timestamps keep their original offset until conversion to the supplied
/// IANA zone. Unknown time requires an independently known local Activity day.
pub fn render(input: &Value) -> Result<RenderedProvenance<'_>, &'static str> {
    let fields = [
        "kind",
        "timestamp",
        "day",
        "author",
        "channel",
        "display_timezone",
    ];
    let object = input.as_object().ok_or("invalid_provenance")?;
    if object.keys().any(|key| !fields.contains(&key.as_str())) {
        return Err("unknown_provenance_field");
    }
    let kind = SourceKind::parse(&input["kind"])?;
    let author = label(input, "author")?;
    if !author.chars().any(char::is_alphabetic) {
        return Err("invalid_provenance_label");
    }
    let channel = label(input, "channel")?;
    let timezone = input["display_timezone"]
        .as_str()
        .filter(|name| !name.is_empty() && name.len() <= 128)
        .ok_or("invalid_provenance_timezone")?;
    let timezone = TimeZone::get(timezone).map_err(|_| "invalid_provenance_timezone")?;
    if timezone.is_unknown() {
        return Err("invalid_provenance_timezone");
    }
    let (day, time) = match input.get("timestamp") {
        Some(Value::String(value)) if input.get("day").is_none() => {
            let local = from_rfc3339(value)?.to_zoned(timezone);
            let day = local.date().to_string();
            if !crate::iso_date(&day) {
                return Err("invalid_provenance_day");
            }
            (day, format!("{:02}:{:02}", local.hour(), local.minute()))
        }
        Some(Value::Null) => (
            input["day"]
                .as_str()
                .filter(|day| crate::iso_date(day))
                .ok_or("invalid_provenance_day")?
                .to_owned(),
            "—".into(),
        ),
        _ => return Err("invalid_provenance_timestamp_or_day"),
    };
    Ok(RenderedProvenance {
        kind,
        day,
        author,
        stamp: format!("{time} · {author} · {channel}"),
    })
}

/// JSON boundary for external native adapters. Never discovers source or time.
pub fn respond(input: &[u8]) -> Value {
    let result = (|| {
        if input.len() > MAX_REQUEST_BYTES {
            return Err("provenance_request_limit");
        }
        let v: Value = serde_json::from_slice(input).map_err(|_| "invalid_provenance_json")?;
        let options = serde_saphyr::options! {
            duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
            merge_keys: serde_saphyr::MergeKeyPolicy::Error
        };
        let _: Value = serde_saphyr::from_str_with_options(
            std::str::from_utf8(input).map_err(|_| "invalid_provenance_json")?,
            options,
        )
        .map_err(|_| "duplicate_or_invalid_provenance_json")?;
        if v.as_object().is_none_or(|o| o.len() != 2)
            || v["schema"] != "viggobsidian.activity-provenance.v1"
        {
            return Err("invalid_provenance_schema");
        }
        let rendered = render(&v["provenance"])?;
        Ok(
            json!({"schema":"viggobsidian.activity-provenance-result.v1", "status":"complete",
            "day":rendered.day, "author":rendered.author, "footer":rendered.footer()}),
        )
    })();
    result.unwrap_or_else(|reason| {
        json!({"schema":"viggobsidian.activity-provenance-result.v1","status":"incomplete","reason":reason})
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: &str, timestamp: &str, author: &str, channel: &str) -> Value {
        json!({"kind":kind, "timestamp":timestamp, "author":author, "channel":channel,
            "display_timezone":"Europe/Amsterdam"})
    }

    #[test]
    fn source_offset_and_display_timezone_determine_both_minute_and_day() {
        let chat = event(
            "chat-message",
            "2026-07-08T15:09:17-07:00",
            "A. Example",
            "Chat",
        );
        let rendered = render(&chat).unwrap();
        assert_eq!(rendered.kind, SourceKind::ChatMessage);
        assert_eq!(rendered.day, "2026-07-09");
        assert_eq!(rendered.footer(), ">\n> *00:09 · A. Example · Chat*\n");

        let launcher = event(
            "launcher-action",
            "2026-11-08T22:09:00Z",
            "B. Example",
            "Launcher",
        );
        assert_eq!(render(&launcher).unwrap().day, "2026-11-08");
        assert_eq!(
            render(&launcher).unwrap().stamp,
            "23:09 · B. Example · Launcher"
        );

        let mut contribution = event(
            "vault-contribution",
            "2026-11-08T02:09:00+01:00",
            "A. Agent",
            "Vault",
        );
        contribution["display_timezone"] = json!("America/Chicago");
        assert_eq!(render(&contribution).unwrap().day, "2026-11-07");
        assert_eq!(
            render(&contribution).unwrap().stamp,
            "19:09 · A. Agent · Vault"
        );
    }

    #[test]
    fn unknown_requires_explicit_null_and_an_independently_known_valid_day() {
        let mut unknown = event("chat-message", "", "A. Example", "Chat");
        unknown["timestamp"] = Value::Null;
        assert!(render(&unknown).is_err());
        unknown["day"] = json!("2026-02-29");
        assert!(render(&unknown).is_err());
        unknown["day"] = json!("2024-02-29");
        assert_eq!(render(&unknown).unwrap().stamp, "— · A. Example · Chat");
        unknown.as_object_mut().unwrap().remove("timestamp");
        assert!(render(&unknown).is_err());
    }

    #[test]
    fn rejects_ambiguous_timestamp_payload_and_markup() {
        let good = event("chat-message", "2026-11-08T22:09:00Z", "A. Example", "Chat");
        for (field, bad) in [
            ("timestamp", "2026-11-08T22:09:00"),
            ("timestamp", "2026-11-08T22:09:00+24:00"),
            ("timestamp", "2026-11-08T22:09:00+01:60"),
            ("timestamp", "2026-11-08T22:09:60Z"),
            ("timestamp", "2026-11-08T22:09Z"),
            ("timestamp", "2026-11-08T22:09:00Z[UTC]"),
            ("timestamp", "2026-11-31T22:09:00Z"),
            ("timestamp", "9999-12-31T23:59:59Z"),
            ("display_timezone", "Hologram/Example"),
            ("display_timezone", "Etc/Unknown"),
            ("author", "A · B"),
            ("channel", "Chat\n> additional line"),
            ("channel", "*Chat*"),
            ("kind", "file-mtime"),
        ] {
            let mut bad_event = good.clone();
            bad_event[field] = json!(bad);
            assert!(render(&bad_event).is_err(), "{field}: {bad}");
        }
        let mut independent_day = good;
        independent_day["day"] = json!("2026-11-08");
        assert!(render(&independent_day).is_err());
    }

    #[test]
    fn bounded_adapter_protocol_replays_and_rejects_duplicate_facts() {
        let request = json!({
            "schema":"viggobsidian.activity-provenance.v1",
            "provenance":event("chat-message", "2026-11-08T22:09:00Z", "A. Example", "Chat")
        });
        let bytes = serde_json::to_vec(&request).unwrap();
        let response = respond(&bytes);
        assert_eq!(response["status"], "complete");
        assert_eq!(response["day"], "2026-11-08");
        assert_eq!(respond(&bytes), response);
        let duplicate = request
            .to_string()
            .replace("\"channel\":", "\"channel\":\"Other\",\"channel\":");
        assert_eq!(respond(duplicate.as_bytes())["status"], "incomplete");
        assert_eq!(
            respond(&vec![b' '; MAX_REQUEST_BYTES + 1])["status"],
            "incomplete"
        );
    }
}
