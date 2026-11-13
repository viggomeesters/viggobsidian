use serde_json::{Value, json};

pub const MAX_NOTE_BYTES: usize = 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;
const RESULT: &str = "viggobsidian.wiki-layout-result.v1";

fn evaluate(input: &Value) -> Result<Value, &'static str> {
    if input["schema"] != "viggobsidian.wiki-layout-snapshot.v1" {
        return Err("unsupported_schema");
    }
    if input["ruleset"] != "wiki-layout.v1" {
        return Err("unsupported_ruleset");
    }
    if input["complete"] != true {
        return Err("complete_snapshot_required");
    }
    let profile = input["profile"].as_str().ok_or("profile_required")?;
    let classification = input["classification"]
        .as_str()
        .ok_or("classification_required")?;
    if profile != "numbered-v1" || classification != "wiki" {
        return Ok(
            json!({"schema":RESULT,"status":"not_applicable","reason":"profile_or_classification_not_selected"}),
        );
    }
    let path = input["path"].as_str().ok_or("path_required")?;
    if !path.ends_with(".md")
        || path.contains(['\\', ':', '\0'])
        || path
            .split('/')
            .any(|part| part.is_empty() || part.starts_with('.'))
    {
        return Err("unsafe_note_path");
    }
    let body = input["body"].as_str().ok_or("body_required")?;
    if body.len() > MAX_NOTE_BYTES {
        return Err("content_limit");
    }
    let (has_facts, findings) = crate::findings(body);
    let findings = findings
        .into_iter()
        .map(|(code, message)| json!({"path":path,"code":code,"message":message}))
        .collect::<Vec<_>>();
    Ok(
        json!({"schema":RESULT,"status":"complete","report":{"path":path,"has_facts":has_facts,"findings":findings}}),
    )
}

/// The adapter supplies body text after metadata and explicit classification.
/// Completion only attests to this text slice; it never proves filesystem facts.
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
