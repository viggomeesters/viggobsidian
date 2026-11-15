use serde_json::Value;
use std::process::Command;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_viggobsidian"))
}

#[test]
fn product_info_prints_the_versioned_catalog_without_a_vault() {
    let output = cli().arg("product-info").output().unwrap();
    assert!(output.status.success(), "stderr: {:?}", output.stderr);
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema"], "viggobsidian.product-info.v1");
    assert_eq!(value["engine"]["name"], "viggobsidian");
    assert_eq!(value["engine"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        value["catalog"]["schema"],
        "viggobsidian.product-capabilities.v1"
    );
    assert_eq!(value["catalog"]["catalog_version"], 1);
}

#[test]
fn product_info_rejects_a_nonexistent_vault_without_probing_it() {
    let nonexistent = format!(
        "/definitely-not-a-vault-product-info-{}",
        std::process::id()
    );
    for arguments in [
        vec!["--vault", nonexistent.as_str()],
        vec!["--help"],
        vec!["positional"],
        vec!["--"],
    ] {
        let output = cli().arg("product-info").args(&arguments).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "{arguments:?}");
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "product-info accepts no arguments\n"
        );
    }
}
