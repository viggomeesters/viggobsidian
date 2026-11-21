//! Pure status classification and Work intent and Activity preparation.

use serde_json::{Value, json};

const STATUSES: [(&str, &str); 6] = [
    ("backlog", "🟣 backlog"),
    ("todo", "🔴 to-do"),
    ("in-progress", "🟠 in progress"),
    ("waiting", "🔵 waiting"),
    ("done", "🟢 done"),
    ("cancelled", "⚫ cancelled"),
];

/// Resolve the existing launcher key to the stored Markdown status label.
pub fn status_value(action: &str) -> Result<&'static str, &'static str> {
    STATUSES
        .iter()
        .find_map(|&(key, label)| (key == action).then_some(label))
        .ok_or("Onbekende taakstatus")
}

/// Recognize a stored task label, as opposed to a launcher key.
pub fn is_status_label(value: &str) -> bool {
    STATUSES.iter().any(|&(_, label)| label == value)
}

fn validate_reason(reason: &str) -> Result<(), &'static str> {
    if reason.len() > 65_536 || reason.contains('\0') {
        return Err("De reden is te lang of bevat een ongeldig teken");
    }
    Ok(())
}

/// Encode the Work intent. Existing receipts bind these exact bytes, including
/// field order and the unnormalized reason.
pub fn status_input(status: &str, reason: &str) -> Result<String, &'static str> {
    status_value(status)?;
    validate_reason(reason)?;
    Ok(format!(
        r#"{{"reason":{},"status":{}}}"#,
        json!(reason),
        json!(status)
    ))
}

/// Read a bare status key or the existing Work intent object.
pub fn status_intent(input: &str) -> Result<(&'static str, String), &'static str> {
    if !input.starts_with('{') {
        return Ok((status_value(input)?, String::new()));
    }
    let value: Value = serde_json::from_str(input).map_err(|_| "Ongeldige statusinvoer")?;
    let (Some(2), Some(status), Some(reason)) = (
        value.as_object().map(|fields| fields.len()),
        value["status"].as_str(),
        value["reason"].as_str(),
    ) else {
        return Err("Statusinvoer vereist uitsluitend status en reden");
    };
    let label = status_value(status)?;
    validate_reason(reason)?;
    Ok((label, reason.to_owned()))
}

/// Escape literal human text for the existing Work Activity renderer.
pub fn literal_text(text: &str) -> String {
    let mut escaped = String::new();
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '\\' | '`' | '*' | '_' | '[' | ']' | '#' | '|' => {
                escaped.push('\\');
                escaped.push(character);
            }
            _ => escaped.push(character),
        }
    }
    escaped
}

/// Render the Work reason beneath its transition. The existing writer supplies
/// the transition, Activity heading, date, author and footer.
pub fn status_message(transition: &str, reason: &str) -> String {
    if reason.is_empty() {
        return transition.to_owned();
    }
    let reason = reason.replace("\r\n", "\n").replace('\r', "\n");
    let reason = literal_text(&reason);
    if reason.len() > 1200 || reason.lines().count() > 30 {
        format!(
            "{transition}\n\n> [!activity-more]- Meer\n> {}",
            reason.replace('\n', "\n> ")
        )
    } else {
        format!("{transition}\n\n{reason}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_status_keys_and_labels_are_exact() {
        for (key, label) in [
            ("backlog", "🟣 backlog"),
            ("todo", "🔴 to-do"),
            ("in-progress", "🟠 in progress"),
            ("waiting", "🔵 waiting"),
            ("done", "🟢 done"),
            ("cancelled", "⚫ cancelled"),
        ] {
            assert_eq!(status_intent(key), Ok((label, String::new())));
            assert!(is_status_label(label));
            assert!(!is_status_label(key));
        }
        assert_eq!(status_value("Done"), Err("Onbekende taakstatus"));
        assert!(!is_status_label("🟢 done "));
    }

    #[test]
    fn work_wire_keeps_original_reason_and_receipt_bytes() {
        let reason = "Eerste regel 🌍\r\n\"tweede\"\\pad\r";
        let old_wire = r#"{"reason":"Eerste regel 🌍\r\n\"tweede\"\\pad\r","status":"done"}"#;
        assert_eq!(status_input("done", reason).unwrap(), old_wire);
        assert_eq!(status_intent(old_wire), Ok(("🟢 done", reason.to_owned())));
        assert_eq!(
            status_input("todo", "").unwrap(),
            r#"{"reason":"","status":"todo"}"#
        );
        assert_eq!(
            status_intent(r#"{"status":"waiting","status":"done","reason":""}"#),
            Ok(("🟢 done", String::new()))
        );
    }

    #[test]
    fn refuses_invalid_intents_at_the_existing_boundaries() {
        for value in [
            r#"{"status":"done","reason":"x","actor":"Other"}"#,
            r#"{"status":"done","reason":null}"#,
            r#"{"reason":"x"}"#,
        ] {
            assert_eq!(
                status_intent(value),
                Err("Statusinvoer vereist uitsluitend status en reden")
            );
        }
        assert_eq!(status_intent("{"), Err("Ongeldige statusinvoer"));
        assert_eq!(status_intent(" {}"), Err("Onbekende taakstatus"));
        assert!(status_input("done", &"é".repeat(32_768)).is_ok());
        for reason in ["é".repeat(32_769), "\0".to_owned()] {
            assert_eq!(
                status_input("done", &reason),
                Err("De reden is te lang of bevat een ongeldig teken")
            );
        }
    }

    #[test]
    fn work_activity_escapes_then_folds_at_the_existing_threshold() {
        assert_eq!(status_message("Overgang", ""), "Overgang");
        assert_eq!(
            status_message("Overgang", "# Kop\r\n> [!note]\r<!-- tekst --> &"),
            "Overgang\n\n\\# Kop\n&gt; \\[!note\\]\n&lt;!-- tekst --&gt; &amp;"
        );
        assert!(!status_message("Overgang", &"&".repeat(240)).contains("[!activity-more]"));
        assert!(status_message("Overgang", &"&".repeat(241)).contains("[!activity-more]"));
        assert!(!status_message("Overgang", &"x\n".repeat(30)).contains("[!activity-more]"));
        assert!(status_message("Overgang", &"x\n".repeat(31)).contains("[!activity-more]"));
    }
}
