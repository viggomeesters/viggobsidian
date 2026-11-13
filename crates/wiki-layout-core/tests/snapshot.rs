use serde_json::{Value, json};
fn request() -> Value {
    json!({"schema":"viggobsidian.wiki-layout-snapshot.v1","ruleset":"wiki-layout.v1","complete":true,"profile":"numbered-v1","classification":"wiki","path":"d/wiki/wiki.md","body":""})
}
fn respond(value: &Value) -> Value {
    wiki_layout_core::respond(&serde_json::to_vec(value).unwrap())
}

#[test]
fn snapshot_matches_frozen_legacy_and_never_invents_locations() {
    let corpus: Value =
        serde_json::from_str(include_str!("../fixtures/wiki-layout-legacy.json")).unwrap();
    for case in corpus["cases"].as_array().unwrap() {
        let mut input = request();
        input["body"] = case["body"].clone();
        let output = respond(&input);
        assert_eq!(output["status"], "complete");
        assert_eq!(output["report"]["has_facts"], case["has_facts"]);
        let expected = case["findings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| json!({"path":"d/wiki/wiki.md","code":i["code"],"message":i["message"]}))
            .collect::<Vec<_>>();
        assert_eq!(
            output["report"]["findings"],
            json!(expected),
            "{}",
            case["id"]
        );
    }
}

#[test]
fn unsupported_missing_and_bounded_input_cannot_be_clean() {
    for (field, value) in [
        ("schema", json!("future")),
        ("ruleset", json!("future")),
        ("complete", json!(false)),
        ("profile", Value::Null),
        ("classification", Value::Null),
        ("path", json!("../wiki.md")),
        ("body", Value::Null),
        (
            "body",
            json!("x".repeat(wiki_layout_core::MAX_NOTE_BYTES + 1)),
        ),
    ] {
        let mut input = request();
        input[field] = value;
        let output = respond(&input);
        assert_eq!(output["status"], "incomplete", "{field}");
        assert!(output.get("report").is_none());
    }
    for (field, value) in [("profile", "none"), ("classification", "note")] {
        let mut input = request();
        input[field] = json!(value);
        let output = respond(&input);
        assert_eq!(output["status"], "not_applicable");
        assert!(output.get("report").is_none());
    }
    for input in [
        vec![b'x'; wiki_layout_core::MAX_REQUEST_BYTES + 1],
        b"{malformed".to_vec(),
        vec![0xff],
    ] {
        assert_eq!(wiki_layout_core::respond(&input)["status"], "incomplete");
    }
}
