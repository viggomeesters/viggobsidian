//! Read-only phase preflight. Never installs, builds, edits or publishes a plugin.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const SUFFIX: &[u8] = b"\n/* nosourcemap */";
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreviewIntent {
    PublicReview,
    LocalPrivate,
}
impl PreviewIntent {
    fn from_args(args: &[String], phase: &str) -> Result<Self> {
        let mut selected = None;
        for (index, arg) in args.iter().enumerate() {
            let value = if arg == "--preview-intent" {
                Some(
                    args.get(index + 1)
                        .filter(|value| !value.starts_with("--"))
                        .map(String::as_str)
                        .ok_or("missing value for --preview-intent")?,
                )
            } else {
                arg.strip_prefix("--preview-intent=")
            };
            if let Some(value) = value
                && selected.replace(value).is_some()
            {
                return Err("duplicate --preview-intent".into());
            }
        }
        Self::parse(selected.unwrap_or("public-review"), phase)
    }
    fn parse(value: &str, phase: &str) -> Result<Self> {
        match value {
            "public-review" => Ok(Self::PublicReview),
            "local-private" if phase == "source-preview" => Ok(Self::LocalPrivate),
            "local-private" => Err("local-private intent is only valid for source-preview".into()),
            _ => Err("invalid --preview-intent; use public-review or local-private".into()),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::PublicReview => "public-review",
            Self::LocalPrivate => "local-private",
        }
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn command(program: &str, args: &[&str], cwd: Option<&Path>) -> Result<Vec<u8>> {
    let mut cmd = Command::new(program);
    if program == "git" {
        for key in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE"] {
            cmd.env_remove(key);
        }
    }
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let out = cmd.output()?;
    if !out.status.success() {
        return Err(format!("{program} failed: {}", String::from_utf8_lossy(&out.stderr)).into());
    }
    Ok(out.stdout)
}
fn git(dir: &Path, args: &[&str]) -> Result<String> {
    Ok(String::from_utf8(command("git", args, Some(dir))?)?
        .trim()
        .to_owned())
}
fn file_json(dir: &Path, name: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(dir.join(name))?)?)
}
fn gh(endpoint: &str) -> Value {
    match command("gh", &["api", endpoint], None) {
        Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
            Ok(value) => json!({"state":"ok","value":value}),
            Err(e) => json!({"state":"error","detail":e.to_string()}),
        },
        Err(e) => {
            json!({"state":if e.to_string().contains("HTTP 404"){"missing"}else{"error"},"detail":e.to_string()})
        }
    }
}
fn download(url: &str) -> Result<Vec<u8>> {
    command(
        "curl",
        &[
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--max-time",
            "30",
            "--max-filesize",
            "52428800",
            url,
        ],
        None,
    )
}
fn source_text(dir: &Path, out: &mut String) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let meta = fs::symlink_metadata(&path)?;
        if meta.is_symlink() {
            return Err("source symlinks require separate review".into());
        }
        if meta.is_dir() {
            source_text(&path, out)?;
        } else if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("ts" | "tsx" | "js" | "mjs")
        ) {
            if meta.len() > 10_000_000 {
                return Err("source exceeds preflight read limit".into());
            }
            out.push_str(&fs::read_to_string(path)?);
        }
    }
    Ok(())
}
fn es2020(lib: &str) -> bool {
    let s = lib.to_ascii_lowercase();
    s == "esnext" || s == "es2020.string" || (2020..=2030).any(|year| s == format!("es{year}"))
}
fn local(dir: &Path, candidate: &str, installed: Option<&Path>) -> Result<Value> {
    let manifest = file_json(dir, "manifest.json")?;
    let mut assets = serde_json::Map::new();
    for name in ["main.js", "manifest.json", "styles.css"] {
        if dir.join(name).is_file() {
            assets.insert(name.into(), json!(digest(&fs::read(dir.join(name))?)));
        }
    }
    let mut text = String::new();
    source_text(&dir.join("src"), &mut text)?;
    source_text(&dir.join("lib"), &mut text)?;
    if dir.join("main.js").exists() {
        text.push_str(&fs::read_to_string(dir.join("main.js"))?);
    }
    if dir.join("main.ts").exists() {
        text.push_str(&fs::read_to_string(dir.join("main.ts"))?);
    }
    let ts = file_json(dir, "tsconfig.json").ok();
    let libs = ts
        .as_ref()
        .and_then(|t| t["compilerOptions"]["lib"].as_array());
    let compiler_lib = libs.map(|rows| rows.iter().filter_map(Value::as_str).any(es2020));
    let commit = git(dir, &["rev-parse", &format!("{candidate}^{{commit}}")])?;
    let changed = git(dir, &["diff", "--name-only", &commit, "--"])?;
    let bound = changed.lines().all(|path| {
        (path.starts_with("docs/")
            && matches!(
                Path::new(path).extension().and_then(|s| s.to_str()),
                Some("md" | "png" | "jpg" | "jpeg" | "webp")
            ))
            || (path.starts_with("docs/evidence/") && path.ends_with(".json"))
            || matches!(
                path,
                "README.md" | "CHANGELOG.md" | "SECURITY.md" | "CONTRIBUTING.md" | "LICENSE"
            )
    });
    let mut installation = json!({"state":"not-supplied"});
    if let Some(target) = installed {
        let m = file_json(target, "manifest.json")?;
        let mut hashes = BTreeMap::new();
        let mut suffix_hashes = BTreeMap::new();
        for name in assets.keys() {
            let bytes = fs::read(target.join(name))?;
            hashes.insert(name.clone(), digest(&bytes));
            if name == "main.js" && bytes.ends_with(SUFFIX) {
                suffix_hashes.insert(name.clone(), digest(&bytes[..bytes.len() - SUFFIX.len()]));
            }
        }
        installation =
            json!({"state":"read","manifest":m,"hashes":hashes,"suffixHashes":suffix_hashes});
    }
    let pkg = file_json(dir, "package.json")?;
    let lint = pkg["devDependencies"]["eslint-plugin-obsidianmd"].is_string()
        || pkg["dependencies"]["eslint-plugin-obsidianmd"].is_string();
    let lint_config = [
        "eslint.config.mjs",
        "eslint.config.js",
        "eslint.config.cjs",
        ".eslintrc.json",
        ".eslintrc.cjs",
    ]
    .iter()
    .any(|p| dir.join(p).is_file());
    let docs_present = [
        "README.md",
        "LICENSE",
        "SECURITY.md",
        "CONTRIBUTING.md",
        "CHANGELOG.md",
    ]
    .iter()
    .all(|p| dir.join(p).is_file());
    Ok(
        json!({"manifest":manifest,"package":pkg,"lock":file_json(dir,"package-lock.json").ok(),"versions":file_json(dir,"versions.json").ok(),"assets":assets,"commit":commit,"candidateBound":bound,"clean":git(dir,&["status","--porcelain"])?.is_empty(),"matchAll":text.contains(".matchAll("),"es2020Library":compiler_lib,"officialLintConfigured":lint&&lint_config,"docsPresent":docs_present,"installed":installation}),
    )
}
fn dereference_tag(mut tag: Value, mut fetch: impl FnMut(&str) -> Value) -> Value {
    for _ in 0..16 {
        if tag["state"] != "ok" || tag["value"]["object"]["type"] != "tag" {
            return tag;
        }
        let next = fetch(tag["value"]["object"]["sha"].as_str().unwrap_or(""));
        tag = if next["state"] == "ok" {
            json!({"state":"ok","value":{"object":next["value"]["object"]}})
        } else {
            next
        };
    }
    json!({"state":"error","detail":"annotated tag dereference limit"})
}
fn remote(repo: &str, local: &Value, phase: &str) -> Value {
    let base = format!("repos/{repo}");
    let version = local["manifest"]["version"].as_str().unwrap_or("");
    let release = gh(&format!("{base}/releases/tags/{version}"));
    let (default_manifest_state, default_manifest) = match command(
        "gh",
        &[
            "api",
            &format!("{base}/contents/manifest.json"),
            "--jq",
            ".content | @base64d",
        ],
        None,
    ) {
        Ok(raw) => match serde_json::from_slice::<Value>(&raw) {
            Ok(value) => ("ok", value),
            Err(_) => ("invalid", Value::Null),
        },
        Err(e) => (
            if e.to_string().contains("HTTP 404") {
                "missing"
            } else {
                "error"
            },
            Value::Null,
        ),
    };
    let tag = dereference_tag(gh(&format!("{base}/git/ref/tags/{version}")), |sha| {
        gh(&format!("{base}/git/tags/{sha}"))
    });
    let tag_commit = if tag["value"]["object"]["type"] == "commit" {
        tag["value"]["object"]["sha"].clone()
    } else {
        Value::Null
    };
    let mut downloaded = serde_json::Map::new();
    if phase != "source-preview" && release["state"] == "ok" && release["value"]["draft"] == false {
        for name in local["assets"]
            .as_object()
            .into_iter()
            .flat_map(|a| a.keys())
        {
            if let Some(asset) = release["value"]["assets"]
                .as_array()
                .and_then(|a| a.iter().find(|v| v["name"] == name.as_str()))
            {
                let url = asset["browser_download_url"].as_str().unwrap_or("");
                if url.starts_with(&format!("https://github.com/{repo}/releases/download/")) {
                    downloaded.insert(name.clone(),match download(url){Ok(raw)=>json!({"state":"ok","hash":digest(&raw),"manifest":if name=="manifest.json"{serde_json::from_slice::<Value>(&raw).ok()}else{None}}),Err(e)=>json!({"state":"error","detail":e.to_string()})});
                }
            }
        }
    }
    let id = local["manifest"]["id"].as_str().unwrap_or("");
    let mut catalog = json!({"state":"not-requested"});
    if phase == "catalog-install" {
        catalog = match download(
            "https://raw.githubusercontent.com/obsidianmd/obsidian-releases/HEAD/community-plugins.json",
        ) {
            Ok(raw) => match serde_json::from_slice::<Vec<Value>>(&raw) {
                Ok(rows) => json!({"state":"ok","entry":rows.into_iter().find(|v|v["id"]==id)}),
                Err(e) => json!({"state":"error","detail":e.to_string()}),
            },
            Err(e) => json!({"state":"error","detail":e.to_string()}),
        };
        let url = format!("https://community.obsidian.md/plugins/{id}");
        catalog["listingVersion"] = match download(&url) {
            Ok(raw) => {
                let html = String::from_utf8_lossy(&raw);
                let marker = "Current version</div><div";
                html.find(marker)
                    .and_then(|i| html[i + marker.len()..].split_once('>'))
                    .and_then(|(_, tail)| tail.split_once('<'))
                    .map(|(version, _)| json!(version))
                    .unwrap_or(Value::Null)
            }
            Err(_) => Value::Null,
        };
    }
    json!({"repo":gh(&base),"release":release,"defaultManifest":default_manifest,"defaultManifestState":default_manifest_state,"tagCommit":tag_commit,"tagState":if tag["value"]["object"]["type"]=="tag"{"error"}else{tag["state"].as_str().unwrap_or("error")},"downloads":downloaded,"catalog":catalog})
}
fn check(checks: &mut Vec<Value>, id: &str, status: &str, detail: &str) {
    checks.push(json!({"id":id,"status":status,"detail":detail}));
}
fn boolean(checks: &mut Vec<Value>, id: &str, ok: bool, detail: &str) {
    check(checks, id, if ok { "pass" } else { "fail" }, detail);
}
fn proof(checks: &mut Vec<Value>, evidence: &Value, id: &str, commit: &Value) {
    let item = &evidence[id];
    check(
        checks,
        id,
        if item["status"] == "pass" && &item["commit"] == commit {
            "pass"
        } else {
            "pending"
        },
        "Supplied evidence must name this exact candidate; not independently executed by this read-only checker.",
    );
}
pub(crate) fn evaluate(
    local: &Value,
    remote: &Value,
    evidence: &Value,
    phase: &str,
    initial: bool,
    repo: &str,
    intent: PreviewIntent,
) -> Value {
    let mut checks = Vec::new();
    // The scope is checked again here so non-CLI callers cannot use a private
    // preview intent to weaken a publication phase.
    let private_preview = intent == PreviewIntent::LocalPrivate && phase == "source-preview";
    boolean(
        &mut checks,
        "preview-intent-scope",
        intent != PreviewIntent::LocalPrivate || private_preview,
        "Local-private acceptance is limited to source-preview; publication requires public source and release/catalog evidence.",
    );
    let m = &local["manifest"];
    let version = m["version"].as_str().unwrap_or("");
    let id = m["id"].as_str().unwrap_or("");
    let semver = version.split('.').count() == 3
        && version.split('.').all(|p| {
            !p.is_empty()
                && p.bytes().all(|c| c.is_ascii_digit())
                && (p.len() == 1 || !p.starts_with('0'))
        });
    boolean(
        &mut checks,
        "manifest",
        semver
            && !id.is_empty()
            && id.bytes().all(|c| c.is_ascii_lowercase() || c == b'-')
            && !id.contains("obsidian")
            && !id.ends_with("plugin")
            && m["isDesktopOnly"].is_boolean()
            && ["author", "name", "minAppVersion", "description"]
                .iter()
                .all(|k| m[k].as_str().is_some_and(|s| !s.trim().is_empty())),
        "Valid root Obsidian manifest; version x.y.z without v.",
    );
    boolean(
        &mut checks,
        "version-agreement",
        local["package"]["version"] == m["version"]
            && local["lock"]["version"] == m["version"]
            && local["lock"]["packages"][""]["version"] == m["version"]
            && local["versions"][version] == m["minAppVersion"],
        "Manifest/package/lock roots/current versions map must agree.",
    );
    boolean(
        &mut checks,
        "candidate-binding",
        local["candidateBound"] == true && local["clean"] == true,
        "Clean candidate checkout or proved runtime-equivalent docs-only HEAD; use an isolated worktree for unrelated edits.",
    );
    boolean(
        &mut checks,
        "local-assets",
        local["assets"]["main.js"].is_string() && local["assets"]["manifest.json"].is_string(),
        "main.js and manifest.json required; styles.css is conditional on use.",
    );
    boolean(
        &mut checks,
        "documentation",
        local["docsPresent"] == true,
        "README/license/security/contributing/changelog release contract.",
    );
    boolean(
        &mut checks,
        "official-lint-configuration",
        local["officialLintConfigured"] == true,
        "Official lint dependency/config required; its actual run is separate evidence.",
    );
    if local["matchAll"] == true {
        check(
            &mut checks,
            "runtime-library-types",
            match local["es2020Library"].as_bool() {
                Some(true) => "pass",
                Some(false) => "fail",
                None if evidence["effectiveTypeScript"]["commit"] == local["commit"]
                    && evidence["effectiveTypeScript"]["libraries"]
                        .as_array()
                        .is_some_and(|rows| rows.iter().filter_map(Value::as_str).any(es2020)) =>
                {
                    "pass"
                }
                None => "pending",
            },
            "String.matchAll needs ES2020.String or a later effective library; resolve JSONC/extends explicitly. Do not rely on accidental Node globals.",
        );
    }
    boolean(
        &mut checks,
        "build-portability",
        !local["package"]["scripts"]["build"]
            .as_str()
            .is_some_and(|s| s.trim_start().starts_with("NODE_ENV=")),
        "Use a portable build command, not a shell-specific environment prefix.",
    );
    if private_preview {
        check(
            &mut checks,
            "public-repository",
            "expected",
            "Explicit local-private preview: public source is not asserted or verified. Official review still requires public-review intent.",
        );
    } else if remote["repo"]["state"] != "ok" {
        check(
            &mut checks,
            "public-repository",
            "pending",
            "Repository metadata unavailable; an auth/network error is not a missing release.",
        );
    } else {
        boolean(
            &mut checks,
            "public-repository",
            remote["repo"]["value"]["private"] == false,
            "Community reviewers and anonymous users need the intended public source.",
        );
    }
    proof(&mut checks, evidence, "localValidation", &local["commit"]);
    proof(&mut checks, evidence, "hostValidation", &local["commit"]);
    proof(&mut checks, evidence, "independentReview", &local["commit"]);
    if evidence["candidate"]["bundleSHA256"] != local["assets"]["main.js"] {
        check(
            &mut checks,
            "evidence-bundle-binding",
            "pending",
            "Supplied run evidence must bind the exact main.js SHA-256.",
        );
    } else {
        check(
            &mut checks,
            "evidence-bundle-binding",
            "pass",
            "Supplied evidence names the exact local bundle.",
        );
    }
    check(
        &mut checks,
        "local-command-evidence",
        if [
            "npmCi",
            "officialLint",
            "typecheck",
            "tests",
            "build",
            "productionAudit",
            "typesResolved",
        ]
        .iter()
        .all(|key| evidence["localValidation"]["checks"][key] == true)
        {
            "pass"
        } else {
            "pending"
        },
        "Record actual clean install/lint/typecheck/tests/build/production-audit and type-resolution results, not a single build-pass claim.",
    );
    let host = &evidence["hostValidation"];
    check(
        &mut checks,
        "host-evidence",
        if host["pluginVersion"] == m["version"]
            && host["payloadSHA256"] == local["assets"]["main.js"]
            && ["appVersion", "platform"]
                .iter()
                .all(|k| host[k].as_str().is_some_and(|s| !s.is_empty()))
            && ["settingsRestored", "sourcePreserved", "unloadPassed"]
                .iter()
                .all(|k| host[k] == true)
        {
            "pass"
        } else {
            "pending"
        },
        "Host/version/payload, source preservation, settings and unload must be evidenced; build or synthetic-only success is insufficient.",
    );
    if phase == "source-preview" {
        check(
            &mut checks,
            "public-release",
            if !private_preview && remote["release"]["state"] == "error" {
                "pending"
            } else {
                "expected"
            },
            "A release is not required for source preview; preview acceptance does not establish a public release or permission to submit a new plugin.",
        );
    } else {
        check(
            &mut checks,
            "default-branch-manifest",
            if remote["defaultManifestState"] == "error" || remote["repo"]["state"] != "ok" {
                "pending"
            } else if remote["defaultManifest"] == *m {
                "pass"
            } else {
                "fail"
            },
            "Community reads the default branch manifest, not your local branch or release title.",
        );
        let release = &remote["release"];
        if release["state"] != "ok" {
            check(
                &mut checks,
                "matching-public-release",
                if release["state"] == "missing"
                    && remote["repo"]["state"] == "ok"
                    && remote["repo"]["value"]["private"] == false
                {
                    "fail"
                } else {
                    "pending"
                },
                "No accessible exact matching release. Require tag equal to manifest version; distinguish missing from auth/network failure.",
            );
        } else {
            boolean(
                &mut checks,
                "matching-public-release",
                release["value"]["tag_name"] == m["version"]
                    && release["value"]["draft"] == false
                    && release["value"]["prerelease"] == false,
                "Exact tag; public, non-draft and stable. A tag alone or authenticated draft is insufficient.",
            );
            check(
                &mut checks,
                "dereferenced-tag-commit",
                if remote["tagState"] == "error" {
                    "pending"
                } else if remote["tagCommit"] == local["commit"] {
                    "pass"
                } else {
                    "fail"
                },
                "Dereference annotated tags; target_commitish=main is not source binding.",
            );
            for (name, hash) in local["assets"]
                .as_object()
                .into_iter()
                .flat_map(|a| a.iter())
            {
                let download = &remote["downloads"][name];
                check(
                    &mut checks,
                    &format!("anonymous-asset:{name}"),
                    if download["state"] == "error" {
                        "pending"
                    } else if download["hash"] == *hash {
                        "pass"
                    } else {
                        "fail"
                    },
                    "Anonymous downloaded bytes must match the tested candidate; missing or changed assets block release acceptance.",
                );
            }
            check(
                &mut checks,
                "downloaded-manifest",
                if remote["downloads"]["manifest.json"]["state"] == "error" {
                    "pending"
                } else if remote["downloads"]["manifest.json"]["manifest"] == *m {
                    "pass"
                } else {
                    "fail"
                },
                "The release manifest asset must match the committed/local manifest.",
            );
        }
        if initial {
            check(
                &mut checks,
                "sourcePreview",
                "expected",
                "For a first submission publish/check the GitHub release before registration. Do not require an unavailable pre-registration dashboard preview.",
            );
        } else if evidence["sourcePreview"]["previewIntent"]
            .as_str()
            .is_some_and(|intent| intent != "public-review")
            || evidence["sourcePreview"]["claimCeiling"] == "private-local-preview-only"
        {
            check(
                &mut checks,
                "sourcePreview",
                "pending",
                "Private local preview evidence cannot establish official public source review.",
            );
        } else {
            proof(&mut checks, evidence, "sourcePreview", &local["commit"]);
        }
    }
    if phase == "catalog-install" {
        proof(&mut checks, evidence, "releaseReview", &local["commit"]);
        let catalog = &remote["catalog"];
        check(
            &mut checks,
            "catalog-registration",
            if catalog["state"] != "ok" {
                "pending"
            } else if catalog["entry"]["repo"] == repo {
                "pass"
            } else {
                "pending"
            },
            "Correct ID/repository in official app registry; new-ID propagation is pending, never a successful install.",
        );
        check(
            &mut checks,
            "listing-version",
            if catalog["listingVersion"] == m["version"] {
                "pass"
            } else {
                "pending"
            },
            "Public Community page must show the intended version; missing/changed HTML needs direct review.",
        );
        check(
            &mut checks,
            "installed-payload",
            if local["installed"]["state"] != "read" {
                "pending"
            } else if local["installed"]["manifest"] == *m
                && local["assets"].as_object().is_some_and(|a| {
                    a.iter().all(|(n, h)| {
                        local["installed"]["hashes"][n] == *h
                            || (n == "main.js" && local["installed"]["suffixHashes"][n] == *h)
                    })
                })
            {
                "pass"
            } else {
                "fail"
            },
            "Installed files must match exactly; only the observed exact nosourcemap suffix is accepted.",
        );
        proof(&mut checks, evidence, "catalogInstall", &local["commit"]);
        if evidence["catalogInstall"]["channel"] != "community"
            || evidence["catalogInstall"]["fresh"] != true
        {
            check(
                &mut checks,
                "install-channel",
                "pending",
                "Matching manual files do not prove a fresh Community installation.",
            );
        }
    }
    let passed = checks
        .iter()
        .all(|c| c["status"] == "pass" || c["status"] == "expected");
    json!({"schema":"viggobsidian.obsidian-plugin-preflight.v1","phase":phase,"previewIntent":intent.name(),"initialSubmission":initial,"repository":repo,"version":m["version"],"candidateCommit":local["commit"],"bundleSHA256":local["assets"]["main.js"],"phasePassed":passed,"deliveryComplete":passed&&phase=="catalog-install","publicAvailabilityVerified":passed&&matches!(phase,"public-release"|"catalog-install"),"claimCeiling":if private_preview{"private-local-preview-only"}else{match phase{"source-preview"=>"ready-for-source-preview-only","public-release"=>"public-GitHub-release-only",_=>"catalog-installation-acceptance"}},"evidenceAuthority":"Local/host/review/channel statuses are supplied evidence, not newly executed or independently certified by this checker.","readOnly":true,"checks":checks})
}
pub(crate) fn run(args: &[String]) -> Result<Value> {
    let get = |key: &str| -> Result<&str> {
        let i = args
            .iter()
            .position(|a| a == key)
            .ok_or_else(|| format!("missing {key}"))?;
        Ok(args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {key}"))?)
    };
    let repo = get("--github")?;
    let parts: Vec<_> = repo.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|s| {
            s.is_empty()
                || !s
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        })
    {
        return Err("--github must be OWNER/REPOSITORY".into());
    }
    let phase = get("--phase")?;
    if !["source-preview", "public-release", "catalog-install"].contains(&phase) {
        return Err("invalid plugin-check phase".into());
    }
    let intent = PreviewIntent::from_args(args, phase)?;
    let dir = super::root(Path::new(get("--repo-dir")?))?;
    let candidate = if args.iter().any(|a| a == "--candidate") {
        get("--candidate")?
    } else {
        "HEAD"
    };
    let installed = if args.iter().any(|a| a == "--installed-dir") {
        Some(Path::new(get("--installed-dir")?))
    } else {
        None
    };
    let evidence = if args.iter().any(|a| a == "--evidence") {
        serde_json::from_slice(&fs::read(get("--evidence")?)?)?
    } else {
        json!({})
    };
    let local = local(&dir, candidate, installed)?;
    let remote = if intent == PreviewIntent::LocalPrivate {
        json!({"state":"not-requested","reason":"explicit local-private preview"})
    } else {
        remote(repo, &local, phase)
    };
    Ok(evaluate(
        &local,
        &remote,
        &evidence,
        phase,
        args.iter().any(|a| a == "--initial"),
        repo,
        intent,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Value, Value, Value) {
        let m = json!({"id":"example-viewer","name":"Example Viewer","author":"Example","description":"Inspect files.","minAppVersion":"1.5.0","version":"1.2.3","isDesktopOnly":false});
        let local = json!({"manifest":m,"package":{"version":"1.2.3","scripts":{"build":"node build.mjs"}},"lock":{"version":"1.2.3","packages":{"":{"version":"1.2.3"}}},"versions":{"1.2.3":"1.5.0"},"assets":{"main.js":"hash-js","manifest.json":"hash-manifest"},"commit":"commit-a","candidateBound":true,"clean":true,"matchAll":true,"es2020Library":true,"officialLintConfigured":true,"docsPresent":true,"installed":{"state":"read","manifest":m,"hashes":{"main.js":"hash-js","manifest.json":"hash-manifest"}}});
        let remote = json!({"repo":{"state":"ok","value":{"private":false}},"release":{"state":"ok","value":{"tag_name":"1.2.3","draft":false,"prerelease":false,"target_commitish":"main"}},"tagCommit":"commit-a","defaultManifest":m,"downloads":{"main.js":{"state":"ok","hash":"hash-js"},"manifest.json":{"state":"ok","hash":"hash-manifest","manifest":m}},"catalog":{"state":"ok","entry":{"repo":"example/plugin"},"listingVersion":"1.2.3"}});
        let mut evidence = json!({"candidate":{"bundleSHA256":"hash-js"}});
        for k in [
            "localValidation",
            "hostValidation",
            "independentReview",
            "sourcePreview",
            "releaseReview",
            "catalogInstall",
        ] {
            evidence[k] = json!({"status":"pass","commit":"commit-a"});
        }
        evidence["localValidation"]["checks"] = json!({"npmCi":true,"officialLint":true,"typecheck":true,"tests":true,"build":true,"productionAudit":true,"typesResolved":true});
        evidence["hostValidation"]["pluginVersion"] = json!("1.2.3");
        evidence["hostValidation"]["payloadSHA256"] = json!("hash-js");
        evidence["hostValidation"]["appVersion"] = json!("1.15.0");
        evidence["hostValidation"]["platform"] = json!("fixture");
        for k in ["settingsRestored", "sourcePreserved", "unloadPassed"] {
            evidence["hostValidation"][k] = json!(true);
        }
        evidence["catalogInstall"]["channel"] = json!("community");
        evidence["catalogInstall"]["fresh"] = json!(true);
        (local, remote, evidence)
    }
    fn result(l: &Value, r: &Value, e: &Value, phase: &str, initial: bool) -> Value {
        evaluate(
            l,
            r,
            e,
            phase,
            initial,
            "example/plugin",
            PreviewIntent::PublicReview,
        )
    }
    fn status(out: &Value, id: &str) -> String {
        out["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == id)
            .unwrap()["status"]
            .as_str()
            .unwrap()
            .into()
    }
    #[test]
    fn source_preview_missing_release_is_expected_but_never_delivery() {
        let (l, mut r, e) = fixture();
        r["release"] = json!({"state":"missing"});
        let out = result(&l, &r, &e, "source-preview", false);
        assert_eq!(out["phasePassed"], true);
        assert_eq!(out["deliveryComplete"], false);
        assert_eq!(status(&out, "public-release"), "expected");
    }
    #[test]
    fn private_preview_accepts_local_proof_without_asserting_visibility_or_delivery() {
        let (l, mut r, e) = fixture();
        for metadata in [
            json!({"state":"ok","value":{"private":true}}),
            json!({"state":"error","detail":"HTTP 403"}),
            Value::Null,
        ] {
            r["repo"] = metadata;
            r["release"] = json!({"state":"error"});
            let out = evaluate(
                &l,
                &r,
                &e,
                "source-preview",
                false,
                "example/plugin",
                PreviewIntent::LocalPrivate,
            );
            assert_eq!(out["phasePassed"], true);
            assert_eq!(out["previewIntent"], "local-private");
            assert_eq!(out["claimCeiling"], "private-local-preview-only");
            assert_eq!(out["deliveryComplete"], false);
            assert_eq!(out["publicAvailabilityVerified"], false);
            assert_eq!(status(&out, "public-repository"), "expected");
        }
    }
    #[test]
    fn official_preview_still_requires_public_source_and_exact_local_proof() {
        let (l, mut r, e) = fixture();
        r["repo"]["value"]["private"] = json!(true);
        let out = result(&l, &r, &e, "source-preview", false);
        assert_eq!(status(&out, "public-repository"), "fail");
        assert_eq!(out["phasePassed"], false);
        r["repo"]["value"]["private"] = json!(false);
        let out = result(&l, &r, &e, "source-preview", false);
        assert_eq!(out["phasePassed"], true);
        assert_eq!(out["previewIntent"], "public-review");
        assert_eq!(out["publicAvailabilityVerified"], false);
    }
    #[test]
    fn private_preview_does_not_bypass_candidate_hash_host_or_review_checks() {
        let (l, r, e) = fixture();
        for defect in ["candidate", "hash", "host", "review"] {
            let mut local = l.clone();
            let mut evidence = e.clone();
            match defect {
                "candidate" => local["candidateBound"] = json!(false),
                "hash" => evidence["candidate"]["bundleSHA256"] = json!("other"),
                "host" => evidence["hostValidation"]["unloadPassed"] = json!(false),
                _ => evidence["independentReview"]["commit"] = json!("old"),
            }
            let out = evaluate(
                &local,
                &r,
                &evidence,
                "source-preview",
                false,
                "example/plugin",
                PreviewIntent::LocalPrivate,
            );
            assert_eq!(out["phasePassed"], false, "{defect}");
        }
    }
    #[test]
    fn private_intent_cannot_weaken_publication_even_outside_cli() {
        let (l, mut r, e) = fixture();
        for phase in ["public-release", "catalog-install"] {
            assert!(PreviewIntent::parse("local-private", phase).is_err());
            let out = evaluate(
                &l,
                &r,
                &e,
                phase,
                true,
                "example/plugin",
                PreviewIntent::LocalPrivate,
            );
            assert_eq!(status(&out, "preview-intent-scope"), "fail");
            assert_eq!(out["phasePassed"], false);
            assert_eq!(out["deliveryComplete"], false);
            assert_eq!(out["publicAvailabilityVerified"], false);
            r["repo"]["value"]["private"] = json!(true);
            let out = result(&l, &r, &e, phase, true);
            assert_eq!(status(&out, "public-repository"), "fail");
            assert_eq!(out["phasePassed"], false);
            r["repo"]["value"]["private"] = json!(false);
        }
        assert!(PreviewIntent::parse("skip", "source-preview").is_err());
    }
    #[test]
    fn cli_rejects_ambiguous_or_invalid_intent_before_collection() {
        // This sentinel cannot pass root resolution: receiving the exact intent
        // error proves collection never reaches filesystem or command access.
        let prefix = [
            "--github",
            "example/plugin",
            "--repo-dir",
            "/nonexistent-plugin-check-collection-sentinel",
            "--phase",
            "source-preview",
        ];
        for (options, expected) in [
            (
                vec!["--preview-intent"],
                "missing value for --preview-intent",
            ),
            (
                vec!["--preview-intent", "--initial"],
                "missing value for --preview-intent",
            ),
            (
                vec!["--preview-intent=nonsense"],
                "invalid --preview-intent; use public-review or local-private",
            ),
            (
                vec!["--preview-intent="],
                "invalid --preview-intent; use public-review or local-private",
            ),
            (
                vec!["--preview-intent", "nonsense"],
                "invalid --preview-intent; use public-review or local-private",
            ),
            (
                vec![
                    "--preview-intent",
                    "local-private",
                    "--preview-intent",
                    "public-review",
                ],
                "duplicate --preview-intent",
            ),
            (
                vec![
                    "--preview-intent=local-private",
                    "--preview-intent=public-review",
                ],
                "duplicate --preview-intent",
            ),
            (
                vec![
                    "--preview-intent=local-private",
                    "--preview-intent",
                    "local-private",
                ],
                "duplicate --preview-intent",
            ),
        ] {
            let args: Vec<_> = prefix
                .iter()
                .copied()
                .chain(options)
                .map(String::from)
                .collect();
            assert_eq!(run(&args).unwrap_err().to_string(), expected);
        }
        let args: Vec<_> = [
            "--github",
            "example/plugin",
            "--phase",
            "public-release",
            "--preview-intent=local-private",
        ]
        .iter()
        .map(|s| String::from(*s))
        .collect();
        assert_eq!(
            run(&args).unwrap_err().to_string(),
            "local-private intent is only valid for source-preview"
        );
    }
    #[test]
    fn cli_intent_forms_and_default_are_unambiguous() {
        assert!(
            PreviewIntent::from_args(&[], "source-preview").unwrap() == PreviewIntent::PublicReview
        );
        for args in [
            vec!["--preview-intent", "local-private"],
            vec!["--preview-intent=local-private"],
        ] {
            let args: Vec<_> = args.into_iter().map(String::from).collect();
            assert!(
                PreviewIntent::from_args(&args, "source-preview").unwrap()
                    == PreviewIntent::LocalPrivate
            );
        }
    }
    #[test]
    fn publication_claims_require_public_release_and_actual_catalog_install() {
        let (l, r, mut e) = fixture();
        let release = result(&l, &r, &e, "public-release", true);
        assert_eq!(release["publicAvailabilityVerified"], true);
        assert_eq!(release["deliveryComplete"], false);
        let catalog = result(&l, &r, &e, "catalog-install", true);
        assert_eq!(catalog["publicAvailabilityVerified"], true);
        assert_eq!(catalog["deliveryComplete"], true);
        e["catalogInstall"]["channel"] = json!("manual");
        let manual = result(&l, &r, &e, "catalog-install", true);
        assert_eq!(manual["deliveryComplete"], false);
        assert_eq!(manual["publicAvailabilityVerified"], false);
    }
    #[test]
    fn private_preview_evidence_cannot_substitute_for_official_source_review() {
        let (l, r, e) = fixture();
        for (field, value) in [
            ("previewIntent", "local-private"),
            ("claimCeiling", "private-local-preview-only"),
        ] {
            let mut evidence = e.clone();
            evidence["sourcePreview"][field] = json!(value);
            for phase in ["public-release", "catalog-install"] {
                let out = result(&l, &r, &evidence, phase, false);
                assert_eq!(status(&out, "sourcePreview"), "pending");
                assert_eq!(out["phasePassed"], false);
            }
        }
    }
    #[test]
    fn initial_submission_requires_public_release_without_impossible_preview() {
        let (l, r, mut e) = fixture();
        e.as_object_mut().unwrap().remove("sourcePreview");
        assert_eq!(
            result(&l, &r, &e, "public-release", true)["phasePassed"],
            true
        );
        assert_eq!(
            result(&l, &r, &e, "public-release", false)["phasePassed"],
            false
        );
    }
    #[test]
    fn missing_release_blocks_submission() {
        let (l, mut r, e) = fixture();
        r["release"] = json!({"state":"missing"});
        let out = result(&l, &r, &e, "public-release", true);
        assert_eq!(status(&out, "matching-public-release"), "fail");
        assert_eq!(out["phasePassed"], false);
    }
    #[test]
    fn inaccessible_is_not_misreported_as_missing() {
        let (l, mut r, e) = fixture();
        r["release"] = json!({"state":"error","detail":"HTTP 403"});
        assert_eq!(
            status(
                &result(&l, &r, &e, "public-release", true),
                "matching-public-release"
            ),
            "pending"
        );
    }
    #[test]
    fn draft_prerelease_and_v_prefix_are_not_stable_publication() {
        for key in ["draft", "prerelease", "tag_name"] {
            let (l, mut r, e) = fixture();
            r["release"]["value"][key] = if key == "tag_name" {
                json!("v1.2.3")
            } else {
                json!(true)
            };
            assert_eq!(
                status(
                    &result(&l, &r, &e, "public-release", true),
                    "matching-public-release"
                ),
                "fail"
            );
        }
    }
    #[test]
    fn tag_commit_must_match_even_when_target_is_main() {
        let (l, mut r, e) = fixture();
        r["tagCommit"] = json!("other-commit");
        assert_eq!(
            status(
                &result(&l, &r, &e, "public-release", true),
                "dereferenced-tag-commit"
            ),
            "fail"
        );
    }
    #[test]
    fn wrong_default_manifest_is_not_hidden_by_correct_release() {
        let (l, mut r, e) = fixture();
        r["defaultManifest"]["version"] = json!("1.2.4");
        assert_eq!(
            status(
                &result(&l, &r, &e, "public-release", true),
                "default-branch-manifest"
            ),
            "fail"
        );
    }
    #[test]
    fn wrong_missing_or_unreachable_asset_is_distinct() {
        for value in [
            Value::Null,
            json!({"state":"ok","hash":"different"}),
            json!({"state":"error"}),
        ] {
            let (l, mut r, e) = fixture();
            r["downloads"]["main.js"] = value;
            let out = result(&l, &r, &e, "public-release", true);
            assert!(!out["phasePassed"].as_bool().unwrap());
            assert_eq!(
                status(&out, "anonymous-asset:main.js"),
                if r["downloads"]["main.js"]["state"] == "error" {
                    "pending"
                } else {
                    "fail"
                }
            );
        }
    }
    #[test]
    fn downloaded_manifest_payload_must_match() {
        let (l, mut r, e) = fixture();
        r["downloads"]["manifest.json"]["manifest"]["version"] = json!("1.2.2");
        assert_eq!(
            status(
                &result(&l, &r, &e, "public-release", true),
                "downloaded-manifest"
            ),
            "fail"
        );
    }
    #[test]
    fn css_is_optional_but_must_match_when_used() {
        let (mut l, mut r, e) = fixture();
        assert_eq!(
            result(&l, &r, &e, "public-release", true)["phasePassed"],
            true
        );
        l["assets"]["styles.css"] = json!("css");
        assert_eq!(
            result(&l, &r, &e, "public-release", true)["phasePassed"],
            false
        );
        r["downloads"]["styles.css"] = json!({"state":"ok","hash":"css"});
        assert_eq!(
            result(&l, &r, &e, "public-release", true)["phasePassed"],
            true
        );
    }
    #[test]
    fn implicit_node_types_cannot_mask_es2018_match_all() {
        let (mut l, r, e) = fixture();
        l["es2020Library"] = json!(false);
        assert_eq!(
            status(
                &result(&l, &r, &e, "source-preview", false),
                "runtime-library-types"
            ),
            "fail"
        );
        l["es2020Library"] = Value::Null;
        assert_eq!(
            status(
                &result(&l, &r, &e, "source-preview", false),
                "runtime-library-types"
            ),
            "pending"
        );
    }
    #[test]
    fn version_lock_drift_and_unbound_source_fail() {
        for key in ["version", "source"] {
            let (mut l, r, e) = fixture();
            if key == "version" {
                l["lock"]["packages"][""]["version"] = json!("old");
            } else {
                l["candidateBound"] = json!(false);
            }
            assert_eq!(
                result(&l, &r, &e, "source-preview", false)["phasePassed"],
                false
            );
        }
    }
    #[test]
    fn unresolved_effective_types_need_current_commit_and_sufficient_libraries() {
        let (mut l, r, mut e) = fixture();
        l["es2020Library"] = Value::Null;
        for (commit, libraries, expected) in [
            ("old", json!(["es2020"]), "pending"),
            ("commit-a", json!(["dom", "es2018"]), "pending"),
            ("commit-a", json!(["dom", "es2020.string"]), "pass"),
        ] {
            e["effectiveTypeScript"] = json!({"commit":commit,"libraries":libraries});
            assert_eq!(
                status(
                    &result(&l, &r, &e, "source-preview", false),
                    "runtime-library-types"
                ),
                expected
            );
        }
        l["es2020Library"] = json!(false);
        assert_eq!(
            status(
                &result(&l, &r, &e, "source-preview", false),
                "runtime-library-types"
            ),
            "fail"
        );
    }
    #[test]
    fn collection_errors_remain_pending_instead_of_false_mismatches() {
        for (field, check_id) in [
            ("defaultManifestState", "default-branch-manifest"),
            ("tagState", "dereferenced-tag-commit"),
        ] {
            let (l, mut r, e) = fixture();
            r[field] = json!("error");
            assert_eq!(
                status(&result(&l, &r, &e, "public-release", true), check_id),
                "pending"
            );
        }
        let (l, mut r, e) = fixture();
        r["downloads"]["manifest.json"] = json!({"state":"error"});
        assert_eq!(
            status(
                &result(&l, &r, &e, "public-release", true),
                "downloaded-manifest"
            ),
            "pending"
        );
        r["release"] = json!({"state":"missing"});
        r["repo"] = json!({"state":"error","detail":"HTTP 403"});
        assert_eq!(
            status(
                &result(&l, &r, &e, "public-release", true),
                "matching-public-release"
            ),
            "pending"
        );
    }
    #[test]
    fn matching_manual_install_never_proves_catalog() {
        let (l, r, mut e) = fixture();
        e["catalogInstall"]["channel"] = json!("manual");
        let out = result(&l, &r, &e, "catalog-install", true);
        assert_eq!(out["deliveryComplete"], false);
        assert_eq!(status(&out, "install-channel"), "pending");
    }
    #[test]
    fn propagation_is_pending_not_build_failure() {
        let (l, mut r, e) = fixture();
        r["catalog"]["entry"] = Value::Null;
        let out = result(&l, &r, &e, "catalog-install", true);
        assert_eq!(status(&out, "catalog-registration"), "pending");
        assert_eq!(status(&out, "matching-public-release"), "pass");
        assert_eq!(out["deliveryComplete"], false);
    }
    #[test]
    fn complete_catalog_evidence_is_required_for_delivery() {
        let (l, r, e) = fixture();
        assert_eq!(
            result(&l, &r, &e, "catalog-install", true)["deliveryComplete"],
            true
        );
        let mut stale = e.clone();
        stale["releaseReview"]["commit"] = json!("old");
        assert_eq!(
            result(&l, &r, &stale, "catalog-install", true)["deliveryComplete"],
            false
        );
    }
    #[test]
    fn a_build_only_claim_does_not_replace_host_checks() {
        let (l, r, mut e) = fixture();
        e["hostValidation"]["unloadPassed"] = json!(false);
        assert_eq!(
            status(
                &result(&l, &r, &e, "source-preview", false),
                "host-evidence"
            ),
            "pending"
        );
    }
    #[test]
    fn shell_build_prefix_is_caught() {
        let (mut l, r, e) = fixture();
        l["package"]["scripts"]["build"] = json!("NODE_ENV=production node build.mjs");
        assert_eq!(
            status(
                &result(&l, &r, &e, "source-preview", false),
                "build-portability"
            ),
            "fail"
        );
    }
    #[test]
    fn exact_nosourcemap_suffix_only() {
        let payload = b"let a=1;";
        let mut installed = payload.to_vec();
        installed.extend_from_slice(SUFFIX);
        assert_eq!(
            digest(&installed[..installed.len() - SUFFIX.len()]),
            digest(payload)
        );
        let mut changed = payload.to_vec();
        changed.extend_from_slice(b"\n/* nosourcemap */\n");
        assert!(!changed.ends_with(SUFFIX));
    }
}

#[cfg(test)]
mod collector_tests {
    use super::*;
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "obsidian-preflight-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            fs::create_dir(path.join("repo")).unwrap();
            fs::create_dir(path.join("installed")).unwrap();
            let repo = path.join("repo");
            for (name, text) in [
                (
                    "manifest.json",
                    r#"{"id":"test-viewer","name":"Test Viewer","author":"Example","version":"1.0.0","minAppVersion":"1.5.0","isDesktopOnly":false,"description":"Inspect files."}"#,
                ),
                (
                    "package.json",
                    r#"{"version":"1.0.0","devDependencies":{"eslint-plugin-obsidianmd":"1"}}"#,
                ),
                (
                    "package-lock.json",
                    r#"{"version":"1.0.0","packages":{"":{"version":"1.0.0"}}}"#,
                ),
                ("versions.json", r#"{"1.0.0":"1.5.0"}"#),
                (
                    "tsconfig.json",
                    r#"{"compilerOptions":{"lib":["dom","es2020"]}}"#,
                ),
                ("eslint.config.mjs", "export default [];"),
                ("main.js", "let a=1;\n/* nosourcemap */"),
                ("README.md", "Example"),
                ("LICENSE", "MIT"),
                ("SECURITY.md", "Example"),
                ("CONTRIBUTING.md", "Example"),
                ("CHANGELOG.md", "Example"),
            ] {
                fs::write(repo.join(name), text).unwrap();
            }
            git(&repo, &["init", "--quiet"]).unwrap();
            git(&repo, &["config", "--local", "user.name", "Example"]).unwrap();
            git(
                &repo,
                &["config", "--local", "user.email", "example@example.invalid"],
            )
            .unwrap();
            git(&repo, &["config", "--local", "core.hooksPath", ".no-hooks"]).unwrap();
            git(&repo, &["add", "."]).unwrap();
            git(&repo, &["commit", "--quiet", "-m", "fixture"]).unwrap();
            Self(path)
        }
        fn repo(&self) -> PathBuf {
            self.0.join("repo")
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn collector_checks_all_runtime_changes_but_allows_docs_only() {
        let fixture = Fixture::new();
        let repo = fixture.repo();
        let candidate = git(&repo, &["rev-parse", "HEAD"]).unwrap();
        fs::write(repo.join("README.md"), "Updated docs").unwrap();
        git(&repo, &["add", "."]).unwrap();
        git(&repo, &["commit", "--quiet", "-m", "docs"]).unwrap();
        assert_eq!(
            local(&repo, &candidate, None).unwrap()["candidateBound"],
            true
        );
        for file in [
            "docs/parser.ts",
            "docs/build.mjs",
            "scripts/build.mjs",
            "lib/parser.ts",
            "rollup.config.mjs",
            "eslint.config.mjs",
        ] {
            let full = repo.join(file);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, "// changed runtime or build input").unwrap();
            git(&repo, &["add", "."]).unwrap();
            git(&repo, &["commit", "--quiet", "-m", "runtime"]).unwrap();
            assert_eq!(
                local(&repo, &candidate, None).unwrap()["candidateBound"],
                false,
                "missed {file}"
            );
        }
    }
    #[test]
    fn collector_keeps_raw_bytes_then_one_exact_suffix_alternative() {
        let fixture = Fixture::new();
        let repo = fixture.repo();
        let installed = fixture.0.join("installed");
        for name in ["main.js", "manifest.json"] {
            fs::copy(repo.join(name), installed.join(name)).unwrap();
        }
        let initial = local(&repo, "HEAD", Some(&installed)).unwrap();
        assert_eq!(
            initial["assets"]["main.js"],
            initial["installed"]["hashes"]["main.js"]
        );
        let mut bytes = fs::read(repo.join("main.js")).unwrap();
        bytes.extend_from_slice(SUFFIX);
        fs::write(installed.join("main.js"), &bytes).unwrap();
        let transformed = local(&repo, "HEAD", Some(&installed)).unwrap();
        assert_ne!(
            transformed["assets"]["main.js"],
            transformed["installed"]["hashes"]["main.js"]
        );
        assert_eq!(
            transformed["assets"]["main.js"],
            transformed["installed"]["suffixHashes"]["main.js"]
        );
        bytes.extend_from_slice(b"\n");
        fs::write(installed.join("main.js"), bytes).unwrap();
        let wrong = local(&repo, "HEAD", Some(&installed)).unwrap();
        assert_ne!(
            wrong["assets"]["main.js"],
            wrong["installed"]["hashes"]["main.js"]
        );
        assert_eq!(wrong["installed"]["suffixHashes"]["main.js"], Value::Null);
    }
    #[test]
    fn annotated_tags_are_dereferenced_and_collection_failure_survives() {
        let first = json!({"state":"ok","value":{"object":{"type":"tag","sha":"tag-one"}}});
        let result = dereference_tag(first.clone(), |sha| {
            if sha == "tag-one" {
                json!({"state":"ok","value":{"object":{"type":"tag","sha":"tag-two"}}})
            } else {
                json!({"state":"ok","value":{"object":{"type":"commit","sha":"actual-commit"}}})
            }
        });
        assert_eq!(result["value"]["object"]["sha"], "actual-commit");
        assert_eq!(
            dereference_tag(first, |_| json!({"state":"error","detail":"HTTP403"}))["state"],
            "error"
        );
    }
    #[test]
    fn private_cli_collects_local_candidate_without_remote_metadata() {
        let fixture = Fixture::new();
        let output = run(&[
            "--github".into(),
            "example/not-a-published-plugin".into(),
            "--repo-dir".into(),
            fixture
                .repo()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .into(),
            "--phase".into(),
            "source-preview".into(),
            "--preview-intent".into(),
            "local-private".into(),
        ])
        .unwrap();
        assert_eq!(output["previewIntent"], "local-private");
        assert_eq!(
            output["candidateCommit"],
            git(&fixture.repo(), &["rev-parse", "HEAD"]).unwrap()
        );
        assert_eq!(output["phasePassed"], false); // Run/review proof was not supplied.
        assert_eq!(output["publicAvailabilityVerified"], false);
        assert_eq!(output["deliveryComplete"], false);
    }
}
