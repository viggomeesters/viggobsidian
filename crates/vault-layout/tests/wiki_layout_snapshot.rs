use serde_json::{Value, json};
use std::{fs, process::Command};

#[test]
fn filesystem_cli_matches_frozen_text_corpus_without_changing_notes() {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("wiki-parity-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("99-system/contracts")).unwrap();
    fs::write(
        root.join("99-system/contracts/vault-schema.yaml"),
        include_str!("../fixtures/layout-contract.yaml"),
    )
    .unwrap();
    let corpus: Value = serde_json::from_str(include_str!(
        "../../wiki-layout-core/fixtures/wiki-layout-legacy.json"
    ))
    .unwrap();
    for case in corpus["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let path = format!("{id}/{id}.md");
        let source = format!(
            "---\ntype: reference\ncategory: knowledge\nlayer: wiki\ncssclasses: [vault-wiki, vault-wiki-direct]\n---\n{}",
            case["body"].as_str().unwrap()
        );
        fs::create_dir_all(root.join(id)).unwrap();
        fs::write(root.join(&path), &source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
            .args(["--json", "--vault", root.to_str().unwrap(), id])
            .output()
            .unwrap();
        assert_ne!(
            output.status.code(),
            Some(2),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let native: Value = serde_json::from_slice(&output.stdout).unwrap();
        let issues = native["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|i| {
                i["code"].as_str().is_some_and(|c| {
                    [
                        "wiki-preamble",
                        "wiki-facts",
                        "wiki-facts-position",
                        "wiki-toc",
                        "wiki-toc-anchor",
                        "wiki-source-inventory",
                    ]
                    .contains(&c)
                })
            })
            .map(|i| json!({"code":i["code"],"message":i["message"]}))
            .collect::<Vec<_>>();
        assert_eq!(json!(issues), case["findings"], "{} native {}", id, native);
        assert_eq!(fs::read_to_string(root.join(&path)).unwrap(), source);
    }
    fs::create_dir_all(root.join("ordinary")).unwrap();
    let ordinary =
        "---\ntype: reference\ncategory: knowledge\n---\n# Ordinary\n\n> [!wiki-facts]\nmalformed";
    fs::write(root.join("ordinary/ordinary.md"), ordinary).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
        .args(["--json", "--vault", root.to_str().unwrap(), "ordinary"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(!report["findings"].as_array().unwrap().iter().any(|i| {
        i["code"]
            .as_str()
            .is_some_and(|code| code.starts_with("wiki-"))
    }));
    assert_eq!(
        fs::read_to_string(root.join("ordinary/ordinary.md")).unwrap(),
        ordinary
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn distributed_native_engine_builds_offline_with_its_path_dependencies() {
    let scratch = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("wiki-installed-{}", std::process::id()));
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let distribution: Value =
        serde_json::from_slice(&fs::read(source.join("distribution.json")).unwrap()).unwrap();
    for entry in distribution["files"].as_array().unwrap() {
        let target = entry["target"].as_str().unwrap();
        if !target.starts_with(".agents/tools/") {
            continue;
        }
        let destination = scratch.join(target);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(source.join(entry["source"].as_str().unwrap()), destination).unwrap();
    }
    let output = Command::new(env!("CARGO"))
        .args(["check", "--offline", "--locked", "--manifest-path"])
        .arg(scratch.join(".agents/tools/vault-layout/Cargo.toml"))
        .arg("--target-dir")
        .arg(scratch.join("target"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(scratch).unwrap();
}
