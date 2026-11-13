use serde_json::{Value, json};

pub const MAX_NOTE_BYTES: usize = 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;
const RESULT: &str = "viggobsidian.wiki-layout-result.v1";

fn evaluate(input: &Value) -> Result<Value, &'static str> {
    let portable = input["schema"] == "viggobsidian.wiki-layout-snapshot.v2";
    if !portable && input["schema"] != "viggobsidian.wiki-layout-snapshot.v1" {
        return Err("unsupported_schema");
    }
    if input["ruleset"] != "wiki-layout.v1" {
        return Err("unsupported_ruleset");
    }
    if input["complete"] != true {
        return Err("complete_snapshot_required");
    }
    let profile = input["profile"].as_str().ok_or("profile_required")?;
    let classification = if portable {
        "wiki"
    } else {
        input["classification"]
            .as_str()
            .ok_or("classification_required")?
    };
    if !portable && (profile != "numbered-v1" || classification != "wiki") {
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
    let classes = if portable {
        let classes = input["cssclasses"]
            .as_array()
            .ok_or("cssclasses_required")?;
        if !classes.iter().all(Value::is_string) {
            return Err("invalid_cssclasses");
        }
        Some(classes)
    } else {
        None
    };
    let body = input["body"].as_str().ok_or("body_required")?;
    if body.len() > MAX_NOTE_BYTES {
        return Err("content_limit");
    }
    if let Some(classes) = classes
        && (profile != "numbered-v1"
            || !crate::is_wiki_layout_target(path, classes.iter().filter_map(Value::as_str)))
    {
        return Ok(
            json!({"schema":RESULT,"status":"not_applicable","reason":"profile_or_classification_not_selected"}),
        );
    }
    // The native frontmatter adapter uses str::lines().join("\n") too.
    let normalized;
    let body = if portable {
        normalized = body.lines().collect::<Vec<_>>().join("\n");
        &normalized
    } else {
        body
    };
    let body = if portable {
        crate::without_knowledge_owner_notice(body)
    } else {
        body
    };
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

#[cfg(test)]
mod portable_tests {
    use super::*;
    #[test]
    fn canonical_selection_and_v1_compatibility() {
        for (path, classes, selected) in [
            ("d/wiki/wiki.md", vec![], true),
            ("d/wiki/nested/wiki.md", vec![], true),
            ("d/wiki/note.md", vec![], false),
            ("d/d.md", vec!["vault-wiki-direct"], true),
            ("d/wiki.md", vec![], false),
        ] {
            let mut request = json!({"schema":"viggobsidian.wiki-layout-snapshot.v2","ruleset":"wiki-layout.v1","complete":true,"profile":"numbered-v1","path":path,"cssclasses":classes,"body":"# Example\r\n"});
            let response = respond(&serde_json::to_vec(&request).unwrap());
            assert_eq!(
                response["status"],
                if selected {
                    "complete"
                } else {
                    "not_applicable"
                }
            );
            if selected {
                request["schema"] = json!("viggobsidian.wiki-layout-snapshot.v1");
                request["classification"] = json!("wiki");
                request["body"] = json!("# Example");
                assert_eq!(response, respond(&serde_json::to_vec(&request).unwrap()));
            }
        }
    }
    #[test]
    fn portable_notice_preprocessing_matches_native() {
        let notice = "<!-- viggobsidian:knowledge-owner -->\n> [!info] Actuele kennis\n> [[d/wiki/wiki|Open actuele kennis]]\n<!-- /viggobsidian:knowledge-owner -->\n";
        let body = "# Wiki\n\n> [!navigation|shortcuts]\n> - [[d|Back]]\n> - [[d/raw/raw|Raw]]\n> - [Finder](file:///fiction/d/wiki)\n\n";
        for (prefix, clean) in [
            (notice.to_owned(), true),
            (notice.replace("Actuele kennis", "Other"), false),
        ] {
            let input = json!({"schema":"viggobsidian.wiki-layout-snapshot.v2","ruleset":"wiki-layout.v1","complete":true,"profile":"numbered-v1","path":"d/wiki/wiki.md","cssclasses":[],"body":format!("{prefix}{body}")});
            let response = respond(&serde_json::to_vec(&input).unwrap());
            assert_eq!(
                response["report"]["findings"]
                    .as_array()
                    .unwrap()
                    .is_empty(),
                clean
            );
        }
    }
    #[test]
    fn malformed_portable_inputs_cannot_be_clean() {
        let base = json!({"schema":"viggobsidian.wiki-layout-snapshot.v2","ruleset":"wiki-layout.v1","complete":true,"profile":"numbered-v1","path":"d/wiki/wiki.md","cssclasses":[],"body":""});
        for (key, value) in [
            ("cssclasses", json!("vault-wiki-direct")),
            ("cssclasses", json!([1])),
            ("path", json!("../wiki.md")),
            ("body", json!(null)),
        ] {
            let mut request = base.clone();
            request[key] = value;
            assert_eq!(
                respond(&serde_json::to_vec(&request).unwrap())["status"],
                "incomplete"
            );
        }
    }
}
