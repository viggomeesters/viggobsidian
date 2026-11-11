//! The release owns generic navigation metadata; local schema retains exact scopes.
use serde_json::Value;
use std::{fs, io, path::Path};

const CONTRACT: &str = ".viggobsidian/core/navigation-contract.yaml";

pub(crate) fn apply(root: &Path, schema: &mut Value) -> io::Result<()> {
    let path = root.join(CONTRACT);
    if !path.exists() && !root.join(".viggobsidian/installed.json").exists() {
        // Standalone engine fixtures and non-adopted deployments keep their contract.
        return Ok(());
    }
    let invalid = |message: &str| io::Error::new(io::ErrorKind::InvalidData, message.to_owned());
    if !schema.is_object()
        || [
            "/validation_profiles",
            "/validation_profiles/minimal",
            "/common_fields",
            "/common_fields/layer",
            "/contracts",
            "/contracts/raw_package",
        ]
        .iter()
        .any(|p| {
            schema
                .pointer(p)
                .is_some_and(|v| !v.is_object() && !v.is_null())
        })
    {
        return Err(invalid(
            "local schema must contain mapping-shaped navigation fields",
        ));
    }
    let mut parent = path.as_path();
    while parent != root {
        if fs::symlink_metadata(parent)?.file_type().is_symlink() {
            return Err(invalid(
                "navigation contract must not traverse symbolic links",
            ));
        }
        parent = parent
            .parent()
            .ok_or_else(|| invalid("invalid navigation contract path"))?;
    }
    let options =
        serde_saphyr::options! { duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error };
    let data: Value = serde_saphyr::from_str_with_options(&fs::read_to_string(path)?, options)
        .map_err(|e| invalid(&format!("invalid navigation contract: {e}")))?;
    if data["schema"] != "viggobsidian.navigation.v1"
        || data["version"] != 1
        || data["profile"] != "numbered-v1"
    {
        return Err(invalid("unsupported navigation contract"));
    }
    for pointer in [
        "/layout/validation_profiles/minimal/required",
        "/layout/common_fields/layer/allowed_values",
        "/layout/contracts/raw_package/required_fields",
    ] {
        if !data.pointer(pointer).is_some_and(|v| {
            v.as_array()
                .is_some_and(|items| !items.is_empty() && items.iter().all(Value::is_string))
        }) {
            return Err(invalid(
                "navigation contract lacks required metadata arrays",
            ));
        }
    }
    if !data["layout"]["contracts"]["raw_package"]["wiki_status_values"]
        .as_object()
        .is_some_and(|values| values.contains_key("pending") && values.contains_key("processed"))
    {
        return Err(invalid("navigation contract lacks Raw processing states"));
    }
    schema["validation_profiles"]["minimal"]["required"] =
        data["layout"]["validation_profiles"]["minimal"]["required"].clone();
    schema["common_fields"]["layer"]["allowed_values"] =
        data["layout"]["common_fields"]["layer"]["allowed_values"].clone();
    schema["contracts"]["raw_package"]["required_fields"] =
        data["layout"]["contracts"]["raw_package"]["required_fields"].clone();
    schema["contracts"]["raw_package"]["wiki_status_values"] =
        data["layout"]["contracts"]["raw_package"]["wiki_status_values"].clone();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn shared_contract_overrides_only_generic_metadata_and_fails_closed() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("nav-contract-{}", std::process::id()));
        fs::create_dir_all(root.join(".viggobsidian/core")).unwrap();
        let path = root.join(CONTRACT);
        let source = include_str!("../fixtures/navigation-contract.yaml");
        fs::write(&path, source).unwrap();
        let mut schema = json!({"validation_profiles":{"minimal":{"required":["legacy"]}}, "strict_layout_exceptions":{"writer_files":{"exact/path":"local"}}, "contracts":{"task":{"status_values":["local-status"]}}});
        apply(&root, &mut schema).unwrap();
        assert!(apply(&root, &mut json!({"contracts":[]})).is_err());
        assert_eq!(
            schema["validation_profiles"]["minimal"]["required"],
            json!(["type", "category"])
        );
        assert_eq!(
            schema["contracts"]["task"]["status_values"],
            json!(["local-status"])
        );
        assert_eq!(
            schema["strict_layout_exceptions"]["writer_files"]["exact/path"],
            "local"
        );
        fs::write(&path, "schema: unsupported").unwrap();
        assert!(apply(&root, &mut schema).is_err());
        fs::remove_file(&path).unwrap();
        fs::write(root.join(".viggobsidian/installed.json"), "{}").unwrap();
        assert!(apply(&root, &mut schema).is_err());
        std::os::unix::fs::symlink(root.join(".viggobsidian/installed.json"), &path).unwrap();
        assert!(apply(&root, &mut schema).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
