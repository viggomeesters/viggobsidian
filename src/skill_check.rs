//! Read-only release byte and local identity checks. Registers never authorize bytes.
use crate::{Result, SKILLS, STATE, checked, digest_at, fail, field, rows, skill_library, state};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};
const BUNDLE: &str = ".agents/skills/_bundle";

fn decode(raw: &str) -> Result<Value> {
    let options = serde_saphyr::options! {duplicate_keys:serde_saphyr::DuplicateKeyPolicy::Error};
    Ok(serde_saphyr::from_str_with_options(raw, options)?)
}
fn read(vault: &Path, path: &str) -> Result<Value> {
    decode(&fs::read_to_string(checked(vault, path)?)?)
}
fn names(
    values: impl IntoIterator<Item = String>,
    label: &str,
    errors: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for name in values {
        if !set.insert(name.clone()) {
            errors.push(format!("duplicate {label}: {name}"));
        }
    }
    let expected: BTreeSet<_> = SKILLS.iter().map(|s| s.to_string()).collect();
    for name in expected.difference(&set) {
        errors.push(format!("missing {label}: {name}"));
    }
    for name in set.difference(&expected) {
        errors.push(format!("unexpected {label}: {name}"));
    }
    set
}
fn strings(value: &Value) -> Result<Vec<String>> {
    value
        .as_array()
        .ok_or("expected name array")?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "expected skill name".into())
        })
        .collect()
}
fn keys(value: &Value) -> Result<Vec<String>> {
    Ok(value
        .as_object()
        .ok_or("expected identity mapping")?
        .keys()
        .cloned()
        .collect())
}
fn instruction_files(vault: &Path, path: &str, found: &mut BTreeSet<String>) -> Result<()> {
    for entry in fs::read_dir(checked(vault, path)?)? {
        let entry = entry?;
        let child = format!(
            "{path}/{}",
            entry.file_name().to_str().ok_or("non-UTF8 skill path")?
        );
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            // Code/assets may retain provider-owned links. Never traverse them;
            // manifest-owned instruction links are rejected by digest_at above.
            if path == ".agents/skills" || child.ends_with(".md") {
                return fail(format!("symlink refused: {child}"));
            }
            continue;
        }
        if kind.is_dir() {
            instruction_files(vault, &child, found)?;
        } else if path == ".agents/skills" || child.ends_with(".md") {
            found.insert(child);
        }
    }
    Ok(())
}
pub(crate) fn run(vault: &Path) -> Result<Value> {
    // Strict decoding also rejects duplicate keys in the receipt/embedded manifest
    // before the package updater's existing state validation is used.
    let receipt = read(vault, STATE)?;
    let manifest = decode(field(&receipt, "manifest_bytes")?)?;
    let installed = state(vault)?;
    if installed.is_null() {
        return fail("skill-check requires an installed package");
    }
    let mut errors = Vec::new();
    let managed: BTreeSet<_> = rows(&manifest, "files")?
        .iter()
        .map(|r| field(r, "target").map(str::to_owned))
        .collect::<Result<BTreeSet<_>>>()?
        .into_iter()
        .filter(|p| p.starts_with(".agents/skills/") && p.ends_with(".md"))
        .collect();
    let expected: BTreeSet<_> = skill_library::targets()
        .iter()
        .filter(|p| p.starts_with(".agents/skills/") && p.ends_with(".md"))
        .cloned()
        .collect();
    if managed != expected {
        errors.push("installed skill instruction surface differs from this CLI release".into());
    }
    for path in &managed {
        if digest_at(vault, path)? != installed["files"][path] {
            errors.push(format!("release instruction drift: {path}"));
        }
    }
    let mut actual = BTreeSet::new();
    instruction_files(vault, ".agents/skills", &mut actual)?;
    for path in actual.difference(&managed) {
        errors.push(format!("unmanaged skill instruction: {path}"));
    }
    let manifest_names = managed
        .iter()
        .filter_map(|p| p.strip_prefix(".agents/skills/")?.strip_suffix("/SKILL.md"))
        .map(str::to_owned);
    names(manifest_names, "manifest core", &mut errors);
    let actual_names = actual
        .iter()
        .filter_map(|p| p.strip_prefix(".agents/skills/")?.strip_suffix("/SKILL.md"))
        .map(str::to_owned);
    let actual_names = names(actual_names, "installed core", &mut errors);
    let native = read(vault, &format!("{BUNDLE}/native-surface.json"))?;
    names(strings(&native["public"])?, "native core", &mut errors);
    let sources = read(vault, &format!("{BUNDLE}/source-lock.json"))?;
    let sources = sources.as_array().ok_or("source-lock must be an array")?;
    names(
        sources
            .iter()
            .map(|s| field(s, "name").map(str::to_owned))
            .collect::<Result<Vec<_>>>()?,
        "source core",
        &mut errors,
    );
    for row in sources {
        let name = field(row, "name")?;
        let path = format!(".agents/skills/{name}/SKILL.md");
        let origin = format!(
            "https://github.com/viggomeesters/viggobsidian/blob/{}/skills/{name}/SKILL.md",
            field(&installed, "revision")?
        );
        if row["path"] != path
            || row["management_role"] != "viggobsidian-release"
            || row["hash_authority"] != STATE
            || row["origin"] != origin
        {
            errors.push(format!("invalid release source identity: {name}"));
        }
    }
    let catalog = read(vault, &format!("{BUNDLE}/catalog.yaml"))?;
    let entrypoints = keys(&catalog["entrypoints"])?;
    names(entrypoints.clone(), "catalog entry", &mut errors);
    let catalog_sources = rows(&catalog, "sources")?;
    names(
        catalog_sources
            .iter()
            .map(|s| field(s, "source_name").map(str::to_owned))
            .collect::<Result<Vec<_>>>()?,
        "catalog source",
        &mut errors,
    );
    for row in catalog_sources {
        let name = field(row, "source_name")?;
        if row["source_owner"] != "vault"
            || row["vault_path"] != format!(".agents/skills/{name}/SKILL.md")
        {
            errors.push(format!("invalid catalog source identity: {name}"));
        }
    }
    let modules = catalog["modules"]
        .as_object()
        .ok_or("catalog modules must be a mapping")?;
    if !modules.is_empty() {
        errors.push("retired procedure modules are not supported".into());
    }
    let aliases = read(vault, &format!("{BUNDLE}/aliases.yaml"))?;
    names(keys(&aliases)?, "identity route", &mut errors);
    for (name, route) in aliases
        .as_object()
        .ok_or("identity routes must be a mapping")?
    {
        if route["entry"] != *name
            || route["source"] != format!(".agents/skills/{name}/SKILL.md")
            || !route["module"].is_null()
            || route["disposition"] != "canonical-core"
        {
            errors.push(format!("nonidentity skill route: {name}"));
        }
    }
    Ok(
        json!({"schema":"viggobsidian.skill-check.v1", "version":env!("CARGO_PKG_VERSION"), "vault":vault.canonicalize()?,
        "sources":sources.len(), "native_skills":actual_names.len(), "entrypoints":entrypoints.len(),
        "modules":modules.len(), "checked_instruction_files":managed.len(), "errors":errors}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{atomic, hash, stamp};
    use std::path::PathBuf;
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let base = std::env::var_os("VIGGOBSIDIAN_TEST_TMPROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tmp"));
            let f = Self(base.join(format!("skill-check-{}", stamp().unwrap())));
            fs::create_dir_all(&f.0).unwrap();
            let mut files = Vec::new();
            for path in skill_library::targets()
                .iter()
                .filter(|p| p.starts_with(".agents/skills/") && p.ends_with(".md"))
            {
                let content = format!("Fictional release instruction: {path}\n");
                atomic(&f.0, path, content.as_bytes()).unwrap();
                files.push(json!({"target":path,"sha256":hash(content.as_bytes())}));
            }
            let manifest = json!({"schema":"viggobsidian.package.v1","version":env!("CARGO_PKG_VERSION"),"revision":"a".repeat(40),"files":files});
            f.manifest(manifest);
            f.put("native-surface.json", json!({"public": SKILLS}));
            f.put("source-lock.json", Value::Array(SKILLS.iter().map(|n| json!({"name":n,"path":format!(".agents/skills/{n}/SKILL.md"),"management_role":"viggobsidian-release","hash_authority":STATE,"origin":format!("https://github.com/viggomeesters/viggobsidian/blob/{}/skills/{n}/SKILL.md", "a".repeat(40))})).collect()));
            let entries: serde_json::Map<_, _> =
                SKILLS.iter().map(|n| (n.to_string(), json!({}))).collect();
            f.put("catalog.yaml", json!({"entrypoints":entries,"modules":{},"sources":SKILLS.iter().map(|n|json!({"source_name":n,"source_owner":"vault","vault_path":format!(".agents/skills/{n}/SKILL.md")})).collect::<Vec<_>>()}));
            let aliases: serde_json::Map<_,_> = SKILLS.iter().map(|n|(n.to_string(),json!({"entry":n,"source":format!(".agents/skills/{n}/SKILL.md"),"disposition":"canonical-core"}))).collect();
            f.put("aliases.yaml", Value::Object(aliases));
            f
        }
        fn manifest(&self, manifest: Value) {
            let raw = serde_json::to_string(&manifest).unwrap();
            let files: serde_json::Map<_, _> = manifest["files"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    (
                        r["target"].as_str().unwrap().to_string(),
                        r["sha256"].clone(),
                    )
                })
                .collect();
            atomic(&self.0, STATE, &serde_json::to_vec(&json!({"schema":"viggobsidian.installed.v1","version":manifest["version"],"revision":manifest["revision"],"manifest_bytes":raw,"manifest_sha256":hash(raw.as_bytes()),"files":files})).unwrap()).unwrap();
        }
        fn put(&self, name: &str, value: Value) {
            atomic(
                &self.0,
                &format!("{BUNDLE}/{name}"),
                &serde_json::to_vec(&value).unwrap(),
            )
            .unwrap();
        }
        fn get(&self, name: &str) -> Value {
            read(&self.0, &format!("{BUNDLE}/{name}")).unwrap()
        }
        fn errors(&self) -> Vec<Value> {
            run(&self.0).unwrap()["errors"].as_array().unwrap().clone()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn baseline_checks_all_thirteen_instructions_without_mutation() {
        let f = Fixture::new();
        let before = fs::read(f.0.join(STATE)).unwrap();
        let out = run(&f.0).unwrap();
        assert_eq!(out["errors"], json!([]));
        assert_eq!(out["checked_instruction_files"], 13);
        assert_eq!(out["sources"], 10);
        assert_eq!(out["vault"], json!(f.0.canonicalize().unwrap()));
        assert_eq!(fs::read(f.0.join(STATE)).unwrap(), before);
    }
    #[test]
    fn each_reference_is_hashed_independently_of_source_lock() {
        for path in skill_library::targets()
            .iter()
            .filter(|p| p.contains("/references/"))
        {
            let f = Fixture::new();
            atomic(&f.0, path, b"altered reference").unwrap();
            assert!(
                f.errors()
                    .iter()
                    .any(|e| e.as_str().unwrap().contains(path))
            );
        }
    }
    #[test]
    fn missing_duplicate_and_renamed_rows_fail_every_array_register() {
        for (file, member, key) in [
            ("source-lock.json", None, "name"),
            ("native-surface.json", Some("public"), ""),
            ("catalog.yaml", Some("sources"), "source_name"),
        ] {
            for mutation in ["missing", "duplicate", "rename"] {
                let f = Fixture::new();
                let mut value = f.get(file);
                let array = if let Some(member) = member {
                    value[member].as_array_mut().unwrap()
                } else {
                    value.as_array_mut().unwrap()
                };
                match mutation {
                    "missing" => {
                        array.remove(0);
                    }
                    "duplicate" => array.push(array[0].clone()),
                    _ => {
                        if key.is_empty() {
                            array[0] = json!("other");
                        } else {
                            array[0][key] = json!("other");
                        }
                    }
                }
                f.put(file, value);
                assert!(!f.errors().is_empty(), "{file} {mutation}");
            }
        }
    }
    #[test]
    fn identity_routes_and_catalog_entries_cannot_omit_or_redirect_cores() {
        for file in ["catalog.yaml", "aliases.yaml"] {
            let f = Fixture::new();
            let mut value = f.get(file);
            let map = if file == "catalog.yaml" {
                value["entrypoints"].as_object_mut().unwrap()
            } else {
                value.as_object_mut().unwrap()
            };
            map.remove("audit");
            f.put(file, value);
            assert!(!f.errors().is_empty());
        }
        let f = Fixture::new();
        let mut value = f.get("aliases.yaml");
        value["audit"]["entry"] = json!("research");
        f.put("aliases.yaml", value);
        assert!(!f.errors().is_empty());
        let f = Fixture::new();
        let mut value = f.get("source-lock.json");
        value[0]["path"] = json!(".agents/skills/research/SKILL.md");
        f.put("source-lock.json", value);
        assert!(!f.errors().is_empty());
    }
    #[test]
    fn duplicate_mapping_keys_reject_instead_of_silently_collapsing() {
        for raw in ["{\"audit\":{},\"audit\":{}}", "audit: {}\naudit: {}\n"] {
            assert!(decode(raw).is_err());
        }
        let f = Fixture::new();
        atomic(
            &f.0,
            &format!("{BUNDLE}/aliases.yaml"),
            b"audit: {}\naudit: {}\n",
        )
        .unwrap();
        assert!(run(&f.0).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn provider_code_links_are_not_instruction_sources_or_mutated() {
        let f = Fixture::new();
        let code = f.0.join(".agents/skills/vendor/scripts/office");
        fs::create_dir_all(code.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink("/unavailable/provider-code", &code).unwrap();
        assert!(f.errors().is_empty());
        assert_eq!(
            fs::read_link(&code).unwrap(),
            PathBuf::from("/unavailable/provider-code")
        );
        std::os::unix::fs::symlink(
            "/unavailable/instruction",
            f.0.join(".agents/skills/vendor/SKILL.md"),
        )
        .unwrap();
        assert!(run(&f.0).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn top_level_discovery_links_cannot_hide_an_extra_skill() {
        let f = Fixture::new();
        let external = f.0.join("external");
        fs::create_dir_all(&external).unwrap();
        fs::write(external.join("SKILL.md"), b"extra skill").unwrap();
        std::os::unix::fs::symlink(&external, f.0.join(".agents/skills/retired")).unwrap();
        assert!(run(&f.0).is_err());
    }
    #[test]
    fn missing_reference_in_manifest_and_unmanaged_instructions_fail() {
        let f = Fixture::new();
        let state = read(&f.0, STATE).unwrap();
        let mut manifest = decode(field(&state, "manifest_bytes").unwrap()).unwrap();
        manifest["files"].as_array_mut().unwrap().retain(|r| {
            !r["target"]
                .as_str()
                .unwrap()
                .contains("/references/interface-check.md")
        });
        f.manifest(manifest);
        assert!(!f.errors().is_empty());
        let f = Fixture::new();
        atomic(
            &f.0,
            ".agents/skills/retired/PROCEDURE.md",
            b"hidden instruction",
        )
        .unwrap();
        assert!(!f.errors().is_empty());
    }
}
