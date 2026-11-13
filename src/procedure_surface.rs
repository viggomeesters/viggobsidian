//! Compiled substantive vault-contract surface, without retired route wrappers.
use serde_json::Value;
use std::{collections::BTreeSet, sync::OnceLock};
pub(crate) fn targets() -> &'static BTreeSet<String> {
    static TARGETS: OnceLock<BTreeSet<String>> = OnceLock::new();
    TARGETS.get_or_init(|| {
        let spec: Value = serde_json::from_str(include_str!("../schemas/procedures.json")).unwrap();
        let mut targets = BTreeSet::from([
            ".viggobsidian/core/procedures.json".to_owned(),
            ".agents/skills/vault/SKILL.md".to_owned(),
        ]);
        for item in spec["contracts"].as_array().unwrap() {
            targets.insert(format!(
                ".viggobsidian/core/procedures/{}.md",
                item.as_str().unwrap()
            ));
        }
        targets
    })
}
