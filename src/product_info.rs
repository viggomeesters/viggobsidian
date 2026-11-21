use serde_json::{Value, json};

const CATALOG: &str = include_str!("../schemas/product-capabilities.v1.json");
const CATALOG_SCHEMA: &str = "viggobsidian.product-capabilities.v1";
const PRODUCT_INFO_SCHEMA: &str = "viggobsidian.product-info.v1";

// Rust owns the declarations; the JSON is their checked portable export.
// Neither is a runtime availability probe, grant of authority or rule engine.
struct Capability {
    id: &'static str,
    status: &'static str,
    surface: &'static str,
    protocols: &'static [&'static str],
    rulesets: &'static [&'static str],
    commands: &'static [&'static str],
    scope: &'static str,
}

const CAPABILITIES: &[Capability] = &[
    Capability {
        id: "link-style-text",
        status: "shipped",
        surface: "portable-text",
        protocols: &[
            "viggobsidian.link-style-snapshot.v1",
            "viggobsidian.link-style-result.v1",
        ],
        rulesets: &[],
        commands: &[],
        scope: "Captured Markdown inventory and visible link-style diagnostics only.",
    },
    Capability {
        id: "wiki-layout-text",
        status: "shipped",
        surface: "portable-text",
        protocols: &[
            "viggobsidian.wiki-layout-snapshot.v1",
            "viggobsidian.wiki-layout-snapshot.v2",
            "viggobsidian.wiki-layout-result.v1",
        ],
        rulesets: &["wiki-layout.v1"],
        commands: &[],
        scope: "Captured selected Wiki text only; selection, filesystem and writer authority remain native.",
    },
    Capability {
        id: "activity-text",
        status: "shipped",
        surface: "portable-text",
        protocols: &[
            "viggobsidian.activity-snapshot.v1",
            "viggobsidian.activity-result.v1",
        ],
        rulesets: &["activity-text.v1"],
        commands: &[],
        scope: "Captured Activity text only; native strict selection and domain actions remain native.",
    },
    Capability {
        id: "source-preparation-native",
        status: "shipped",
        surface: "native-only",
        protocols: &[
            "viggobsidian.source-preparation.v1",
            "viggobsidian.source-preparation-result.v1",
        ],
        rulesets: &["source-capture-v1", "source-capture-v2"],
        commands: &["source-note-transform"],
        scope: "Read-only typed Raw source postimages; it neither selects a vault nor grants writer authority.",
    },
    Capability {
        id: "vault-layout-native",
        status: "shipped",
        surface: "native-only",
        protocols: &[],
        rulesets: &[],
        commands: &["vault-layout"],
        scope: "Native filesystem selection, strict validation and local adapter boundaries.",
    },
    Capability {
        id: "package-lifecycle-native",
        status: "shipped",
        surface: "native-only",
        protocols: &[
            "viggobsidian.plan.v1",
            "viggobsidian.receipt.v1",
            "viggobsidian.receipt.v2",
        ],
        rulesets: &[],
        commands: &["plan", "apply", "rollback", "check", "doctor"],
        scope: "Existing native package lifecycle; local writers retain exclusive execution and recovery authority.",
    },
    Capability {
        id: "visible-folder-entrance-proof",
        status: "unshipped-proof",
        surface: "portable-text-proof",
        protocols: &[
            "viggobsidian.structure-snapshot.v1",
            "viggobsidian.structure-result.v1",
        ],
        rulesets: &["visible-folder-entrance.v1"],
        commands: &["structure-snapshot-check"],
        scope: "Bounded visible-inventory experiment; it is not a shipped product capability or filesystem attestation.",
    },
];

fn canonical_catalog() -> Value {
    let capabilities: Vec<_> = CAPABILITIES
        .iter()
        .map(|entry| {
            json!({
                "id": entry.id, "status": entry.status, "surface": entry.surface,
                "protocols": entry.protocols, "rulesets": entry.rulesets,
                "commands": entry.commands, "scope": entry.scope,
            })
        })
        .collect();
    json!({"schema": CATALOG_SCHEMA, "catalog_version": 1, "capabilities": capabilities})
}

fn validate_catalog(catalog: &Value) -> Result<(), String> {
    // Includes every field, ordered array and authority/scope ceiling.
    if catalog != &canonical_catalog() {
        return Err(
            "embedded product capability catalog differs from canonical Rust declarations".into(),
        );
    }
    Ok(())
}

pub(crate) fn catalog() -> Result<Value, String> {
    let catalog: Value = serde_json::from_str(CATALOG)
        .map_err(|error| format!("embedded product capability catalog is invalid JSON: {error}"))?;
    validate_catalog(&catalog)?;
    Ok(catalog)
}

pub(crate) fn run(arguments: &[String]) -> Result<Value, String> {
    if !arguments.is_empty() {
        return Err("product-info accepts no arguments".into());
    }
    Ok(json!({
        "schema": PRODUCT_INFO_SCHEMA,
        "engine": {"name": env!("CARGO_PKG_NAME"), "version": env!("CARGO_PKG_VERSION")},
        "catalog": catalog()?
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_export_exactly_matches_all_canonical_declarations() {
        assert_eq!(catalog().unwrap(), canonical_catalog());
    }

    #[test]
    fn changed_catalog_shapes_versions_and_capabilities_are_rejected() {
        let valid = canonical_catalog();
        for key in ["schema", "catalog_version", "capabilities"] {
            let mut changed = valid.clone();
            changed.as_object_mut().unwrap().remove(key);
            assert!(validate_catalog(&changed).is_err(), "missing {key}");
        }
        let mut changed = valid.clone();
        changed["extra"] = json!(true);
        assert!(validate_catalog(&changed).is_err());
        for (key, value) in [
            ("schema", json!("future")),
            ("catalog_version", json!(2)),
            ("capabilities", json!({})),
        ] {
            let mut changed = valid.clone();
            changed[key] = value;
            assert!(validate_catalog(&changed).is_err(), "changed {key}");
        }
        for key in [
            "id",
            "status",
            "surface",
            "protocols",
            "rulesets",
            "commands",
            "scope",
        ] {
            let mut changed = valid.clone();
            changed["capabilities"][0]
                .as_object_mut()
                .unwrap()
                .remove(key);
            assert!(validate_catalog(&changed).is_err(), "missing entry {key}");
        }
        let mut changed = valid.clone();
        changed["capabilities"][0]["extra"] = json!(true);
        assert!(validate_catalog(&changed).is_err());
        for key in ["id", "status", "surface", "scope"] {
            let mut changed = valid.clone();
            changed["capabilities"][0][key] = json!("invented");
            assert!(validate_catalog(&changed).is_err(), "changed entry {key}");
        }
        let mut changed = valid.clone();
        changed["capabilities"].as_array_mut().unwrap().remove(0);
        assert!(validate_catalog(&changed).is_err());
        let mut changed = valid.clone();
        changed["capabilities"][1] = changed["capabilities"][0].clone();
        assert!(validate_catalog(&changed).is_err());
    }

    #[test]
    fn protocol_ruleset_and_command_mutations_never_pass_as_compatible() {
        for (index, field) in [(1, "protocols"), (3, "rulesets"), (5, "commands")] {
            for mutation in 0..6 {
                let mut changed = canonical_catalog();
                let value = &mut changed["capabilities"][index][field];
                match mutation {
                    0 => value.as_array_mut().unwrap().push(json!("invented")),
                    1 => {
                        value.as_array_mut().unwrap().remove(0);
                    }
                    2 => value[0] = json!("invented"),
                    3 => value[0] = json!(42),
                    4 => *value = json!("not-an-array"),
                    _ => *value = Value::Null,
                }
                assert!(
                    validate_catalog(&changed).is_err(),
                    "{field} mutation {mutation}"
                );
            }
        }
        for (index, field) in [(1, "protocols"), (5, "commands")] {
            let mut changed = canonical_catalog();
            changed["capabilities"][index][field]
                .as_array_mut()
                .unwrap()
                .swap(0, 1);
            assert!(validate_catalog(&changed).is_err(), "reordered {field}");
        }
    }

    #[test]
    fn product_info_reports_only_compiled_identity_and_declarations() {
        let output = run(&[]).unwrap();
        assert_eq!(output["schema"], PRODUCT_INFO_SCHEMA);
        assert_eq!(output["engine"]["name"], env!("CARGO_PKG_NAME"));
        assert_eq!(output["engine"]["version"], env!("CARGO_PKG_VERSION"));
        assert!(run(&["--vault".into(), "/does-not-exist".into()]).is_err());
    }
}
