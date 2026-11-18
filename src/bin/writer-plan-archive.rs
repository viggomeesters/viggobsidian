//! Explicit maintenance of completed canonical-writer plans; never a content writer.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const ARCHIVE: &str = "completed-plan-archives-v1";
const LIMIT: u64 = 512 * 1024 * 1024;
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn regular(path: &Path) -> Result<()> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(format!("not a regular file: {}", path.display()).into());
    }
    Ok(())
}
fn directory(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err("absolute path required".into());
    }
    let mut current = PathBuf::new();
    for part in path.components() {
        current.push(part);
        if !matches!(
            part,
            std::path::Component::RootDir | std::path::Component::Normal(_)
        ) || !fs::symlink_metadata(&current)?.file_type().is_dir()
        {
            return Err("directory/symlink guard".into());
        }
    }
    Ok(())
}
fn bytes(path: &Path) -> Result<Vec<u8>> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    let before = file.metadata()?;
    if !before.is_file() || before.len() > LIMIT {
        return Err("plan/receipt exceeds 512 MiB limit".into());
    }
    let mut raw = Vec::new();
    (&mut file).take(LIMIT + 1).read_to_end(&mut raw)?;
    if raw.len() as u64 > LIMIT
        || identity(&before) != identity(&file.metadata()?)
        || identity(&before) != identity(&fs::symlink_metadata(path)?)
    {
        return Err("file changed while reading".into());
    }
    Ok(raw)
}
fn identity(m: &fs::Metadata) -> (u64, u64, u64, i64, i64) {
    (m.dev(), m.ino(), m.len(), m.mtime(), m.mtime_nsec())
}
// Only the exact tool-owned scratch file is discarded, while holding writer.lock.
fn discard_stage(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(m) => {
            regular(path)?;
            let file = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW)
                .open(path)?;
            if identity(&m) != identity(&file.metadata()?)
                || identity(&m) != identity(&fs::symlink_metadata(path)?)
            {
                return Err("stage identity changed".into());
            }
            fs::remove_file(path)?;
            sync(path.parent().ok_or("missing parent")?)?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
fn sync(path: &Path) -> Result<()> {
    File::open(path)?.sync_all()?;
    Ok(())
}
fn run_id(id: &str) -> Result<()> {
    if id.len() != 32
        || !id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid run id".into());
    }
    Ok(())
}
// Python json.dumps(sort_keys=True, ensure_ascii=False), used by this writer.
fn canonical(v: &Value) -> Result<String> {
    Ok(match v {
        Value::Object(m) => {
            let mut keys: Vec<_> = m.keys().collect();
            keys.sort();
            format!(
                "{{{}}}",
                keys.into_iter()
                    .map(|k| Ok(format!(
                        "{}: {}",
                        serde_json::to_string(k)?,
                        canonical(&m[k])?
                    )))
                    .collect::<Result<Vec<_>>>()?
                    .join(", ")
            )
        }
        Value::Array(a) => format!(
            "[{}]",
            a.iter()
                .map(canonical)
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        ),
        Value::Number(n) if !n.is_i64() && !n.is_u64() => {
            return Err("noninteger plan number unsupported".into());
        }
        _ => serde_json::to_string(v)?,
    })
}
fn completed(root: &Path, id: &str, raw: &[u8], receipt: &[u8]) -> Result<String> {
    let mut p: Value = serde_json::from_slice(raw)?;
    let r: Value = serde_json::from_slice(receipt)?;
    let seal = p["sha256"].as_str().ok_or("missing plan seal")?.to_owned();
    if p["version"] != 1
        || p["run_id"] != id
        || p["root"].as_str() != root.to_str()
        || r["run_id"] != id
        || r["plan_sha256"] != seal
        || !matches!(r["status"].as_str(), Some("committed" | "written"))
        || !r["outputs"].is_object()
        || !p["changes"].is_object()
    {
        return Err("unproven completed plan".into());
    }
    let changes = p["changes"].as_object().ok_or("missing changes")?;
    let outputs = r["outputs"].as_object().ok_or("missing outputs")?;
    if changes.len() != outputs.len()
        || changes.iter().any(|(path, change)| {
            !change.is_object()
                || change.get("after").is_none()
                || if change["after"].is_null() {
                    outputs.get(path) != Some(&Value::Null)
                } else {
                    !outputs.get(path).and_then(Value::as_str).is_some_and(|h| {
                        h.len() == 64
                            && h.bytes()
                                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    })
                }
        })
    {
        return Err("incomplete terminal receipt outputs".into());
    }
    p.as_object_mut().ok_or("invalid plan")?.remove("sha256");
    if hash(canonical(&p)?.as_bytes()) != seal {
        return Err("invalid plan seal".into());
    }
    Ok(seal)
}
struct Session {
    root: PathBuf,
    state: PathBuf,
    _lock: File,
}
impl Session {
    fn open(root: &Path, base: &Path) -> Result<Self> {
        directory(root)?;
        directory(base)?;
        regular(&root.join("AGENTS.md"))?;
        let root = root.canonicalize()?;
        let state = base.join(&hash(root.to_str().ok_or("non-UTF8 root")?.as_bytes())[..16]);
        directory(&state)?;
        let lock = state.join("writer.lock");
        if lock.symlink_metadata().is_ok() {
            regular(&lock)?;
        }
        let lock = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .custom_flags(libc::O_NOFOLLOW)
            .mode(0o600)
            .open(lock)?;
        lock.try_lock().map_err(|_| "writer_busy")?;
        if identity(&lock.metadata()?)
            != identity(&fs::symlink_metadata(state.join("writer.lock"))?)
        {
            return Err("writer lock identity changed".into());
        }
        if fs::read_dir(&state)?.any(|e| {
            e.map(|e| e.file_name().to_string_lossy().ends_with(".journal.json"))
                .unwrap_or(true)
        }) {
            return Err("recovery_required".into());
        }
        Ok(Self {
            root,
            state,
            _lock: lock,
        })
    }
    fn path(&self, id: &str, suffix: &str) -> PathBuf {
        self.state.join(format!("{id}.{suffix}"))
    }
    fn eligible(&self, id: &str, days: u64) -> Result<(Vec<u8>, Vec<u8>)> {
        run_id(id)?;
        // An unfinished structural directory ledger remains recovery authority.
        if self.path(id, "layout-dirs.json").symlink_metadata().is_ok() {
            return Err("layout_recovery_state_preserved".into());
        }
        let plan = self.path(id, "plan.json");
        let receipt = self.path(id, "receipt.json");
        for path in [&plan, &receipt] {
            regular(path)?;
            if SystemTime::now().duration_since(fs::metadata(path)?.modified()?)?
                < Duration::from_secs(days.checked_mul(86400).ok_or("age overflow")?)
            {
                return Err("retention_window".into());
            }
        }
        let raw = bytes(&plan)?;
        let receipt = bytes(&receipt)?;
        completed(&self.root, id, &raw, &receipt)?;
        Ok((raw, receipt))
    }
    fn archive_dir(&self) -> Result<PathBuf> {
        let path = self.state.join(ARCHIVE);
        if !path.exists() && !path.is_symlink() {
            fs::create_dir(&path)?;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
            sync(&self.state)?;
        }
        directory(&path)?;
        Ok(path)
    }
    fn preview(&self, days: u64) -> Result<Value> {
        let mut accepted = Vec::new();
        let mut skipped = Vec::new();
        let mut total = 0u64;
        for entry in fs::read_dir(&self.state)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(id) = name.strip_suffix(".plan.json") {
                match self.eligible(id, days) {
                    Ok((raw, _)) => {
                        total += raw.len() as u64;
                        accepted.push(json!({"run_id":id,"bytes":raw.len()}));
                    }
                    Err(e) => skipped.push(json!({"run_id":id,"reason":e.to_string()})),
                }
            }
        }
        accepted.sort_by(|a, b| a["run_id"].as_str().cmp(&b["run_id"].as_str()));
        Ok(
            json!({"schema":1,"root":self.root,"state":self.state,"min_age_days":days,"eligible":accepted,"eligible_bytes":total,"skipped":skipped}),
        )
    }
    fn verify_archive(&self, id: &str) -> Result<Vec<u8>> {
        run_id(id)?;
        let dir = self.state.join(ARCHIVE);
        directory(&dir)?;
        let manifest: Value = serde_json::from_slice(&bytes(&dir.join(format!("{id}.json")))?)?;
        let packed = dir.join(format!("{id}.plan.json.gz"));
        regular(&packed)?;
        let output = Command::new("/usr/bin/gzip")
            .args(["-dc"])
            .arg(&packed)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut output = output;
        let mut raw = Vec::new();
        output
            .stdout
            .take()
            .ok_or("gzip stdout missing")?
            .take(LIMIT + 1)
            .read_to_end(&mut raw)?;
        if raw.len() as u64 > LIMIT {
            output.kill()?;
            let _ = output.wait();
            return Err("archive decompression limit".into());
        }
        if !output.wait()?.success() {
            return Err("archive corrupt".into());
        }
        let receipt = bytes(&self.path(id, "receipt.json"))?;
        if manifest["schema"] != 1
            || manifest["run_id"] != id
            || manifest["root"].as_str() != self.root.to_str()
            || manifest["bytes"] != raw.len() as u64
            || manifest["raw_sha256"] != hash(&raw)
            || manifest["receipt_sha256"] != hash(&receipt)
            || manifest["plan_sha256"] != completed(&self.root, id, &raw, &receipt)?
        {
            return Err("archive integrity mismatch".into());
        }
        Ok(raw)
    }
    fn archive(&self, id: &str, days: u64) -> Result<Value> {
        let (raw, receipt) = self.eligible(id, days)?;
        let dir = self.archive_dir()?;
        let packed = dir.join(format!("{id}.plan.json.gz"));
        let meta = dir.join(format!("{id}.json"));
        if !packed.exists() && !packed.is_symlink() {
            let stage = dir.join(format!(".{id}.gzip-stage"));
            discard_stage(&stage)?;
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&stage)?;
            // gzip reads the regular original directly; no pipe deadlock or shell expansion.
            let status = Command::new("/usr/bin/gzip")
                .args(["-cn"])
                .arg(self.path(id, "plan.json"))
                .stdout(Stdio::from(file))
                .status();
            match status {
                Ok(s) if s.success() => {}
                _ => {
                    let _ = fs::remove_file(&stage);
                    return Err("gzip failed".into());
                }
            }
            sync(&stage)?;
            fs::hard_link(&stage, &packed)?;
            sync(&dir)?;
            fs::remove_file(&stage)?;
        }
        if !meta.exists() && !meta.is_symlink() {
            let manifest = json!({"schema":1,"root":self.root,"run_id":id,"bytes":raw.len(),"raw_sha256":hash(&raw),"receipt_sha256":hash(&receipt),"plan_sha256":completed(&self.root,id,&raw,&receipt)?});
            publish(&meta, &serde_json::to_vec(&manifest)?)?;
        }
        if self.verify_archive(id)? != raw || bytes(&self.path(id, "plan.json"))? != raw {
            return Err("archive readback mismatch".into());
        }
        // Durable verified archive precedes unlink. Crash leaves either both copies or archive.
        let source = self.path(id, "plan.json");
        let before = fs::symlink_metadata(&source)?;
        if bytes(&source)? != raw || identity(&before) != identity(&fs::symlink_metadata(&source)?)
        {
            return Err("plan changed before unlink".into());
        }
        fs::remove_file(source)?;
        sync(&self.state)?;
        Ok(
            json!({"run_id":id,"original_bytes":raw.len(),"archive_bytes":fs::metadata(packed)?.len(),"status":"archived"}),
        )
    }
    fn restore(&self, id: &str) -> Result<Value> {
        if self.path(id, "layout-dirs.json").symlink_metadata().is_ok() {
            return Err("layout_recovery_state_preserved".into());
        }
        let raw = self.verify_archive(id)?;
        publish(&self.path(id, "plan.json"), &raw)?;
        if bytes(&self.path(id, "plan.json"))? != raw {
            return Err("restore readback mismatch".into());
        }
        Ok(json!({"run_id":id,"status":"restored","bytes":raw.len()}))
    }
}
fn publish(path: &Path, raw: &[u8]) -> Result<()> {
    let stage = path.with_extension("publish-stage");
    discard_stage(&stage)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&stage)?;
    file.write_all(raw)?;
    file.sync_all()?;
    let result = fs::hard_link(&stage, path); // atomic no-replace publication
    if let Err(e) = result {
        fs::remove_file(&stage)?;
        return Err(e.into());
    }
    sync(path.parent().ok_or("missing parent")?)?;
    fs::remove_file(stage)?;
    Ok(())
}
fn run() -> Result<Value> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        return Err("usage: writer-plan-archive preview|archive|restore ROOT STATE_BASE [DAYS|RUN_ID]; archive accepts DAYS (default 14), restore RUN_ID; STATE_BASE must be the canonical writer's actual configured state base".into());
    }
    if args.len() > 4 {
        return Err("too many arguments".into());
    }
    let session = Session::open(Path::new(&args[1]), Path::new(&args[2]))?;
    match args[0].as_str() {
        "preview" => {
            session._lock.unlock()?;
            session.preview(args.get(3).map(|s| s.parse()).transpose()?.unwrap_or(14))
        }
        "archive" => {
            let days = args.get(3).map(|s| s.parse()).transpose()?.unwrap_or(14);
            session._lock.unlock()?;
            let preview = session.preview(days)?;
            let mut results = Vec::new();
            for row in preview["eligible"].as_array().ok_or("invalid preview")? {
                let locked = Session::open(&session.root, Path::new(&args[2]))?;
                let result =
                    locked.archive(row["run_id"].as_str().ok_or("invalid run id")?, days)?;
                drop(locked);
                eprintln!("{result}");
                results.push(result);
            }
            Ok(
                json!({"status":"completed","root":session.root,"results":results,"skipped":preview["skipped"]}),
            )
        }
        "restore" => session.restore(args.get(3).ok_or("run id required")?),
        _ => Err("unknown command".into()),
    }
}
fn main() {
    match run() {
        Ok(v) => println!("{v}"),
        Err(e) => {
            eprintln!("{}", json!({"status":"refused","reason":e.to_string()}));
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    const ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    #[test]
    fn python_json_dumps_golden() {
        // Independent Python stdlib oracle, not generated by this canonicalizer.
        let value = json!({"z":[null,true,false,-123,18446744073709551615u64],"a":"ë 😀 \" slash/ backslash\\ tab\t newline\n control\u{1}","nested":{"Ω":"x","A":"y"}});
        assert_eq!(
            hash(canonical(&value).unwrap().as_bytes()),
            "971d143d1a77f518b8fc0ea25dfa4a9e3977ba9959b365745967e0ea4738182e"
        );
    }
    struct Fixture {
        home: PathBuf,
        root: PathBuf,
        base: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            // macOS /tmp may be a symlink: fixtures use its canonical location.
            let home = std::env::temp_dir().canonicalize().unwrap().join(format!(
                "writer-archive-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&home).unwrap();
            let root = home.join("vault");
            let base = home.join("state");
            fs::create_dir(&root).unwrap();
            fs::create_dir(&base).unwrap();
            fs::write(root.join("AGENTS.md"), "fixture").unwrap();
            fs::create_dir(base.join(&hash(root.to_str().unwrap().as_bytes())[..16])).unwrap();
            Self { home, root, base }
        }
        fn session(&self) -> Session {
            Session::open(&self.root, &self.base).unwrap()
        }
        fn plan(&self, s: &Session) -> Vec<u8> {
            let mut p = json!({"version":1,"root":self.root,"run_id":ID,"changes":{"fictional.md":{"before":null,"after":"Zm9v"}},"goal":"ë 😀","warnings":[]});
            let seal = hash(canonical(&p).unwrap().as_bytes());
            p["sha256"] = json!(seal);
            let raw = canonical(&p).unwrap().into_bytes();
            fs::write(s.path(ID, "plan.json"), &raw).unwrap();
            fs::write(
                s.path(ID, "receipt.json"),
                serde_json::to_vec(
                    &json!({"status":"written","run_id":ID,"plan_sha256":seal,"outputs":{"fictional.md":hash(b"foo")}}),
                )
                .unwrap(),
            )
            .unwrap();
            raw
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.home).unwrap();
        }
    }
    #[test]
    fn scratch_crashes_retry_safely() {
        let f = Fixture::new();
        let s = f.session();
        let raw = f.plan(&s);
        let dir = s.archive_dir().unwrap();
        fs::write(dir.join(format!(".{ID}.gzip-stage")), b"partial").unwrap();
        fs::write(dir.join(format!("{ID}.publish-stage")), b"partial metadata").unwrap();
        s.archive(ID, 0).unwrap();
        fs::write(s.path(ID, "plan.publish-stage"), b"partial restore").unwrap();
        s.restore(ID).unwrap();
        assert_eq!(bytes(&s.path(ID, "plan.json")).unwrap(), raw);
        // A crash after hard-link publication leaves a completed target and scratch.
        fs::hard_link(s.path(ID, "plan.json"), s.path(ID, "plan.publish-stage")).unwrap();
        assert!(s.restore(ID).is_err());
        assert_eq!(bytes(&s.path(ID, "plan.json")).unwrap(), raw);
        assert!(!s.path(ID, "plan.publish-stage").exists());
    }
    #[test]
    fn incomplete_receipt_is_not_terminal_proof() {
        let f = Fixture::new();
        let s = f.session();
        f.plan(&s);
        let path = s.path(ID, "receipt.json");
        let mut r: Value = serde_json::from_slice(&bytes(&path).unwrap()).unwrap();
        r["outputs"] = json!({});
        fs::write(path, serde_json::to_vec(&r).unwrap()).unwrap();
        assert!(s.archive(ID, 0).is_err());
        assert!(s.path(ID, "plan.json").exists());
    }
    #[test]
    fn roundtrip_and_no_clobber() {
        let f = Fixture::new();
        let s = f.session();
        let raw = f.plan(&s);
        let receipt = bytes(&s.path(ID, "receipt.json")).unwrap();
        assert_eq!(
            s.preview(0).unwrap()["eligible"].as_array().unwrap().len(),
            1
        );
        s.archive(ID, 0).unwrap();
        assert!(!s.path(ID, "plan.json").exists());
        assert_eq!(bytes(&s.path(ID, "receipt.json")).unwrap(), receipt);
        s.restore(ID).unwrap();
        assert_eq!(bytes(&s.path(ID, "plan.json")).unwrap(), raw);
        assert!(s.restore(ID).is_err());
        s.archive(ID, 0).unwrap();
        assert_eq!(s.verify_archive(ID).unwrap(), raw);
    }
    #[test]
    fn preserves_unfinished_and_young() {
        let f = Fixture::new();
        let s = f.session();
        f.plan(&s);
        assert!(s.archive(ID, 14).is_err());
        fs::write(s.path(ID, "layout-dirs.json"), "{}").unwrap();
        assert!(s.archive(ID, 0).is_err());
        fs::remove_file(s.path(ID, "layout-dirs.json")).unwrap();
        fs::remove_file(s.path(ID, "receipt.json")).unwrap();
        assert!(s.archive(ID, 0).is_err());
        assert!(s.path(ID, "plan.json").exists());
    }
    #[test]
    fn lock_matches_native_flock() {
        use std::os::fd::AsRawFd;
        unsafe extern "C" {
            fn flock(fd: i32, operation: i32) -> i32;
        }
        let f = Fixture::new();
        let s = f.session();
        let other = File::open(s.state.join("writer.lock")).unwrap();
        // LOCK_EX | LOCK_NB is 2 | 4 on both Darwin and Linux (Python fcntl writer).
        assert_eq!(unsafe { flock(other.as_raw_fd(), 2 | 4) }, -1);
        assert!(Session::open(&f.root, &f.base).is_err());
        drop(s);
        assert_eq!(unsafe { flock(other.as_raw_fd(), 2 | 4) }, 0);
        assert!(Session::open(&f.root, &f.base).is_err());
        drop(other);
        let s = f.session();
        fs::write(s.path(ID, "journal.json"), "{}").unwrap();
        drop(s);
        assert!(Session::open(&f.root, &f.base).is_err());
    }
    #[test]
    fn corrupt_archive_and_changed_receipt_refuse_restore() {
        let f = Fixture::new();
        let s = f.session();
        let raw = f.plan(&s);
        s.archive(ID, 0).unwrap();
        let receipt = bytes(&s.path(ID, "receipt.json")).unwrap();
        fs::write(s.path(ID, "receipt.json"), b"{}").unwrap();
        assert!(s.restore(ID).is_err());
        fs::write(s.path(ID, "receipt.json"), receipt).unwrap();
        fs::write(
            s.state.join(ARCHIVE).join(format!("{ID}.plan.json.gz")),
            b"broken",
        )
        .unwrap();
        assert!(s.restore(ID).is_err());
        assert!(!s.path(ID, "plan.json").exists());
        assert!(!raw.is_empty());
    }
    #[test]
    fn crash_after_durable_archive_retries() {
        let f = Fixture::new();
        let s = f.session();
        let raw = f.plan(&s);
        s.archive(ID, 0).unwrap();
        // Reconstruct the crash state immediately before original unlink.
        fs::write(s.path(ID, "plan.json"), &raw).unwrap();
        s.archive(ID, 0).unwrap();
        assert_eq!(s.verify_archive(ID).unwrap(), raw);
    }
    #[test]
    fn tampered_plan_and_symlink_rejected() {
        let f = Fixture::new();
        let s = f.session();
        f.plan(&s);
        let mut p: Value =
            serde_json::from_slice(&bytes(&s.path(ID, "plan.json")).unwrap()).unwrap();
        p["goal"] = json!("tampered");
        fs::write(s.path(ID, "plan.json"), serde_json::to_vec(&p).unwrap()).unwrap();
        assert!(s.archive(ID, 0).is_err());
        fs::remove_file(s.path(ID, "plan.json")).unwrap();
        std::os::unix::fs::symlink(f.root.join("AGENTS.md"), s.path(ID, "plan.json")).unwrap();
        assert!(s.archive(ID, 0).is_err());
        assert!(s.restore("../escape").is_err());
    }
}
