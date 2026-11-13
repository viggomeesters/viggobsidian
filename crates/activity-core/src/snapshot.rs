use serde_json::{Value, json};
pub const MAX_NOTE_BYTES: usize = 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;
const RESULT: &str = "viggobsidian.activity-result.v1";

fn evaluate(input: &Value) -> Result<Value, &'static str> {
    if input["schema"] != "viggobsidian.activity-snapshot.v1" {
        return Err("unsupported_schema");
    }
    if input["ruleset"] != "activity-text.v1" {
        return Err("unsupported_ruleset");
    }
    if input["complete"] != true {
        return Err("complete_snapshot_required");
    }
    let profile = input["profile"].as_str().ok_or("profile_required")?;
    let path = input["path"].as_str().ok_or("path_required")?;
    if !path.ends_with(".md")
        || path.contains(['\\', ':', '\0'])
        || path
            .split('/')
            .any(|part| part.is_empty() || part.starts_with('.'))
    {
        return Err("unsafe_note_path");
    }
    let frontmatter = input["frontmatter_present"]
        .as_bool()
        .ok_or("frontmatter_presence_required")?;
    let metadata = input["metadata"]
        .as_object()
        .ok_or("metadata_mapping_required")?;
    let classes = match metadata.get("cssclasses") {
        None => vec![],
        Some(Value::Array(values)) if values.iter().all(Value::is_string) => {
            values.iter().filter_map(Value::as_str).collect()
        }
        _ => return Err("invalid_cssclasses"),
    };
    let body = input["body"].as_str().ok_or("body_required")?;
    if body.len() > MAX_NOTE_BYTES {
        return Err("content_limit");
    }
    if profile != "numbered-v1" {
        return Ok(
            json!({"schema":RESULT,"status":"not_applicable","reason":"profile_not_selected"}),
        );
    }
    if !frontmatter {
        return Err("frontmatter_required");
    }
    let body = body.lines().collect::<Vec<_>>().join("\n");
    let mut findings = Vec::new();
    let mut add = |code: &str, messages: Vec<String>| {
        for message in messages {
            findings.push(json!({"path":path,"code":code,"message":message}));
        }
    };
    add("strict-mail-draft", crate::mail_draft::findings(&body));
    add(
        "strict-activity-delivery",
        crate::activity_delivery::findings(&body),
    );
    let contract = metadata.get("activity_contract");
    if let Some(contract) = contract {
        if contract.as_str() != Some("provenance-v1") {
            add(
                "strict-activity-provenance",
                vec!["unknown activity_contract".into()],
            );
        } else {
            add(
                "strict-activity-provenance",
                crate::activity_delivery::provenance_findings(&body),
            );
        }
    }
    let chat = metadata.get("activity_format").and_then(Value::as_str) == Some("chat-v1");
    if chat {
        add("activity-message", crate::activity_layout::findings(&body));
    }
    let v11 = classes.contains(&"vault-event-v1-1");
    if v11 && let Err(message) = crate::activity_layout::check_v11(&body) {
        add("event-activity-layout", vec![message.into()]);
    }
    // Marker-free historical notes are deliberately not promoted to provenance-v1.
    let applicable = chat
        || contract.is_some()
        || v11
        || !findings.is_empty()
        || crate::activity_delivery::has_live_markers(&body);
    if !applicable {
        return Ok(
            json!({"schema":RESULT,"status":"not_applicable","reason":"activity_contract_not_selected"}),
        );
    }
    if findings.len() > 100 {
        return Err("finding_limit");
    }
    Ok(
        json!({"schema":RESULT,"status":"complete","capabilities":["activity-text.v1","provenance-v1","chat-v1","event-activity-v1.1","mail-delivery-v1"],"report":{"path":path,"findings":findings}}),
    )
}

pub fn respond(bytes: &[u8]) -> Value {
    let result = if bytes.len() > MAX_REQUEST_BYTES {
        Err("request_limit")
    } else {
        serde_json::from_slice(bytes)
            .map_err(|_| "invalid_json")
            .and_then(|input| evaluate(&input))
    };
    result.unwrap_or_else(|reason| json!({"schema":RESULT,"status":"incomplete","reason":reason}))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> Value {
        json!({"schema":"viggobsidian.activity-snapshot.v1","ruleset":"activity-text.v1","complete":true,"profile":"numbered-v1","path":"d/raw/event/event.md","frontmatter_present":true,"metadata":{"activity_contract":"provenance-v1"},"body":"# Event"})
    }
    fn call(value: &Value) -> Value {
        respond(&serde_json::to_vec(value).unwrap())
    }
    #[test]
    fn malformed_and_partial_snapshots_are_never_clean() {
        for key in [
            "schema",
            "ruleset",
            "complete",
            "profile",
            "path",
            "frontmatter_present",
            "metadata",
            "body",
        ] {
            let mut input = base();
            input.as_object_mut().unwrap().remove(key);
            assert_eq!(call(&input)["status"], "incomplete", "{key}");
        }
        for (key, value) in [
            ("schema", json!("future")),
            ("ruleset", json!("future")),
            ("complete", json!(false)),
            ("frontmatter_present", json!(false)),
            ("metadata", json!([])),
            ("metadata", json!({"cssclasses":[1]})),
            ("body", json!(null)),
            ("body", json!("x".repeat(MAX_NOTE_BYTES + 1))),
            ("path", json!("../private.md")),
        ] {
            let mut input = base();
            input[key] = value;
            assert_eq!(call(&input)["status"], "incomplete", "{key}");
        }
        for bytes in [
            b"{broken".as_slice(),
            &[255],
            &vec![b' '; MAX_REQUEST_BYTES + 1],
        ] {
            assert_eq!(respond(bytes)["status"], "incomplete");
        }
    }
    #[test]
    fn legacy_optins_visible_markers_and_unknown_contract_preserve_native_semantics() {
        let mut input = base();
        input["metadata"] = json!({});
        assert_eq!(call(&input)["status"], "not_applicable");
        input["body"] =
            json!("```md\n> <!-- viggobsidian:mail-delivery:v1 id=x version=1 -->\n```\n");
        assert_eq!(call(&input)["status"], "not_applicable");
        input["metadata"] = json!({"activity_contract":null});
        assert_eq!(
            call(&input)["report"]["findings"][0]["message"],
            "unknown activity_contract"
        );
        input["profile"] = json!("none");
        assert_eq!(call(&input)["status"], "not_applicable");
        input["profile"] = json!("numbered-v1");
        input["metadata"] = json!({"activity_format":"chat-v1"});
        assert_eq!(
            call(&input)["report"]["findings"][0]["code"],
            "activity-message"
        );
        input["metadata"] = json!({"cssclasses":["vault-event-v1-1"]});
        assert_eq!(
            call(&input)["report"]["findings"][0]["code"],
            "event-activity-layout"
        );
    }
    #[test]
    fn leading_canonical_knowledge_notice_cannot_change_activity_outputs() {
        let corpus: Value =
            serde_json::from_str(include_str!("../fixtures/activity-legacy.json")).unwrap();
        let notice = "<!-- viggobsidian:knowledge-owner -->\n> [!info] Actuele kennis\n> [[d/wiki/wiki|Open actuele kennis]]\n<!-- /viggobsidian:knowledge-owner -->\n";
        for case in corpus["cases"].as_array().unwrap() {
            let mut input = base();
            input["body"] = case["body"].clone();
            let original = call(&input);
            input["body"] = json!(format!("{notice}{}", case["body"].as_str().unwrap()));
            assert_eq!(call(&input), original, "{}", case["id"]);
        }
    }
}
