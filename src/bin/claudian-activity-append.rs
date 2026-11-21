//! Prepare an explicit-unknown Claudian message for an existing guarded writer.
use activity_core::{activity_delivery, activity_layout, provenance};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::{self, Read};

const INPUT: &str = "viggobsidian.claudian-activity-append.v1";
const VERIFY: &str = "viggobsidian.claudian-activity-inspect.v1";
const OUTPUT: &str = "viggobsidian.claudian-activity-append-result.v1";
const MARKER: &str = "> <!-- viggobsidian:claudian-message:v1 id=";
const MAX_INPUT: usize = 4 * 1024 * 1024;
const MAX_NOTE: usize = 1024 * 1024;

fn hash(data: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(data))
}

fn require_keys(v: &Value, keys: &[&str]) -> Result<(), &'static str> {
    if v.as_object()
        .is_some_and(|o| o.len() == keys.len() && keys.iter().all(|key| o.contains_key(*key)))
    {
        Ok(())
    } else {
        Err("unexpected_or_missing_fields")
    }
}

fn valid_id(value: &Value) -> bool {
    value
        .as_str()
        .is_some_and(|s| !s.is_empty() && s.len() <= 256 && s.bytes().all(|b| b.is_ascii_graphic()))
}

fn valid_path(path: &str) -> bool {
    let Some((root, rest)) = path.split_once('/') else {
        return false;
    };
    matches!(
        root,
        "00-inbox" | "15-projects" | "20-projects" | "30-topics"
    ) && path.len() <= 1024
        && path.ends_with(".md")
        && !path.contains('\\')
        && !path.chars().any(char::is_control)
        && rest.split('/').all(|part| {
            !part.is_empty()
                && !part.starts_with('.')
                && part.trim() == part
                && !matches!(part, "raw" | "imported" | "generated" | "05-validation")
        })
}

fn valid_root(root: &str) -> bool {
    root.strip_prefix('/').is_some_and(|rest| {
        root.len() <= 4096
            && !root.contains('\\')
            && !root.chars().any(char::is_control)
            && rest.split('/').all(|part| !matches!(part, "" | "." | ".."))
    })
}

// Recognize the same source id even when a human or editor rewrapped the
// marker. Only direct bubble lines count: the nested original is literal.
fn carries_source_id(block: &[&str], id: &str) -> bool {
    let direct = block
        .iter()
        .filter_map(|line| {
            let text = line
                .trim_start_matches([' ', '\t'])
                .strip_prefix('>')?
                .trim_start_matches([' ', '\t']);
            (!text.starts_with('>')).then_some(text)
        })
        .collect::<Vec<_>>()
        .join("\n");
    direct.split("<!--").skip(1).any(|entry| {
        let Some(fields) = entry
            .trim_start()
            .strip_prefix("viggobsidian:claudian-message:v1")
        else {
            return false;
        };
        let comment = fields
            .split_once("-->")
            .map_or(fields, |(comment, _)| comment);
        let Some(id_field) = comment.trim_start().strip_prefix("id") else {
            return false;
        };
        id_field.trim_start().strip_prefix('=').is_some_and(|s| {
            let s = s.trim_start();
            let width = s.bytes().take_while(u8::is_ascii_hexdigit).count();
            s[..width].eq_ignore_ascii_case(id)
        })
    })
}

fn prepare(v: &Value) -> Result<Value, &'static str> {
    require_keys(
        v,
        &[
            "schema",
            "vault_root",
            "skill",
            "path",
            "preimage",
            "message",
        ],
    )?;
    let root = v["vault_root"]
        .as_str()
        .filter(|s| valid_root(s))
        .ok_or("absolute_vault_root_required")?;
    let path = v["path"]
        .as_str()
        .filter(|s| valid_path(s))
        .ok_or("invalid_owner_path")?;
    let skill = v["skill"]
        .as_str()
        .ok_or("existing_writer_skill_required")?;
    if !matches!(
        skill,
        "vault-project-refresh" | "vault-inbox" | "vault-capture"
    ) {
        return Err("existing_writer_skill_required");
    }
    let before = v["preimage"]
        .as_str()
        .filter(|s| s.len() <= MAX_NOTE && !s.contains(['\r', '\0']))
        .ok_or("bounded_lf_preimage_required")?;
    let body = before
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(_, body)| body))
        .ok_or("owner_frontmatter_required")?;
    let message = &v["message"];
    require_keys(
        message,
        &[
            "provider",
            "session_id",
            "message_id",
            "role",
            "body",
            "day_basis",
            "provenance",
        ],
    )?;
    if !valid_id(&message["provider"])
        || !valid_id(&message["session_id"])
        || !valid_id(&message["message_id"])
        || message["role"] != "human"
    {
        return Err("identified_original_human_message_required");
    }
    if !matches!(
        message["day_basis"].as_str(),
        Some("original-message-local-date" | "human-confirmed-message-local-date")
    ) {
        return Err("independently_known_message_day_required");
    }
    let original = message["body"]
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 256 * 1024 && !s.contains(['\r', '\0']))
        .ok_or("bounded_original_message_required")?;
    let facts = &message["provenance"];
    if facts["kind"] != "chat-message" || facts.get("timestamp") != Some(&Value::Null) {
        return Err("explicit_unknown_chat_time_required");
    }
    if !facts["channel"]
        .as_str()
        .is_some_and(|s| s == "Claudian" || s.starts_with("Claudian / "))
    {
        return Err("claudian_channel_required");
    }
    let event = provenance::render(facts)?;
    let id = hash(
        json!([
            "claudian",
            message["provider"],
            message["session_id"],
            message["message_id"]
        ])
        .to_string(),
    );
    let intent = hash(message.to_string());
    let marker_prefix = format!("{MARKER}{id} ");
    let bubble = format!(
        "> [!activity-message|human] {}\n{marker_prefix}intent={intent} -->\n{}{}",
        event.author,
        provenance::folded_verbatim(original),
        event.footer()
    );
    let mut tail_day = None;
    let mut same = false;
    let headings = activity_layout::visible_activity_headings(body);
    let literal_headings = body.lines().filter(|l| *l == "## Activity").count();
    if headings != literal_headings || headings > 1 {
        return Err("ambiguous_activity_heading");
    }
    if headings == 1 {
        if !activity_delivery::provenance_findings(body).is_empty() {
            return Err("existing_activity_not_canonical");
        }
        let tail = body
            .strip_prefix("## Activity\n")
            .or_else(|| body.split_once("\n## Activity\n").map(|(_, tail)| tail))
            .ok_or("invalid_activity_heading")?;
        let lines: Vec<_> = tail.lines().collect();
        for block in lines
            .split(|line| line.trim().is_empty())
            .filter(|b| !b.is_empty())
        {
            if let Some((day, _)) = block[0]
                .strip_prefix("> [!activity-day|")
                .and_then(|s| s.split_once("] "))
            {
                tail_day = Some(day);
            }
            if carries_source_id(block, &id) {
                if same || tail_day != Some(event.day.as_str()) || block.join("\n") + "\n" != bubble
                {
                    return Err("source_identity_conflict");
                }
                same = true;
            }
        }
    }
    if same {
        return Ok(
            json!({"schema":OUTPUT,"status":"reused","vault_root":root,"source_id_sha256":id,"intent_sha256":intent}),
        );
    }
    if tail_day.is_some_and(|day| day > event.day.as_str()) {
        return Err("historical_day_precedes_existing_activity_tail");
    }
    let lead = if headings == 0 {
        "\n\n## Activity\n\n"
    } else {
        "\n\n"
    };
    let day_separator = if tail_day == Some(event.day.as_str()) {
        String::new()
    } else {
        format!("> [!activity-day|{}] {}\n\n", event.day, event.day)
    };
    let append = format!("{lead}{day_separator}{bubble}");
    if before.len() + append.len() > MAX_NOTE {
        return Err("postimage_exceeds_replay_limit");
    }
    let postimage = format!("{before}{append}");
    if activity_layout::visible_activity_headings(&postimage) != 1
        || !activity_delivery::provenance_findings(&format!("{body}{append}")).is_empty()
    {
        return Err("noncanonical_append");
    }
    let goal = format!("claudian-message-{id}");
    let mut request = json!({"skill":skill,"goal":goal,"sources":[path],
        "operations":[{"op":"append","path":path,"text":append}]});
    if skill == "vault-capture" {
        let (package, _) = path.rsplit_once('/').ok_or("capture_package_required")?;
        request["capture_package"] = json!(package);
    }
    Ok(
        json!({"schema":OUTPUT,"status":"complete","vault_root":root,"source_id_sha256":id,"intent_sha256":intent,
        "before_sha256":hash(before),"after_sha256":hash(postimage),"request":request}),
    )
}

fn verify(v: &Value) -> Result<Value, &'static str> {
    require_keys(v, &["schema", "input", "inspection"])?;
    if v["input"]["schema"] != INPUT {
        return Err("invalid_original_input_schema");
    }
    let recipe = prepare(&v["input"])?;
    let request = &recipe["request"];
    let path = request["operations"][0]["path"]
        .as_str()
        .filter(|s| valid_path(s))
        .ok_or("invalid_recipe")?;
    let p = &v["inspection"];
    let row = &p["changes"][path];
    let is_hash = |value: &Value, width| {
        value.as_str().is_some_and(|s| {
            s.len() == width
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
    };
    if recipe["status"] != "complete"
        || p["root"] != recipe["vault_root"]
        || !is_hash(&recipe["before_sha256"], 64)
        || !is_hash(&recipe["after_sha256"], 64)
        || p["status"] != "read_only"
        || !is_hash(&p["run_id"], 32)
        || !is_hash(&p["plan_sha256"], 64)
        || p["skill"] != request["skill"]
        || p["goal"] != request["goal"]
        || p["changes"].as_object().is_none_or(|o| o.len() != 1)
        || p["sources"].as_object().is_none_or(|o| o.len() != 1)
        || p["sources"][path] != recipe["before_sha256"]
        || row["operation"] != "modify"
        || row["before"]["sha256"] != recipe["before_sha256"]
        || row["after"]["sha256"] != recipe["after_sha256"]
    {
        return Err("writer_inspection_does_not_match_recipe");
    }
    Ok(
        json!({"schema":OUTPUT,"status":"verified","run_id":p["run_id"],"plan_sha256":p["plan_sha256"]}),
    )
}

fn respond(input: &[u8]) -> Value {
    let result = (|| {
        if input.len() > MAX_INPUT {
            return Err("request_limit");
        }
        let v: Value = serde_json::from_slice(input).map_err(|_| "invalid_json")?;
        let options = serde_saphyr::options! {
            duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
            merge_keys: serde_saphyr::MergeKeyPolicy::Error
        };
        let _: Value = serde_saphyr::from_str_with_options(
            std::str::from_utf8(input).map_err(|_| "invalid_json")?,
            options,
        )
        .map_err(|_| "duplicate_or_invalid_json")?;
        match v["schema"].as_str() {
            Some(INPUT) => prepare(&v),
            Some(VERIFY) => verify(&v),
            _ => Err("invalid_schema"),
        }
    })();
    result.unwrap_or_else(|reason| json!({"schema":OUTPUT,"status":"incomplete","reason":reason}))
}

fn main() {
    let mut input = Vec::new();
    if io::stdin()
        .lock()
        .take(MAX_INPUT as u64 + 1)
        .read_to_end(&mut input)
        .is_err()
    {
        eprintln!("claudian-activity-append: stdin read failed");
        std::process::exit(2);
    }
    let response = respond(&input);
    println!("{response}");
    if response["status"] == "incomplete" {
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BEFORE: &str = "---\ntype: note\ncategory: knowledge\n---\n\n# Fictional owner\n";

    fn request() -> Value {
        json!({"schema":INPUT,"vault_root":"/fictional/vault","skill":"vault-project-refresh", "path":"20-projects/example/example.md", "preimage":BEFORE,
            "message":{"provider":"codex", "session_id":"session-example", "message_id":"message-example", "role":"human",
                "body":"Keep the whole source:\n\n> [!activity-day|2000-01-01] Not a date\n```text\nStory.\n```\n",
                "day_basis":"human-confirmed-message-local-date", "provenance":{"kind":"chat-message", "timestamp":null,
                    "author":"Example Person", "channel":"Claudian / Codex", "day":"2026-11-13", "display_timezone":"Europe/Amsterdam"}}})
    }

    fn run(v: &Value) -> Value {
        respond(v.to_string().as_bytes())
    }

    fn postimage(v: &Value, result: &Value) -> String {
        format!(
            "{}{}",
            v["preimage"].as_str().unwrap(),
            result["request"]["operations"][0]["text"].as_str().unwrap()
        )
    }

    #[test]
    fn stable_resumed_identity_and_literal_source() {
        let mut v = request();
        let a = run(&v);
        assert_eq!(a["status"], "complete", "{a}");
        assert_eq!(run(&v), a);
        let appended = postimage(&v, &a);
        assert_eq!(a["after_sha256"], hash(&appended));
        assert_eq!(a["before_sha256"], hash(BEFORE));
        assert!(appended.starts_with(BEFORE));
        assert!(appended.contains("> > > [!activity-day|2000-01-01] Not a date"));
        assert!(appended.contains("> > ````text\n"));
        assert!(appended.ends_with(">\n> *— · Example Person · Claudian / Codex*\n"));
        assert!(activity_delivery::provenance_findings(&appended).is_empty());
        v["preimage"] = json!(appended);
        let replay = run(&v);
        assert_eq!(replay["status"], "reused", "{replay}");
        assert_eq!(replay["source_id_sha256"], a["source_id_sha256"]);
        assert!(replay.get("request").is_none());
        for replacement in [
            ">  <!--  viggobsidian:claudian-message",
            "> <!--\n> viggobsidian:claudian-message",
        ] {
            let mut changed = v.clone();
            changed["preimage"] = json!(
                v["preimage"]
                    .as_str()
                    .unwrap()
                    .replace("> <!-- viggobsidian:claudian-message", replacement)
            );
            assert_eq!(run(&changed)["reason"], "source_identity_conflict");
        }
        v["message"]["body"] = json!("Not the same message");
        assert_eq!(run(&v)["reason"], "source_identity_conflict");
    }

    #[test]
    fn same_day_reuse_and_historical_refusal() {
        let mut v = request();
        let first = run(&v);
        let before = postimage(&v, &first);
        v["preimage"] = json!(before);
        v["message"]["message_id"] = json!("second-message");
        let a = run(&v);
        assert_eq!(a["status"], "complete", "{a}");
        assert_eq!(
            postimage(&v, &a)
                .matches("> [!activity-day|2026-11-13]")
                .count(),
            1
        );
        v["message"]["provenance"]["day"] = json!("2026-11-12");
        assert_eq!(
            run(&v)["reason"],
            "historical_day_precedes_existing_activity_tail"
        );
        v["message"]["provenance"]["day"] = json!("2026-11-14");
        let a = run(&v);
        assert_eq!(a["status"], "complete", "{a}");
        assert!(postimage(&v, &a).contains("> [!activity-day|2026-11-14]"));
        v["preimage"] = json!(postimage(&v, &a));
        v["message"] = request()["message"].clone();
        assert_eq!(run(&v)["status"], "reused");
    }

    #[test]
    fn missing_aggregated_and_fallback_metadata_refuse() {
        let mut v = request();
        v["message"]["day_basis"] = json!("session-update");
        assert_eq!(
            run(&v)["reason"],
            "independently_known_message_day_required"
        );
        v = request();
        v["message"]["provenance"]
            .as_object_mut()
            .unwrap()
            .remove("day");
        assert_eq!(run(&v)["reason"], "invalid_provenance_day");
        v = request();
        v["message"]["provenance"]["timestamp"] = json!("2026-11-13T09:04:11Z");
        assert_eq!(run(&v)["reason"], "explicit_unknown_chat_time_required");
        v = request();
        v["message"]["role"] = json!("transcript");
        assert_eq!(
            run(&v)["reason"],
            "identified_original_human_message_required"
        );
        v = request();
        v["message"]["message_id"] = json!(null);
        assert_eq!(
            run(&v)["reason"],
            "identified_original_human_message_required"
        );
        let ambiguous = request().to_string().replace(
            "\"timestamp\":null",
            "\"timestamp\":null,\"timestamp\":null",
        );
        assert_eq!(
            respond(ambiguous.as_bytes())["reason"],
            "duplicate_or_invalid_json"
        );
    }

    #[test]
    fn hidden_or_unclosed_activity_and_dirty_plan_refuse() {
        let mut v = request();
        v["preimage"] = json!(format!("{BEFORE}\n```text\n## Activity\n```\n"));
        assert_eq!(run(&v)["reason"], "ambiguous_activity_heading");
        for heading in [
            "## Activity ##",
            "Activity\n--------",
            "Activity\n=========",
            "   ##\tActivity ##",
        ] {
            v["preimage"] = json!(format!("{BEFORE}\n{heading}\n"));
            assert_eq!(run(&v)["reason"], "ambiguous_activity_heading", "{heading}");
        }
        v["preimage"] = json!(format!("{BEFORE}\n<!-- open comment"));
        assert_eq!(run(&v)["reason"], "noncanonical_append");
        v = request();
        let recipe = run(&v);
        let path = v["path"].as_str().unwrap();
        let mut inspect = json!({"schema":VERIFY,"input":v,
            "inspection":{"status":"read_only","root":"/fictional/vault","run_id":"0".repeat(32),"plan_sha256":"1".repeat(64),
                "skill":recipe["request"]["skill"],"goal":recipe["request"]["goal"],"sources":{path:recipe["before_sha256"]},
                "changes":{path:{"operation":"modify", "before":{"sha256":recipe["before_sha256"]},
                    "after":{"sha256":recipe["after_sha256"]}}}}});
        assert_eq!(run(&inspect)["status"], "verified");
        let mut tampered = inspect.clone();
        tampered.as_object_mut().unwrap().remove("input");
        tampered["recipe"] = recipe;
        tampered["recipe"]["request"]["operations"][0]["text"] = json!("CHANGED");
        assert_eq!(run(&tampered)["reason"], "unexpected_or_missing_fields");
        let mut tampered = inspect.clone();
        tampered["input"]["message"]["body"] = json!("CHANGED");
        assert_eq!(
            run(&tampered)["reason"],
            "writer_inspection_does_not_match_recipe"
        );
        inspect["inspection"]["changes"][path]["before"]["sha256"] = json!("2".repeat(64));
        assert_eq!(
            run(&inspect)["reason"],
            "writer_inspection_does_not_match_recipe"
        );
    }

    #[test]
    fn only_replayable_note_size_is_accepted() {
        let mut v = request();
        v["preimage"] = json!(format!("{BEFORE}{}", "a".repeat(1_040_000)));
        v["message"]["body"] = json!("a".repeat(10_000));
        assert_eq!(run(&v)["reason"], "postimage_exceeds_replay_limit");
    }
}
