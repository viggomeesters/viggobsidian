//! Author the reviewed compatibility surface from the public coverage map.
//! This writes repository assets only, never selects or edits a target vault.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() == 2 && args[0] == "--adoption-map" {
        let vault = Path::new(&args[1]).canonicalize()?;
        let distribution: Value =
            serde_json::from_slice(&fs::read(root.join("distribution.json"))?)?;
        let mut hashes = serde_json::Map::new();
        for file in distribution["files"].as_array().unwrap() {
            let raw = file["target"].as_str().unwrap();
            let mut path = vault.clone();
            for part in Path::new(raw).components() {
                if !matches!(part, std::path::Component::Normal(_)) {
                    return Err("unsafe inventory path".into());
                }
                path.push(part);
                if fs::symlink_metadata(&path).is_ok_and(|m| m.is_symlink()) {
                    return Err("inventory symlink".into());
                }
            }
            if path.exists() {
                hashes.insert(
                    raw.into(),
                    json!(format!("{:x}", Sha256::digest(fs::read(path)?))),
                );
            }
        }
        println!("{}", serde_json::to_string_pretty(&hashes)?);
        return Ok(());
    }
    Err("usage: procedure-assets --adoption-map TARGET-VAULT (read-only inventory)".into())
}
