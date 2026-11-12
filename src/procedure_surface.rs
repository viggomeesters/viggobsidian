//! Compiled release allowlist; an untrusted package cannot widen this surface.
use serde_json::Value;
use std::{collections::BTreeSet, sync::OnceLock};

pub(crate) fn targets() -> &'static BTreeSet<String> {
    static TARGETS: OnceLock<BTreeSet<String>> = OnceLock::new();
    TARGETS.get_or_init(|| {
        let spec: Value = serde_json::from_str(include_str!("../schemas/procedures.json")).unwrap();
        let mut targets = BTreeSet::from([
            ".viggobsidian/core/procedures.json".to_owned(),
            ".agents/skills/_bundle/contracts/native-vault.md".to_owned(),
        ]);
        for item in spec["contracts"].as_array().unwrap() {
            targets.insert(format!(
                ".viggobsidian/core/procedures/{}.md",
                item.as_str().unwrap()
            ));
        }
        for (name, route) in spec["routes"].as_object().unwrap() {
            targets.insert(format!(
                ".agents/skills/{name}/{}",
                if route["kind"] == "entry" {
                    "SKILL.md"
                } else {
                    "PROCEDURE.md"
                }
            ));
        }
        for reference in spec["references"].as_object().unwrap().keys() {
            let (name, file) = reference.split_once('/').unwrap();
            targets.insert(format!(".agents/skills/{name}/references/{file}.md"));
        }
        targets
    })
}
