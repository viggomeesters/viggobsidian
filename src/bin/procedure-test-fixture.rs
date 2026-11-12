//! Snapshot only the finite public procedure surface for isolated integration tests.
//! Does not inspect a vault, provider configuration or installed user content.
#[path = "../procedure_surface.rs"]
mod procedure_surface;

use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Component, Path},
};

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let distribution: Value = serde_json::from_slice(&fs::read(root.join("distribution.json"))?)?;
    let mut files = BTreeMap::new();
    for target in procedure_surface::targets() {
        let entries: Vec<_> = distribution["files"]
            .as_array()
            .ok_or("files missing")?
            .iter()
            .filter(|row| row["target"] == *target)
            .collect();
        if entries.len() != 1 {
            return Err("procedure target needs exactly one source".into());
        }
        let source = entries[0]["source"].as_str().ok_or("source missing")?;
        if !Path::new(source)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
        {
            return Err("invalid source locator".into());
        }
        files.insert(
            target,
            json!({"source":source,"content":fs::read_to_string(root.join(source))?}),
        );
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema":"viggobsidian.procedure-test-fixture.v1", "files":files
        }))?
    );
    Ok(())
}
