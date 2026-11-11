use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const MANIFEST: &str = ".mail-source.json";
pub const MAIL_SCHEMA: &str = "viggo-email-ingestion.package.v1";
pub const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
pub fn hash_file(path: &Path) -> Result<(String, u64)> {
    let mut file = File::open(path)?;
    let before = file.metadata()?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 32 * 1024];
    loop {
        let bytes = file.read(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        hasher.update(&buffer[..bytes]);
    }
    let after = fs::symlink_metadata(path)?;
    if !after.is_file() || before.len() != after.len() || before.modified()? != after.modified()? {
        return Err(format!("file changed while hashing: {}", path.display()).into());
    }
    Ok((format!("{:x}", hasher.finalize()), before.len()))
}

pub fn manifest_member(name: &str) -> Result<&Path> {
    let path = Path::new(name);
    if path.as_os_str().is_empty()
        || name.contains('\\')
        || name.contains("//")
        || name.ends_with('/')
        || name.split('/').any(|part| part == "." || part == "..")
        || !path.components().all(|c| matches!(c, Component::Normal(_)))
        || path == Path::new(MANIFEST)
    {
        return Err(format!("invalid mail manifest member: {name}").into());
    }
    Ok(path)
}
pub fn read_manifest(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err("mail manifest exceeds size limit".into());
    }
    Ok(bytes)
}

pub fn verify_manifest(
    bytes: &[u8],
    relative: &Path,
    expected_note: Option<&Path>,
    mut lookup: impl FnMut(&Path) -> Result<Option<(String, u64)>>,
) -> Result<()> {
    let value: Value = serde_json::from_slice(bytes)?;
    if value["schema"].as_str() != Some(MAIL_SCHEMA) {
        return Err(format!("unknown mail manifest schema: {}", relative.display()).into());
    }
    let parent = relative.parent().ok_or("manifest parent missing")?;
    let files = value["files"]
        .as_object()
        .ok_or("mail manifest files missing")?;
    let eml = value["eml"].as_str().ok_or("mail manifest EML missing")?;
    let note = value["capture_foldernote"]
        .as_str()
        .ok_or("mail manifest capture note missing")?;
    if let Some(expected) = expected_note {
        if Path::new(note) != expected {
            return Err("mail manifest must reference the package's matching folder note".into());
        }
        if Path::new(eml) != expected.with_extension("eml") {
            return Err("mail manifest must reference the package's same-name EML".into());
        }
    }
    if files.get(eml).and_then(Value::as_str) != value["raw_sha256"].as_str()
        || value["raw_sha256"].as_str().is_none()
        || manifest_member(eml)?
            .extension()
            .is_none_or(|ext| ext != "eml")
        || manifest_member(note)?
            .extension()
            .is_none_or(|ext| ext != "md")
        || lookup(&parent.join(manifest_member(note)?))?.is_none()
    {
        return Err(format!(
            "mail manifest EML or capture note invalid: {}",
            relative.display()
        )
        .into());
    }
    let mut verified = BTreeMap::new();
    for (name, expected) in files {
        let member = parent.join(manifest_member(name)?);
        let actual = lookup(&member)?.filter(|(hash, _)| Some(hash.as_str()) == expected.as_str());
        let Some(actual) = actual else {
            return Err(format!("mail manifest hash mismatch: {}", member.display()).into());
        };
        verified.insert(member, actual);
    }
    let attachments = value["attachments"]
        .as_array()
        .ok_or("mail manifest attachments missing")?;
    let mut listed = BTreeSet::new();
    for attachment in attachments {
        let name = attachment["file"]
            .as_str()
            .ok_or("attachment file missing")?;
        let member = parent.join(manifest_member(name)?);
        let (hash, size) = verified
            .get(&member)
            .ok_or("attachment absent from files")?;
        if !listed.insert(name)
            || attachment["sha256"].as_str() != Some(hash.as_str())
            || attachment["bytes"].as_u64() != Some(*size)
        {
            return Err(format!("mail attachment metadata mismatch: {}", member.display()).into());
        }
    }
    if files
        .keys()
        .any(|name| name.starts_with("attachments/") && !listed.contains(name.as_str()))
    {
        return Err("mail manifest omits an attachment entry".into());
    }
    Ok(())
}
