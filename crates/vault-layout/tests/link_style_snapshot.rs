use serde_json::{Value, json};
use std::{fs, process::Command};

#[test]
fn filesystem_cli_and_snapshot_share_exact_issue_and_line_semantics() {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("link-style-parity-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let documents = [
        (
            "owner.md",
            "# Own\n[[d/wiki/wiki#Sectie]] [[missing/path]] [[wiki]]\n",
        ),
        (
            "d/wiki/wiki.md",
            "---\r\nlayer: wiki\r\n---\r\n# Élève\r\n[[d/raw/20261111-source/20261111-source]]\r\n",
        ),
        ("other/wiki/wiki.md", "# Other\n"),
        (
            "d/raw/20261111-source/20261111-source.md",
            "# Source\n\n> [!navigation|shortcuts]\n> - [[d/wiki/wiki]]\n\n## Activity\nHuman [[d/wiki/wiki]]\n> Quote [[d/wiki/wiki]]\n",
        ),
        (
            "mask.md",
            "---\nref: '[[missing/path]]'\n---\n# Mask\n```md\n[[missing/path]]\n```\n> Quote [[missing/path]]\n![[missing/path]]\n`[[missing/path]]` <!-- [[missing/path]] -->\n",
        ),
        ("90-templates/template.md", "[[missing/path]]\n"),
    ];
    for (path, markdown) in documents {
        let file = root.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, markdown).unwrap();
    }
    let request = json!({"schema":"viggobsidian.link-style-snapshot.v1","complete":true,
        "inventory":documents.iter().map(|(p,_)|p).collect::<Vec<_>>(),
        "notes":documents.iter().map(|(p,m)|json!({"path":p,"markdown":m})).collect::<Vec<_>>()});
    let response = link_style_core::respond(&serde_json::to_vec(&request).unwrap());
    assert_eq!(response["status"], "complete");
    let output = Command::new(env!("CARGO_BIN_EXE_vault-layout"))
        .args(["--json", "--vault", root.to_str().unwrap(), "--link-style"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let native: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["report"], native);
    assert!(
        native["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["kind"] == "unresolved-long-link")
    );
    assert!(
        native["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["kind"] == "wiki-source-style-class")
    );
    assert!(
        !native["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["path"] == "mask.md")
    );
    for (path, markdown) in documents {
        assert_eq!(fs::read_to_string(root.join(path)).unwrap(), markdown);
    }
    fs::remove_dir_all(root).unwrap();
}
