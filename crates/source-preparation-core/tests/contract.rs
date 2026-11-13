use serde_json::{Value, json};
use source_preparation_core::{FORMAT, MAX_NOTE_BYTES, MAX_REQUEST_BYTES, hash, respond};
use std::io::Write;
use std::process::{Command, Stdio};

const ORIGINAL: &str = "---\ntype: note\ncategory: source\ncreated: 2026-11-02\nsource_identity: fixture:original\n---\n\n# Original source\n\n## Original request\n\nKeep these exact words: café → knowledge.\n\n> [!note] Original callout\n> Unchanged.\n\n```text\nAn original fence.\n```\n";

fn request() -> Value {
    json!({"schema":"viggobsidian.source-preparation.v1", "format":FORMAT, "operation":"prepare",
        "date":"2026-11-13", "source_note":ORIGINAL, "source_path":"00-inbox/20261102-source/20261102-source.md",
        "destination_owner":"30-topics/example/example.md", "vault_root":"/fictional/vault"})
}

fn run(v: &Value) -> Value {
    respond(&serde_json::to_vec(v).unwrap())
}

#[test]
fn fictional_prepare_matches_frozen_postimage_and_preserves_original() {
    let result = run(&request());
    assert_eq!(result["status"], "complete", "{result}");
    assert_eq!(
        result["postimage"],
        serde_json::from_str::<String>(include_str!("../fixtures/prepared.json")).unwrap()
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
        serde_json::from_str::<String>(include_str!("../fixtures/processed.json")).unwrap()
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
