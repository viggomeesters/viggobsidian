use serde_json::{Value, json};
use source_preparation_core::{
    AUTHORED_ACTIVITY_FORMAT, FORMAT, MAX_NOTE_BYTES, MAX_REQUEST_BYTES, PROVENANCE_FORMAT, hash,
    respond,
};
use std::io::Write;
use std::process::{Command, Stdio};

const ORIGINAL: &str = "---\ntype: note\ncategory: source\ncreated: 2026-11-02\nsource_identity: fixture:original\n---\n\n# Original source\n\n## Original request\n\nKeep these exact words: café → knowledge.\n\n> [!note] Original callout\n> Unchanged.\n\n```text\nAn original fence.\n```\n";

const AUTHORED: &str = "---\ntype: note\ncategory: general\ncreated: 2026-11-02\nactivity_contract: provenance-v1\nsource_identity: 'fixture:authored'\nsource_timestamp: '2026-11-02T09:02:31+01:00'\n---\n\n# Authored source\n\nOriginal context with a [diagram](diagram.png).\n\n## Activity\n\n> [!activity-day|2026-11-02] 2 November\n\n> [!activity-message|human] Alex Example\n> `09:02 · Alex Example · Chat`\n> ## Original heading\n> Keep the original words and [diagram](diagram.png).  \n\n> [!activity-message|assistant] Fictional Assistant\n> <!-- viggobsidian:activity:v1 kind=result goal=source-review -->\n> The fictional source was read.\n>\n> *09:11 · Fictional Assistant · Chat*";

fn request() -> Value {
    json!({"schema":"viggobsidian.source-preparation.v1", "format":FORMAT, "operation":"prepare",
        "date":"2026-11-13", "source_note":ORIGINAL, "source_path":"00-inbox/20261102-source/20261102-source.md",
        "destination_owner":"30-topics/example/example.md", "vault_root":"/fictional/vault"})
}

fn run(v: &Value) -> Value {
    respond(&serde_json::to_vec(v).unwrap())
}

fn authored_request() -> Value {
    let mut v = request();
    v["format"] = json!(AUTHORED_ACTIVITY_FORMAT);
    v["source_note"] = json!(AUTHORED);
    v["provenance"] = json!({
        "kind":"vault-contribution", "timestamp":"2026-11-13T09:00:14+01:00",
        "display_timezone":"Europe/Amsterdam", "author":"Fictional Assistant", "channel":"Vault"
    });
    v
}

#[test]
fn authored_activity_and_metadata_are_preserved_and_marking_replays_exactly() {
    let prepared_request = authored_request();
    let prepared = run(&prepared_request);
    assert_eq!(prepared["status"], "complete", "{prepared}");
    assert_eq!(run(&prepared_request), prepared);
    let postimage = prepared["postimage"].as_str().unwrap();
    let original_header = AUTHORED.split_once("\n---\n").unwrap().0;
    let original_activity = AUTHORED.split_once("\n## Activity\n").unwrap().1;
    assert!(postimage.starts_with(original_header));
    assert!(
        postimage.contains(
            "> [!event-context] Context\n> Original context with a [diagram](diagram.png)."
        )
    );
    assert_eq!(
        postimage.split_once("\n## Activity\n").unwrap().1,
        original_activity
    );
    assert_eq!(prepared["original_sha256"], hash(AUTHORED));
    assert_eq!(prepared["before_sha256"], hash(AUTHORED));
    assert_eq!(prepared["after_sha256"], hash(postimage));

    let mut processing = marking();
    processing["format"] = json!(AUTHORED_ACTIVITY_FORMAT);
    processing["source_note"] = prepared["postimage"].clone();
    processing["original_note"] = json!(AUTHORED);
    processing["preparation_provenance"] = prepared_request["provenance"].clone();
    processing["provenance"] = prepared_request["provenance"].clone();
    processing["provenance"]["timestamp"] = json!("2026-11-13T09:01:16+01:00");
    let processed = run(&processing);
    assert_eq!(processed["status"], "complete", "{processed}");
    assert_eq!(run(&processing), processed);
    let processed_image = processed["postimage"].as_str().unwrap();
    assert!(processed_image.starts_with(original_header));
    assert_eq!(
        processed_image
            .matches("activity_contract: provenance-v1")
            .count(),
        1
    );
    assert!(processed_image.contains("\nwiki_status: processed\n"));
    assert!(processed_image.contains(&format!(
        "{original_activity}\n\n> [!activity-day|2026-11-13]"
    )));
    assert!(processed_image.contains("> *09:01 · Fictional Assistant · Vault*"));
    assert_eq!(processed["original_sha256"], hash(AUTHORED));
    assert_eq!(processed["before_sha256"], hash(postimage));
    assert_eq!(processed["after_sha256"], hash(processed_image));

    for (field, changed) in [
        ("source_note", postimage.replace("words and", "words or")),
        ("original_note", AUTHORED.replace("09:02:31", "09:02:32")),
        (
            "original_note",
            AUTHORED.replace("\n\n# Authored", "\n\n\n# Authored"),
        ),
    ] {
        let mut bad = processing.clone();
        bad[field] = json!(changed);
        assert_eq!(run(&bad)["reason"], "prepared_postimage_changed");
    }
    processing["preparation_provenance"]["timestamp"] = json!("2026-11-13T09:00:15+01:00");
    assert_eq!(run(&processing)["reason"], "prepared_postimage_changed");
}

#[test]
fn authored_activity_refuses_ambiguous_or_unrelated_sources() {
    for (before, after, reason) in [
        (
            "category: general",
            "category: source",
            "authored_activity_note_required",
        ),
        (
            "type: note",
            "type: task",
            "authored_activity_note_required",
        ),
        (
            "type: note",
            "type: note\nlayer: wiki",
            "authored_activity_note_required",
        ),
        (
            "activity_contract: provenance-v1",
            "activity_contract: unknown",
            "authored_activity_note_required",
        ),
        (
            "type: note",
            "type: note\nwiki_status: pending",
            "unprepared_source_required",
        ),
        ("## Activity", "## Comments", "authored_activity_required"),
        (
            "\n\n## Activity",
            "\n\n## Unmapped source\n\nAnother section.\n\n## Activity",
            "authored_activity_title_and_context_required",
        ),
        (
            "|2026-11-02]",
            "|2026-11-14]",
            "authored_activity_after_preparation_date",
        ),
        (
            "09:02 · Alex Example",
            "09:02 · Other Person",
            "invalid_authored_activity_provenance",
        ),
        (
            "The fictional source was read.",
            "The fictional source was read.\nUnwrapped text",
            "invalid_authored_activity_provenance",
        ),
    ] {
        let mut v = authored_request();
        v["source_note"] = json!(AUTHORED.replace(before, after));
        let out = run(&v);
        assert_eq!(out["reason"], reason, "{before}: {out}");
        assert!(out.get("postimage").is_none());
    }
    let mut v = authored_request();
    v["format"] = json!(PROVENANCE_FORMAT);
    assert_eq!(run(&v)["reason"], "plain_note_source_required");
    v["format"] = json!(AUTHORED_ACTIVITY_FORMAT);
    v.as_object_mut().unwrap().remove("provenance");
    assert_eq!(run(&v)["reason"], "vault_contribution_provenance_required");
}

#[test]
fn fictional_prepare_matches_frozen_postimage_and_preserves_original() {
    let result = run(&request());
    assert_eq!(result["status"], "complete", "{result}");
    assert_eq!(
        result["postimage"],
        serde_json::from_str::<String>(include_str!(
            "../../vault-layout/fixtures/source-prepared-v1.json"
        ))
        .unwrap()
    );
    assert_eq!(result["before_sha256"], hash(ORIGINAL));
    assert_eq!(result["original_sha256"], hash(ORIGINAL));
    let quoted = result["postimage"]
        .as_str()
        .unwrap()
        .split_once("> > [!info]- Meer\n")
        .unwrap()
        .1;
    let restored = quoted
        .split_inclusive('\n')
        .map(|line| line.strip_prefix("> > ").unwrap())
        .collect::<String>();
    assert_eq!(restored, ORIGINAL.split_once("\n---\n").unwrap().1);
    assert_eq!(run(&request()), result);
}

fn marking() -> Value {
    let mut v = request();
    v["operation"] = json!("mark-wiki-processed");
    v["source_path"] = json!("30-topics/example/raw/20261102-source/20261102-source.md");
    v["source_note"] = run(&request())["postimage"].clone();
    v["original_note"] = json!(ORIGINAL);
    v["preparation_date"] = json!("2026-11-13");
    v["wiki_target"] = json!("[[30-topics/example/wiki/wiki#Existing knowledge]]");
    v["summary"] = json!("The original source was compared with the existing knowledge.");
    v
}

#[test]
fn marking_is_exact_and_rejects_changed_original_or_native_header() {
    let v = marking();
    let out = run(&v);
    assert_eq!(out["status"], "complete", "{out}");
    assert_eq!(
        out["postimage"],
        serde_json::from_str::<String>(include_str!(
            "../../vault-layout/fixtures/source-processed-v1.json"
        ))
        .unwrap()
    );
    for (field, value) in [
        (
            "source_note",
            v["source_note"]
                .as_str()
                .unwrap()
                .replace("# Original source", "# Changed"),
        ),
        (
            "original_note",
            ORIGINAL.replace("exact words", "different words"),
        ),
        ("wiki_target", "[[another/wiki/wiki#Other owner]]".into()),
        ("summary", "Summary\n> Injected message".into()),
        ("preparation_date", "2026-11-14".into()),
    ] {
        let mut bad = v.clone();
        bad[field] = json!(value);
        assert_eq!(run(&bad)["status"], "incomplete", "{field}");
    }
}

#[test]
fn contribution_provenance_stays_outside_details_and_is_bound_to_exact_preimage() {
    let provenance = json!({
        "kind": "vault-contribution",
        "timestamp": "2026-11-12T23:43:12Z",
        "display_timezone": "Europe/Amsterdam",
        "author": "Fictional Assistant",
        "channel": "Vault processing"
    });
    let mut request = request();
    request["format"] = json!(PROVENANCE_FORMAT);
    assert_eq!(
        run(&request)["reason"],
        "vault_contribution_provenance_required"
    );
    request["provenance"] = provenance.clone();
    let prepared = run(&request);
    assert_eq!(prepared["status"], "complete", "{prepared}");
    let postimage = prepared["postimage"].as_str().unwrap();
    assert!(postimage.contains(
        "> > An original fence.\n> > ```\n>\n> *00:43 · Fictional Assistant · Vault processing*\n"
    ));
    assert!(postimage.contains("> [!activity-day|2026-11-13]"));
    assert!(!postimage.contains("activity-meta"));
    assert!(!postimage.contains("> `—"));
    assert!(activity_core::activity_delivery::provenance_findings(postimage).is_empty());
    assert_eq!(run(&request), prepared);

    let mut processing = marking();
    processing["format"] = json!(PROVENANCE_FORMAT);
    processing["source_note"] = prepared["postimage"].clone();
    processing["preparation_provenance"] = provenance.clone();
    processing["provenance"] = provenance;
    processing["provenance"]["timestamp"] = json!("2026-11-13T09:01:00+01:00");
    let processed = run(&processing);
    assert_eq!(processed["status"], "complete", "{processed}");
    let processed_image = processed["postimage"].as_str().unwrap();
    assert!(processed_image.ends_with(">\n> *09:01 · Fictional Assistant · Vault processing*\n"));
    assert!(activity_core::activity_delivery::provenance_findings(processed_image).is_empty());
    assert_eq!(run(&processing), processed);
    let original_summary = "Unabridged fictional summary. ".repeat(80);
    processing["summary"] = json!(original_summary);
    let long = run(&processing);
    assert_eq!(long["status"], "complete", "{long}");
    let long = long["postimage"].as_str().unwrap();
    assert!(long.contains(&format!("> > ```text\n> > {original_summary}\n> > ```\n>\n> *09:01 · Fictional Assistant · Vault processing*\n")));

    // A changed second wouldn't change the footer, but it changes the intent.
    processing["preparation_provenance"]["timestamp"] = json!("2026-11-12T23:43:13Z");
    assert_eq!(run(&processing)["reason"], "prepared_postimage_changed");
    processing["preparation_provenance"]["timestamp"] = json!("2026-11-12T23:43:12Z");
    processing["provenance"]["kind"] = json!("chat-message");
    assert_eq!(
        run(&processing)["reason"],
        "vault_contribution_kind_or_day_mismatch"
    );
    request["date"] = json!("2026-11-12");
    assert_eq!(
        run(&request)["reason"],
        "vault_contribution_kind_or_day_mismatch"
    );
}

#[test]
fn v2_wiki_processing_retains_a_v1_preparation_and_supports_explicit_unknown_time() {
    let mut processing = marking();
    let original = processing["source_note"]
        .as_str()
        .unwrap()
        .split_once("\n---\n")
        .unwrap()
        .1
        .to_owned();
    processing["format"] = json!(PROVENANCE_FORMAT);
    processing["provenance"] = json!({
        "kind":"vault-contribution", "timestamp":null, "day":"2026-11-13",
        "display_timezone":"Europe/Amsterdam", "author":"Fictional Assistant", "channel":"Vault"
    });
    let result = run(&processing);
    assert_eq!(result["status"], "complete", "{result}");
    let body = result["postimage"].as_str().unwrap();
    assert!(body.contains(&original));
    assert!(body.ends_with(">\n> *— · Fictional Assistant · Vault*\n"));
    processing["format"] = json!(FORMAT);
    assert_eq!(
        run(&processing)["reason"],
        "provenance_requires_source_capture_v2"
    );
}

#[test]
fn schema_paths_dates_metadata_and_limits_fail_closed_without_postimage() {
    for (field, value) in [
        ("schema", "unknown"),
        ("format", "unknown"),
        ("operation", "move"),
        ("source_path", "../escape.md"),
        ("source_path", "00-inbox/20261102-source/other.md"),
        ("source_path", "00-inbox/20260230-source/20260230-source.md"),
        ("destination_owner", "30-topics/.hidden/.hidden.md"),
        ("destination_owner", "30-topics/example/other.md"),
        (
            "destination_owner",
            "30-topics/inject|target/inject|target.md",
        ),
        ("vault_root", "relative/path"),
        ("vault_root", "/fictional/../vault"),
        ("date", "2026-02-30"),
        ("date", "éé-02-03"),
        ("date", "+026-11-13"),
        (
            "source_note",
            "---\ntype: note\ntype: event\ncategory: source\n---\nBody",
        ),
        (
            "source_note",
            "---\ntype: task\ncategory: source\n---\nBody",
        ),
        ("source_note", "---\ntype: note\ncategory: email\n---\nBody"),
        (
            "source_note",
            "---\ntype: note\ncategory: source\nwiki_status: pending\n---\nBody",
        ),
    ] {
        let mut v = request();
        v[field] = json!(value);
        let out = run(&v);
        assert_eq!(out["status"], "incomplete", "{field}={value}: {out}");
        assert!(out.get("postimage").is_none());
    }
    let mut v = request();
    v["source_note"] = json!("x".repeat(MAX_NOTE_BYTES + 1));
    assert_eq!(run(&v)["status"], "incomplete");
    assert_eq!(
        respond(&vec![b' '; MAX_REQUEST_BYTES + 1])["reason"],
        "request_limit"
    );
    for input in [
        b"{broken".as_slice(),
        b"{\"schema\":\"one\",\"schema\":\"two\"}",
        b"[]",
    ] {
        assert_eq!(respond(input)["status"], "incomplete");
    }
    let encoded = serde_json::to_string(&request()).unwrap();
    let duplicate = format!(
        "{{\"schema\":\"viggobsidian.source-preparation.v1\",{}",
        &encoded[1..]
    );
    assert_eq!(
        respond(duplicate.as_bytes())["reason"],
        "duplicate_or_invalid_json"
    );
    let mut unknown = request();
    unknown["authorize_write"] = json!(true);
    assert_eq!(run(&unknown)["reason"], "unknown_field");
}

#[test]
fn native_cli_matches_library_and_never_needs_a_vault() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_source-note-transform"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&request()).unwrap())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&out.stdout).unwrap(),
        run(&request())
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_source-note-transform"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"{malformed")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&out.stdout).unwrap()["status"],
        "incomplete"
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_source-note-transform"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&vec![b' '; MAX_REQUEST_BYTES + 1])
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&out.stdout).unwrap()["reason"],
        "request_limit"
    );
}

#[test]
fn parser_refuses_all_merge_key_styles_but_preserves_literal_data() {
    for fragment in [
        "<< : {extra: value}",
        "meta: {<<: {extra: value}}",
        "meta:\n  << : {extra: value}",
        "meta: [{<<: {extra: value}}]",
        "!!merge '<<': {extra: value}",
        "base: &base {extra: value}\n<< : *base",
    ] {
        let mut v = request();
        v["source_note"] = json!(ORIGINAL.replacen("\n---\n", &format!("\n{fragment}\n---\n"), 1));
        let out = run(&v);
        assert_eq!(out["status"], "incomplete", "{fragment}: {out}");
        assert!(out.get("postimage").is_none());
        let mut child = Command::new(env!("CARGO_BIN_EXE_source-note-transform"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(&v).unwrap())
            .unwrap();
        let cli = child.wait_with_output().unwrap();
        assert_eq!(cli.status.code(), Some(2), "{fragment}");
        assert_eq!(serde_json::from_slice::<Value>(&cli.stdout).unwrap(), out);
    }
    for fragment in [
        "'<<': {extra: value}",
        "meta: {\"<<\": {extra: value}}",
        "!!str <<: {extra: value}",
        "literal: '<< : {extra: value}'",
    ] {
        let mut v = request();
        v["source_note"] = json!(ORIGINAL.replacen("\n---\n", &format!("\n{fragment}\n---\n"), 1));
        let out = run(&v);
        assert_eq!(out["status"], "complete", "{fragment}: {out}");
        assert!(out["postimage"].as_str().unwrap().contains(fragment));
    }
}
