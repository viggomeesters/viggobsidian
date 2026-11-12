//! Explicit immutable payloads: validate bytes and format; never waive source events.
use crate::{iso_date, read_metadata, text};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Component, Path, PathBuf},
};

pub(crate) fn check(
    package: &Path,
    metadata: &Value,
) -> io::Result<(BTreeSet<PathBuf>, Vec<String>)> {
    let Some(declaration) = metadata.get("export_artifacts") else {
        return Ok((BTreeSet::new(), vec![]));
    };
    let mut valid = BTreeSet::new();
    let result = (|| -> Result<(), String> {
        if declaration.as_str() != Some("source-export.json") {
            return Err("export_artifacts must name source-export.json".into());
        }
        let manifest_path = package.join("source-export.json");
        regular(&manifest_path)?;
        let manifest: Value =
            serde_json::from_slice(&fs::read(&manifest_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if manifest["schema"] != "viggobsidian.source-export.v1" {
            return Err("unknown source export schema".into());
        }
        let stem = package
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("invalid package name")?;
        if manifest["package_note"] != format!("{stem}.md") {
            return Err("export manifest must bind its owning package note".into());
        }
        let consumer = manifest["consumer"]
            .as_str()
            .ok_or("consumer identity required")?;
        if consumer.is_empty()
            || !consumer
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        {
            return Err("invalid consumer identity".into());
        }
        let entries = manifest["artifacts"]
            .as_array()
            .ok_or("artifacts array required")?;
        if entries.is_empty() {
            return Err("empty artifact inventory".into());
        }
        for item in entries {
            let relative = item["path"].as_str().ok_or("artifact path required")?;
            let mut parts = Path::new(relative).components();
            if !matches!(parts.next(), Some(Component::Normal(_)))
                || parts.next().is_some()
                || relative.starts_with('.')
                || !relative.ends_with(".md")
                || relative == format!("{stem}.md")
            {
                return Err("artifact must be a direct secondary Markdown file".into());
            }
            let path = package.join(relative);
            if !valid.insert(path.clone()) {
                return Err("duplicate artifact path".into());
            }
            regular(&path)?;
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            if item["bytes"].as_u64() != Some(bytes.len() as u64)
                || item["sha256"].as_str() != Some(format!("{:x}", Sha256::digest(&bytes)).as_str())
            {
                return Err(format!("artifact bytes changed: {relative}"));
            }
            let (data, body) = read_metadata(&path)
                .map_err(|e| e.to_string())?
                .ok_or("artifact frontmatter required")?;
            if ["task-id", "task_id", "wiki_status", "layer"]
                .iter()
                .any(|key| data.get(*key).is_some())
            {
                return Err(
                    "operational task, Wiki and processing fields cannot be export artifacts"
                        .into(),
                );
            }
            match item["format"].as_str() {
                Some("migration-records-v1") => records(&data, &body)?,
                Some("derived-month-summary-v1") => summary(&data, &body)?,
                Some("reported-health-log-v1") => health_log(&data, &body)?,
                _ => return Err(format!("unsupported artifact format: {relative}")),
            }
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok((valid, vec![])),
        Err(e) => Ok((BTreeSet::new(), vec![e])),
    }
}
fn regular(path: &Path) -> Result<(), String> {
    let m = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !m.is_file() || m.is_symlink() {
        return Err("export member must be a regular file".into());
    }
    Ok(())
}
fn records(data: &Value, body: &str) -> Result<(), String> {
    if text(data, "type") != Some("migration-collection")
        || text(data, "mapping_version").is_none_or(str::is_empty)
        || text(data, "source_set").is_none_or(str::is_empty)
    {
        return Err("record export requires migration-collection provenance".into());
    }
    let lines = body.lines().collect::<Vec<_>>();
    let mut i = 0;
    let next = |i: &mut usize| -> Option<&str> {
        while lines.get(*i).is_some_and(|s| s.is_empty()) {
            *i += 1;
        }
        let line = lines.get(*i).copied();
        *i += usize::from(line.is_some());
        line
    };
    if !next(&mut i).is_some_and(|l| {
        l.starts_with("# ") && !l[2..].trim().is_empty() && !l.contains(['<', '>'])
    }) {
        return Err("record export requires one title".into());
    }
    let mut ids = BTreeSet::new();
    let mut anchors = BTreeSet::new();
    while let Some(line) = next(&mut i) {
        let heading = line
            .strip_prefix("## ")
            .filter(|h| !h.trim().is_empty() && !h.contains(['<', '>']))
            .ok_or("only complete records may follow the export title")?;
        let identity = next(&mut i).ok_or("record identity required")?;
        let (id, tail) = identity
            .strip_prefix("Bron-ID: `")
            .and_then(|s| s.split_once('`'))
            .filter(|(id, _)| !id.is_empty())
            .ok_or("literal source ID required")?;
        if tail.contains(['<', '>']) {
            return Err("record identity cannot hide document anchors".into());
        }
        let anchor = tail
            .rsplit_once(" ^record-")
            .map(|(_, a)| a)
            .ok_or("record anchor required")?;
        if anchor.len() != 16
            || !anchor.bytes().all(|b| b.is_ascii_hexdigit())
            || !anchors.insert(anchor.to_owned())
        {
            return Err("unique record anchor required".into());
        }
        if next(&mut i) != Some("### Geïmporteerde velden") || next(&mut i) != Some("```yaml") {
            return Err("record fields must be an unfenced YAML export block".into());
        }
        let mut yaml = String::new();
        let mut closed = false;
        while let Some(line) = lines.get(i) {
            i += 1;
            if *line == "```" {
                closed = true;
                break;
            }
            yaml.push_str(line);
            yaml.push('\n');
        }
        if !closed {
            return Err("unclosed YAML record".into());
        }
        let v: Value = serde_saphyr::from_str(&yaml).map_err(|e| e.to_string())?;
        if text(&v, "id") != Some(id)
            || !(heading == id || text(&v, "title") == Some(heading))
            || !ids.insert(id.to_owned())
        {
            return Err("YAML ID, heading and unique anchor must bind one record".into());
        }
        let mut look = i;
        if next(&mut look) == Some("### Verwijzingen") {
            i = look;
            let mut count = 0;
            loop {
                let mut look = i;
                let Some(line) = next(&mut look) else {
                    break;
                };
                if line.starts_with("## ") {
                    break;
                }
                reference(line)?;
                i = look;
                count += 1;
            }
            if count == 0 {
                return Err("reference section must contain source links".into());
            }
        }
    }
    if ids.is_empty() {
        return Err("record export must contain anchored records".into());
    }
    Ok(())
}
fn reference(line: &str) -> Result<(), String> {
    let (id, locator) = line
        .strip_prefix("- [")
        .and_then(|s| s.strip_suffix(')'))
        .and_then(|s| s.split_once("]("))
        .ok_or("export reference must be a single source link")?;
    if id.is_empty()
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err("invalid reference source ID".into());
    }
    let (path, anchor) = locator
        .strip_prefix("../")
        .and_then(|s| s.split_once("#^record-"))
        .ok_or("reference must address an anchored sibling Raw export")?;
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() != 2
        || !crate::package_name(parts[0])
        || parts[1].starts_with('.')
        || !parts[1].ends_with(".md")
        || parts[1].contains(['\\', '#', '[', ']'])
        || anchor.len() != 16
        || !anchor.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("invalid sibling export reference".into());
    }
    Ok(())
}
fn summary(data: &Value, body: &str) -> Result<(), String> {
    let period = text(data, "period").unwrap_or("");
    if text(data, "type") != Some("migraine-derived-summary")
        || text(data, "status") != Some("current-derived")
        || !iso_date(&format!("{period}-01"))
        || text(data, "derived_from") != Some("health_migraine_daily")
        || text(data, "source_summary")
            != Some(format!("health.migraine.monthly.{period}").as_str())
        || !text(data, "derived_at").is_some_and(iso_date)
    {
        return Err("derived summary provenance must bind its month and source".into());
    }
    let lines = body.lines().filter(|l| !l.is_empty()).collect::<Vec<_>>();
    let prefix=[format!("# Migraine-overzicht {period}"),"Afgeleid uit dagelijkse bronregistraties; geen medische interpretatie en geen invulling van ontbrekende dagen.".into(),"| Maat | Waarde |".into(),"| --- | ---: |".into()];
    if lines.len() < 13 || !prefix.iter().zip(&lines).all(|(a, b)| a == b) {
        return Err("derived summary requires its exporter title and metric table".into());
    }
    let keys = [
        "Aanwezige dagregistraties",
        "Unieke kalenderdagen",
        "Ontbrekende kalenderdagen",
        "`headache: true`",
        "`migraine_logged: true`",
        "`medication_used: true`",
        "Historische `reported_positive_days`",
    ];
    for (line, key) in lines[4..11].iter().zip(keys) {
        let cells = line
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cells.len() != 2 || cells[0] != key || cells[1].parse::<u64>().is_err() {
            return Err("derived metrics must be the complete numeric exporter table".into());
        }
    }
    if lines[11]
        != "De historische samenvatting blijft bronhistorie. Verschillen met deze afleiding zijn zichtbaar, niet stil gecorrigeerd."
        || lines[12] != "## Dagbronnen"
    {
        return Err("derived summary requires its real daily-source section".into());
    }
    let mut days = BTreeSet::new();
    for line in &lines[13..] {
        let day = line
            .strip_prefix("- `health.migraine.daily.")
            .and_then(|s| s.strip_suffix('`'))
            .ok_or("only daily source IDs may follow the metric table")?;
        if !iso_date(day) || !day.starts_with(&format!("{period}-")) || !days.insert(day) {
            return Err("daily source IDs must be unique and in the summary month".into());
        }
    }
    Ok(())
}
pub(crate) fn timestamp(s: &str) -> bool {
    let Some(date) = s.get(..10) else {
        return false;
    };
    if !iso_date(date) || s.as_bytes().get(10) != Some(&b'T') {
        return false;
    }
    let Some(t) = s.get(11..) else {
        return false;
    };
    let Some(clock) = t.get(..8) else {
        return false;
    };
    let parts = clock.split(':').collect::<Vec<_>>();
    if parts.len() != 3
        || parts
            .iter()
            .any(|p| p.len() != 2 || !p.bytes().all(|b| b.is_ascii_digit()))
        || parts[0].parse::<u8>().map_or(true, |n| n > 23)
        || parts[1..]
            .iter()
            .any(|p| p.parse::<u8>().map_or(true, |n| n > 59))
    {
        return false;
    }
    let mut zone = &t[8..];
    if let Some(fraction) = zone.strip_prefix('.') {
        let n = fraction.bytes().take_while(|b| b.is_ascii_digit()).count();
        if n == 0 {
            return false;
        }
        zone = &fraction[n..];
    }
    if zone == "Z" {
        return true;
    }
    let b = zone.as_bytes();
    if b.len() != 6 || !matches!(b[0], b'+' | b'-') || b[3] != b':' {
        return false;
    }
    if ![b[1], b[2], b[4], b[5]].iter().all(u8::is_ascii_digit) {
        return false;
    }
    zone.get(1..3)
        .and_then(|s| s.parse::<u8>().ok())
        .is_some_and(|n| n <= 23)
        && zone
            .get(4..6)
            .and_then(|s| s.parse::<u8>().ok())
            .is_some_and(|n| n <= 59)
}
fn health_log(data: &Value, body: &str) -> Result<(), String> {
    if text(data, "type") != Some("health-log")
        || text(data, "status") != Some("reported")
        || !text(data, "date").is_some_and(iso_date)
        || text(data, "person").is_none_or(str::is_empty)
        || text(data, "source").is_none_or(str::is_empty)
        || !text(data, "captured_at").is_some_and(timestamp)
    {
        return Err("reported health log requires dated original input provenance".into());
    }
    let lines = body.lines().filter(|l| !l.is_empty()).collect::<Vec<_>>();
    if !lines.first().is_some_and(|s| {
        s.starts_with("# ") && !s[2..].trim().is_empty() && !s.contains(['<', '>'])
    }) {
        return Err("health input title required".into());
    }
    let a = lines
        .iter()
        .position(|l| *l == "## Oorspronkelijke aangeleverde tekst")
        .ok_or("original input section required")?;
    let b = lines
        .iter()
        .position(|l| *l == "## Bron en afbakening")
        .ok_or("source boundary section required")?;
    if a <= 1
        || b <= a + 1
        || b + 1 >= lines.len()
        || lines[1..a]
            .iter()
            .any(|l| !l.starts_with("- **") || l.contains(['<', '>']))
        || lines[a + 1..b].iter().any(|l| !l.starts_with("> "))
        || lines[b + 1..].iter().any(|l| {
            l.trim_start() != *l
                || !l.chars().next().is_some_and(char::is_alphabetic)
                || l.contains(['<', '>'])
        })
    {
        return Err("health input must keep summary bullets, quoted original and plain source boundary in order".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    const PAYLOAD: &str = "---\ntype: migration-collection\nmapping_version: test-1\nsource_set: fictional\n---\n\n# Export\n\n## sample.daily.2020-01-01\n\nBron-ID: `sample.daily.2020-01-01` · reconstruct · accepted ^record-0123456789abcdef\n\n### Geïmporteerde velden\n\n```yaml\nid: sample.daily.2020-01-01\nvalue: 2\n```\n";
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let p = std::env::temp_dir()
                .join(format!(
                    "export-contract-{}-{}",
                    std::process::id(),
                    SERIAL.fetch_add(1, Ordering::Relaxed)
                ))
                .join("20200101-export");
            fs::create_dir_all(&p).unwrap();
            fs::write(p.join("data.md"), PAYLOAD).unwrap();
            let f = Self(p);
            f.manifest("data.md", "migration-records-v1");
            f
        }
        fn manifest(&self, path: &str, format: &str) {
            let bytes = fs::read(self.0.join("data.md")).unwrap();
            fs::write(self.0.join("source-export.json"),serde_json::to_vec(&json!({"schema":"viggobsidian.source-export.v1","package_note":"20200101-export.md","consumer":"fictional-importer","artifacts":[{"path":path,"bytes":bytes.len(),"sha256":format!("{:x}",Sha256::digest(bytes)),"format":format}]})).unwrap()).unwrap();
        }
        fn check(&self) -> (BTreeSet<PathBuf>, Vec<String>) {
            check(&self.0, &json!({"export_artifacts":"source-export.json"})).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(self.0.parent().unwrap()).unwrap();
        }
    }
    #[test]
    fn exact_records_are_accepted_without_changing_bytes() {
        let f = Fixture::new();
        let before = fs::read(f.0.join("data.md")).unwrap();
        let (valid, errors) = f.check();
        assert!(errors.is_empty(), "{errors:?}");
        assert!(valid.contains(&f.0.join("data.md")));
        assert_eq!(before, fs::read(f.0.join("data.md")).unwrap());
        assert!(!valid.contains(&f.0.join("ordinary.md")));
    }
    #[test]
    fn undeclared_payloads_get_no_special_treatment() {
        let f = Fixture::new();
        let (v, e) = check(&f.0, &json!({})).unwrap();
        assert!(v.is_empty() && e.is_empty());
    }
    #[test]
    fn drift_and_missing_bytes_fail_closed() {
        let f = Fixture::new();
        fs::write(f.0.join("data.md"), format!("{PAYLOAD}\n")).unwrap();
        let (v, e) = f.check();
        assert!(v.is_empty() && !e.is_empty());
        fs::remove_file(f.0.join("data.md")).unwrap();
        assert!(!f.check().1.is_empty());
    }
    #[test]
    fn arbitrary_notes_tasks_and_unknown_formats_are_rejected() {
        for original in ["note", "task", "wiki"] {
            let f = Fixture::new();
            fs::write(
                f.0.join("data.md"),
                PAYLOAD.replace("migration-collection", original),
            )
            .unwrap();
            f.manifest("data.md", "migration-records-v1");
            assert!(!f.check().1.is_empty());
        }
        let f = Fixture::new();
        f.manifest("data.md", "any-markdown");
        assert!(!f.check().1.is_empty());
    }
    #[test]
    fn traversal_duplicate_paths_and_head_are_rejected() {
        for path in [
            "../data.md",
            "sub/data.md",
            ".data.md",
            "20200101-export.md",
        ] {
            let f = Fixture::new();
            f.manifest(path, "migration-records-v1");
            assert!(!f.check().1.is_empty());
        }
        let f = Fixture::new();
        let p = f.0.join("source-export.json");
        let mut v: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
        let item = v["artifacts"][0].clone();
        v["artifacts"].as_array_mut().unwrap().push(item);
        fs::write(p, serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(f.check().0.is_empty());
    }
    #[test]
    fn malformed_or_unbound_records_are_rejected() {
        for text in [
            PAYLOAD.replace("id: sample.daily.2020-01-01", "id: another"),
            PAYLOAD.replace("^record-0123456789abcdef", ""),
            PAYLOAD.replace("value: 2", "value: ["),
            format!("{PAYLOAD}\n## dangling\n"),
        ] {
            let f = Fixture::new();
            fs::write(f.0.join("data.md"), text).unwrap();
            f.manifest("data.md", "migration-records-v1");
            assert!(f.check().0.is_empty());
            assert!(!f.check().1.is_empty());
        }
    }
    #[test]
    fn operational_metadata_is_never_an_immutable_export() {
        let f = Fixture::new();
        fs::write(
            f.0.join("data.md"),
            PAYLOAD.replacen("type:", "task-id: task:fictional\ntype:", 1),
        )
        .unwrap();
        f.manifest("data.md", "migration-records-v1");
        assert!(!f.check().1.is_empty());
    }
    #[test]
    fn records_require_visible_complete_blocks_and_no_trailing_prose() {
        for text in [
            format!("{PAYLOAD}\n# Ordinary task\n- [ ] Execute\n"),
            PAYLOAD.replace(
                "## sample.daily.2020-01-01",
                "````markdown\n## sample.daily.2020-01-01",
            ) + "````\n",
            format!("{PAYLOAD}\n### Verwijzingen\n- [x](../../outside.md)\n"),
        ] {
            let f = Fixture::new();
            fs::write(f.0.join("data.md"), text).unwrap();
            f.manifest("data.md", "migration-records-v1");
            assert!(f.check().0.is_empty());
        }
        let f = Fixture::new();
        fs::write(
            f.0.join("data.md"),
            PAYLOAD
                .replace("## sample.daily.2020-01-01", "## Fictional title")
                .replace("value: 2", "title: Fictional title"),
        )
        .unwrap();
        f.manifest("data.md", "migration-records-v1");
        assert!(f.check().1.is_empty());
    }
    #[test]
    fn source_logs_require_real_sections_and_a_valid_timestamp() {
        let good = json!({"type":"health-log","status":"reported","date":"2020-01-01","person":"Fictional","source":"Fictional capture","captured_at":"2020-01-01T12:30:00+01:00"});
        let body = "# Fictional input\n\n- **Summary:** reported input.\n\n## Oorspronkelijke aangeleverde tekst\n\n> Quoted original input.\n\n## Bron en afbakening\n\nFictional source boundary.\n";
        assert!(health_log(&good, body).is_ok());
        assert!(health_log(&good, &format!("```text\n{body}```\n")).is_err());
        assert!(health_log(&good, &format!("{body}\n# Task\n- [ ] Execute\n")).is_err());
        assert!(health_log(&good, &format!("{body}\n # Task\n - [ ] Execute\n")).is_err());
        for marker in [
            "1. [ ] Execute",
            "1) Execute",
            "+ Execute",
            "* Execute",
            "---",
        ] {
            assert!(health_log(&good, &format!("{body}\n{marker}\n")).is_err());
        }
        assert!(
            health_log(
                &good,
                &body.replace("reported input.", "<!-- hidden section")
            )
            .is_err()
        );
        for invalid in [
            "2020-01-01Tnot-a-time",
            "2020-01-01T25:00:00Z",
            "2020-01-01T12:00:00",
            "2020-01-01T12:00:00+99:99",
            "2020-01-01T12:00:00++1:00",
            "2020-01-01T12:00:00+01:+0",
            "éééééé2020T",
        ] {
            let mut bad = good.clone();
            bad["captured_at"] = json!(invalid);
            assert!(health_log(&bad, body).is_err());
        }
    }
    #[test]
    fn summary_contract_is_a_real_table_and_visible_source_section() {
        let data = json!({"type":"migraine-derived-summary","status":"current-derived","period":"2020-01","derived_from":"health_migraine_daily","source_summary":"health.migraine.monthly.2020-01","derived_at":"2020-02-01"});
        let body = "# Migraine-overzicht 2020-01\n\nAfgeleid uit dagelijkse bronregistraties; geen medische interpretatie en geen invulling van ontbrekende dagen.\n\n| Maat | Waarde |\n| --- | ---: |\n| Aanwezige dagregistraties | 1 |\n| Unieke kalenderdagen | 1 |\n| Ontbrekende kalenderdagen | 30 |\n| `headache: true` | 0 |\n| `migraine_logged: true` | 1 |\n| `medication_used: true` | 0 |\n| Historische `reported_positive_days` | 0 |\n\nDe historische samenvatting blijft bronhistorie. Verschillen met deze afleiding zijn zichtbaar, niet stil gecorrigeerd.\n\n## Dagbronnen\n\n- `health.migraine.daily.2020-01-01`\n";
        assert!(summary(&data, body).is_ok());
        for bad in [
            body.replace("## Dagbronnen", "<!-- ## Dagbronnen -->"),
            body.replace("| Maat | Waarde |", "| arbitrary prose"),
            body.replace("2020-01-01`", "2020-02-01`"),
            format!("{body}\n- [ ] Execute\n"),
        ] {
            assert!(summary(&data, &bad).is_err());
        }
    }
    #[cfg(unix)]
    #[test]
    fn symlink_payloads_and_manifests_are_rejected() {
        use std::os::unix::fs::symlink;
        let f = Fixture::new();
        fs::write(f.0.join("original.md"), PAYLOAD).unwrap();
        fs::remove_file(f.0.join("data.md")).unwrap();
        symlink("original.md", f.0.join("data.md")).unwrap();
        assert!(!f.check().1.is_empty());
        let f = Fixture::new();
        fs::rename(f.0.join("source-export.json"), f.0.join("original.json")).unwrap();
        symlink("original.json", f.0.join("source-export.json")).unwrap();
        assert!(!f.check().1.is_empty());
    }
}
