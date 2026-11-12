//! Read-only shared procedure and local capability preflight. Never executes a provider.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const MAP: &str = ".viggobsidian/core/procedures.json";
const PROVIDERS: &str = ".viggobsidian/providers.json";

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn explicit_root(path: &Path) -> Result<()> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut cursor = PathBuf::new();
    for part in absolute.components() {
        if matches!(part, Component::ParentDir) {
            return Err("parent component in skill target root".into());
        }
        cursor.push(part);
        if fs::symlink_metadata(&cursor)?.is_symlink() {
            return Err("symlink in skill target root".into());
        }
    }
    Ok(())
}

fn safe(vault: &Path, raw: &str) -> Result<PathBuf> {
    if raw.is_empty()
        || raw.contains('\\')
        || raw
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == "..")
        || Path::new(raw)
            .components()
            .any(|p| !matches!(p, Component::Normal(_)))
    {
        return Err("unsafe procedure/provider locator".into());
    }
    let mut path = vault.to_path_buf();
    for part in Path::new(raw).components() {
        path.push(part);
        if fs::symlink_metadata(&path)?.is_symlink() {
            return Err("symlink in procedure/provider locator".into());
        }
    }
    if !path.is_file() {
        return Err(format!("missing procedure/provider file: {raw}").into());
    }
    Ok(path)
}

fn decode(bytes: &[u8]) -> Result<Value> {
    let options = serde_saphyr::options! {duplicate_keys:serde_saphyr::DuplicateKeyPolicy::Error};
    Ok(serde_saphyr::from_str_with_options(
        std::str::from_utf8(bytes)?,
        options,
    )?)
}

fn object_keys(value: &Value, allowed: &[&str]) -> Result<()> {
    if value
        .as_object()
        .is_none_or(|o| o.keys().any(|key| !allowed.contains(&key.as_str())))
    {
        return Err("unknown or malformed procedure/provider field".into());
    }
    Ok(())
}

fn installed(vault: &Path) -> Result<(Value, Vec<u8>)> {
    let bytes = fs::read(safe(vault, ".viggobsidian/installed.json")?)?;
    let state = decode(&bytes)?;
    let raw = state["manifest_bytes"]
        .as_str()
        .ok_or("installed manifest missing")?;
    let manifest: Value = serde_json::from_str(raw)?;
    let entries = manifest["files"]
        .as_array()
        .ok_or("installed manifest entries missing")?;
    if state["schema"] != "viggobsidian.installed.v1"
        || manifest["schema"] != "viggobsidian.package.v1"
        || state["manifest_sha256"] != hash(raw.as_bytes())
        || state["version"] != manifest["version"]
        || state["revision"] != manifest["revision"]
    {
        return Err("installed procedure manifest integrity failure".into());
    }
    let mut files = BTreeMap::new();
    for item in entries {
        let path = item["target"].as_str().ok_or("manifest target missing")?;
        let expected =
            if item["transform"] == "css-v1" && path == ".obsidian/snippets/folder-note.css" {
                let derived = &state["derived"][path];
                if derived["kind"] != "css-v1"
                    || derived["source_sha256"] != item["sha256"]
                    || derived["config"] != ".viggobsidian/appearance.json"
                    || derived["config_sha256"].as_str().is_none()
                {
                    return Err("invalid installed CSS derivation".into());
                }
                &derived["sha256"]
            } else if item["transform"].is_null() {
                &item["sha256"]
            } else {
                return Err("unsupported installed transform".into());
            };
        if expected.as_str().is_none() || files.insert(path.to_owned(), expected.clone()).is_some()
        {
            return Err("invalid installed manifest target".into());
        }
    }
    if state["files"] != json!(files) {
        return Err("installed file map mismatch".into());
    }
    Ok((state, bytes))
}

fn verify_asset(vault: &Path, state: &Value, path: &str) -> Result<Value> {
    let bytes = fs::read(safe(vault, path)?)?;
    let digest = hash(&bytes);
    if state["files"][path] != digest {
        return Err(format!("release procedure drift: {path}").into());
    }
    Ok(json!({"path":path,"sha256":digest}))
}

pub(crate) fn check(
    vault: &Path,
    route: Option<&str>,
    source: Option<&str>,
) -> Result<(Value, u8)> {
    if source.is_some()
        && !matches!(
            route,
            Some(
                "vault"
                    | "vault-log"
                    | "vault-process"
                    | "vault-inbox"
                    | "inbox"
                    | "bookmarks"
                    | "vault-email"
                    | "vault-email-process"
            )
        )
    {
        return Err("source adapter is only an additional capture/process dependency".into());
    }
    let (state, state_bytes) = installed(vault)?;
    let descriptor_bytes = fs::read(safe(vault, MAP)?)?;
    let descriptor_hash = hash(&descriptor_bytes);
    if state["files"][MAP] != descriptor_hash {
        return Err("release procedure descriptor drift".into());
    }
    let descriptor = json!({"path":MAP,"sha256":descriptor_hash});
    let spec = decode(&descriptor_bytes)?;
    if spec["schema"] != "viggobsidian.procedures.v1" {
        return Err("unknown procedure map schema".into());
    }
    let contracts = spec["contracts"]
        .as_array()
        .ok_or("procedure contracts missing")?;
    let known: BTreeSet<_> = contracts
        .iter()
        .map(|v| v.as_str().ok_or("invalid contract name"))
        .collect::<std::result::Result<_, _>>()?;
    let routes = spec["routes"]
        .as_object()
        .ok_or("procedure routes missing")?;
    let references = spec["references"]
        .as_object()
        .ok_or("procedure references missing")?;
    let selected: Vec<_> = if let Some(route) = route {
        vec![(
            route,
            routes
                .get(route)
                .ok_or("unknown skill route; no fallback")?,
        )]
    } else {
        routes.iter().map(|(n, v)| (n.as_str(), v)).collect()
    };
    let provider_bytes = fs::read(safe(vault, PROVIDERS)?)?;
    let providers = decode(&provider_bytes)?;
    object_keys(&providers, &["schema", "vault", "capabilities"])?;
    if providers["schema"] != "viggobsidian.providers.v1" || providers["vault"] != json!(vault) {
        return Err("provider target vault mismatch or schema error".into());
    }
    let capabilities = providers["capabilities"]
        .as_object()
        .ok_or("provider capabilities missing")?;
    let mut assets = BTreeMap::new();
    assets.insert(MAP.to_string(), descriptor);
    let mut results = Vec::new();
    for (name, definition) in selected {
        let capability = definition["capability"]
            .as_str()
            .ok_or("route capability missing")?;
        let entry = format!(
            ".agents/skills/{name}/{}",
            if definition["kind"] == "entry" {
                "SKILL.md"
            } else {
                "PROCEDURE.md"
            }
        );
        let mut paths = vec![
            entry,
            ".agents/skills/_bundle/contracts/native-vault.md".to_owned(),
        ];
        for contract in definition["contracts"]
            .as_array()
            .ok_or("route contracts missing")?
        {
            let name = contract.as_str().ok_or("invalid route contract")?;
            if !known.contains(name) {
                return Err("route references unknown contract".into());
            }
            paths.push(format!(".viggobsidian/core/procedures/{name}.md"));
        }
        for (reference, contract) in references {
            let (owner, file) = reference
                .split_once('/')
                .ok_or("invalid reference mapping")?;
            if !known.contains(contract.as_str().ok_or("invalid reference contract")?) {
                return Err("reference uses unknown contract".into());
            }
            if route.is_none() || owner == name {
                paths.push(format!(".agents/skills/{owner}/references/{file}.md"));
                paths.push(format!(
                    ".viggobsidian/core/procedures/{}.md",
                    contract.as_str().unwrap()
                ));
            }
        }
        for path in &paths {
            if !assets.contains_key(path) {
                assets.insert(path.clone(), verify_asset(vault, &state, path)?);
            }
        }
        results.push(provider(vault, name, capability, capabilities)?);
    }
    for (name, capability) in spec["source_adapters"]
        .as_object()
        .ok_or("source adapters missing")?
    {
        if route.is_none() || source == Some(name.as_str()) {
            let capability = capability.as_str().ok_or("invalid source adapter")?;
            results.push(provider(
                vault,
                &format!("source:{name}"),
                capability,
                capabilities,
            )?);
        }
    }
    if source.is_some_and(|name| spec["source_adapters"].get(name).is_none()) {
        return Err("unknown source adapter; no fallback".into());
    }
    let code = if route.is_some() && results.iter().any(|r| r["state"] == "unavailable") {
        1
    } else {
        0
    };
    for (path, observed) in [
        (MAP, &descriptor_bytes),
        (PROVIDERS, &provider_bytes),
        (".viggobsidian/installed.json", &state_bytes),
    ] {
        if fs::read(safe(vault, path)?)? != *observed {
            return Err("procedure/provider binding changed during preflight".into());
        }
    }
    Ok((
        json!({"schema":"viggobsidian.skill-routes.v1","vault":vault,"version":state["version"],"installed_sha256":hash(&state_bytes),"providers_sha256":hash(&provider_bytes),"assets":assets,"routes":results,"write_authority":false}),
        code,
    ))
}

fn provider(
    vault: &Path,
    name: &str,
    capability: &str,
    capabilities: &serde_json::Map<String, Value>,
) -> Result<Value> {
    let provider = capabilities
        .get(capability)
        .ok_or_else(|| format!("missing explicit capability: {capability}"))?;
    object_keys(
        provider,
        &[
            "state",
            "mode",
            "instructions",
            "section",
            "required_files",
            "external_files",
            "reason",
        ],
    )?;
    let status = provider["state"].as_str().ok_or("provider state missing")?;
    let mode = provider["mode"].as_str().ok_or("provider mode missing")?;
    let mut dependencies = Vec::new();
    if status == "available" {
        if !matches!(mode, "read-only" | "guided" | "command" | "legacy") {
            return Err("invalid provider mode".into());
        }
        let instruction = provider["instructions"]
            .as_str()
            .ok_or("provider instructions missing")?;
        let bytes = fs::read(safe(vault, instruction)?)?;
        let section = provider["section"]
            .as_str()
            .ok_or("provider section missing")?;
        if !std::str::from_utf8(&bytes)?
            .lines()
            .any(|line| line == format!("## {section}"))
        {
            return Err("provider instruction section missing".into());
        }
        dependencies.push(json!({"path":instruction,"sha256":hash(&bytes)}));
        for path in provider["required_files"]
            .as_array()
            .ok_or("provider file list missing")?
        {
            let path = path.as_str().ok_or("invalid provider dependency")?;
            dependencies.push(json!({"path":path,"sha256":hash(&fs::read(safe(vault,path)?)?)}));
        }
        if let Some(external) = provider.get("external_files") {
            for path in external.as_array().ok_or("external files must be a list")? {
                let raw = path
                    .as_str()
                    .ok_or("invalid external provider dependency")?;
                let path = Path::new(raw);
                if !path.is_absolute() {
                    return Err("external provider locator must be explicit absolute path".into());
                }
                explicit_root(path)?;
                if !path.is_file() {
                    return Err("external provider member must be a regular file".into());
                }
                dependencies
                    .push(json!({"path":raw,"sha256":hash(&fs::read(path)?),"external":true}));
            }
        }
    } else if status != "unavailable"
        || mode != "unavailable"
        || provider["reason"]
            .as_str()
            .is_none_or(|s| s.trim().is_empty())
    {
        return Err("unavailable provider needs an explicit reason".into());
    }
    Ok(
        json!({"route":name,"capability":capability,"state":status,"mode":mode,"reason":provider["reason"],"instructions":provider["instructions"],"section":provider["section"],"dependencies":dependencies,"write_authority":false}),
    )
}

pub(crate) fn run(vault: &Path, route: Option<&str>, source: Option<&str>) -> Result<u8> {
    let (report, code) = check(vault, route, source)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(code)
}
