//! Explicit, compiled allowlist for reviewed reusable skill instructions.
//! Local adapters, examples, providers and scripts cannot widen this surface.
use serde_json::Value;
use std::{collections::BTreeSet, sync::OnceLock};

pub(crate) fn targets() -> &'static BTreeSet<String> {
    static TARGETS: OnceLock<BTreeSet<String>> = OnceLock::new();
    TARGETS.get_or_init(|| {
        let spec: Value = serde_json::from_str(include_str!("../schemas/skill-library.json"))
            .expect("checked-in skill library schema");
        spec["files"]
            .as_array()
            .expect("skill library files")
            .iter()
            .map(|row| row["target"].as_str().expect("skill target").to_owned())
            .collect()
    })
}
