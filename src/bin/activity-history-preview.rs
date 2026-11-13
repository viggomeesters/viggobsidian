//! Read-only, hash-bound formatting migration. Never installs candidates.
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{self, Read},
};

fn run() -> Result<String, String> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        return Err("usage: activity-history-preview SOURCE EXPECTED_SHA256 < request.json".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(&args[1])
        .map_err(|e| e.to_string())?
        .take(activity_core::MAX_NOTE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > activity_core::MAX_NOTE_BYTES {
        return Err("source exceeds note limit".into());
    }
    let hash = format!("{:x}", Sha256::digest(&bytes));
    if hash != args[2] {
        return Err("source preimage changed".into());
    }
    let source = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
    let mut input = String::new();
    io::stdin()
        .take(activity_core::MAX_REQUEST_BYTES as u64 + 1)
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    if input.len() > activity_core::MAX_REQUEST_BYTES {
        return Err("request exceeds limit".into());
    }
    let request = serde_json::from_str(&input).map_err(|e| e.to_string())?;
    activity_core::history_migration::preview(source, &request).map_err(str::to_owned)
}

fn main() {
    match run() {
        Ok(candidate) => print!("{candidate}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
