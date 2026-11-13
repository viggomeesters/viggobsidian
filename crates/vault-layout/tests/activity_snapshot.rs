use serde_json::{Value, json};
use std::{fs, process::Command};

#[test]
fn native_strict_selection_retains_frozen_activity_outputs_and_source_bytes() {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("activity-native-{}", std::process::id()));
    fs::create_dir_all(root.join("99-system/contracts")).unwrap();
    fs::write(
        root.join("99-system/contracts/vault-schema.yaml"),
        include_str!("../fixtures/layout-contract.yaml"),
    )
    .unwrap();
    let corpus: Value = serde_json::from_str(include_str!(
        "../../activity-core/fixtures/activity-legacy.json"
    ))
    .unwrap();
    for case in corpus["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        for (suffix, metadata, selected) in [
            (
                "provenance",
                "activity_contract: provenance-v1",
                Some(("strict-activity-provenance", "provenance")),
            ),
            (
                "chat",
                "activity_format: chat-v1",
                Some(("activity-message", "chat")),
            ),
            ("legacy", "", None),
            (
                "unknown",
                "activity_contract: future",
                Some(("strict-activity-provenance", "unknown")),
            ),
        ] {
            let folder = format!("{id}-{suffix}");
            let path = format!("{folder}/{folder}.md");
            let source = format!(
                "---\ntype: reference\ncategory: knowledge\n{metadata}\n---\n{}",
                case["body"].as_str().unwrap()
            );
            fs::create_dir_all(root.join(&folder)).unwrap();
            fs::write(root.join(&path), &source).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
                .args([
                    "--json",
                    "--strict",
                    "--vault",
                    root.to_str().unwrap(),
                    &folder,
                ])
                .output()
                .unwrap();
            assert_ne!(
                output.status.code(),
                Some(2),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let report: Value = serde_json::from_slice(&output.stdout).unwrap();
            for (code, key) in [
                ("strict-mail-draft", "mail"),
                ("strict-activity-delivery", "delivery"),
                ("strict-activity-provenance", "provenance"),
                ("activity-message", "chat"),
            ] {
                let actual: Vec<_> = report["findings"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|f| f["code"] == code)
                    .map(|f| f["message"].clone())
                    .collect();
                let expected = if code == "strict-mail-draft" || code == "strict-activity-delivery"
                {
                    case[key].clone()
                } else if let Some((selected_code, selected_key)) = selected {
                    if code == selected_code {
                        if selected_key == "unknown" {
                            json!(["unknown activity_contract"])
                        } else {
                            case[selected_key].clone()
                        }
                    } else {
                        json!([])
                    }
                } else {
                    json!([])
                };
                assert_eq!(json!(actual), expected, "{id} {suffix} {code}");
            }
            assert_eq!(fs::read_to_string(root.join(path)).unwrap(), source);
        }
    }
    fs::remove_dir_all(root).unwrap();
}
