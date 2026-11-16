use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::{Result, absolute_without_symlinks, checked_child, dossier_root, safe_relative};

const SCHEMA: &str = "viggobsidian.dossier-selection.v1";
const MAX_DISCOVERY_DEPTH: usize = 6;
const MAX_DISCOVERY_ENTRIES: usize = 1_024;
const MAX_CANDIDATES: usize = 128;
const MAX_FOLDER_NOTE_BYTES: u64 = 64 * 1024;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Candidate {
    path: String,
    title: String,
}

impl Candidate {
    fn json(&self) -> Value {
        json!({"path": self.path, "title": self.title})
    }
}

fn response(
    status: &str,
    dossier: Option<&Candidate>,
    reason: &str,
    candidates: &[Candidate],
) -> Value {
    let mut output = json!({
        "schema": SCHEMA,
        "status": status,
        "reason": reason,
        "discovery_complete": true,
        "candidates": candidates.iter().map(Candidate::json).collect::<Vec<_>>(),
    });
    if let Some(dossier) = dossier {
        output["dossier"] = json!(dossier.path);
    }
    output
}

fn incomplete_discovery_response(candidates: &[Candidate]) -> Value {
    let mut output = response(
        "needs_selection",
        None,
        "Dossier discovery did not complete within bounded directory limits; provide --dossier or --context-path.",
        candidates,
    );
    output["discovery_complete"] = json!(false);
    output
}

fn real_directory(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(!metadata.file_type().is_symlink() && metadata.is_dir()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("cannot inspect {}: {error}", path.display())),
    }
}

fn visible_normal_file(path: &Path) -> Result<Option<Vec<u8>>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("cannot inspect {}: {error}", path.display())),
    };
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_FOLDER_NOTE_BYTES
    {
        return Ok(None);
    }
    fs::read(path)
        .map(Some)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn folder_note_path(directory: &Path) -> Option<PathBuf> {
    let name = directory.file_name()?.to_str()?;
    Some(directory.join(format!("{name}.md")))
}

fn folder_title(directory: &Path) -> Result<String> {
    let fallback = directory
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("dossier")
        .to_owned();
    let Some(note) = folder_note_path(directory) else {
        return Ok(fallback);
    };
    let Some(bytes) = visible_normal_file(&note)? else {
        return Ok(fallback);
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return Ok(fallback);
    };
    let Ok((_, body, _)) = super::split_frontmatter(text) else {
        return Ok(fallback);
    };
    Ok(super::file_title(&super::visible_lines(body), &fallback))
}

fn is_direct_wiki_foldernote(directory: &Path) -> Result<bool> {
    let Some(note) = folder_note_path(directory) else {
        return Ok(false);
    };
    let Some(bytes) = visible_normal_file(&note)? else {
        return Ok(false);
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return Ok(false);
    };
    let Ok((metadata, _, _)) = super::split_frontmatter(text) else {
        return Ok(false);
    };
    Ok(metadata["layer"] == "wiki"
        && metadata["cssclasses"]
            .as_array()
            .is_some_and(|classes| classes.iter().any(|class| class == "vault-wiki-direct")))
}

fn candidate_at(vault: &Path, directory: &Path) -> Result<Option<Candidate>> {
    if !real_directory(directory)? {
        return Ok(None);
    }
    let raw = real_directory(&directory.join("raw"))?;
    let wiki = real_directory(&directory.join("wiki"))?;
    if !raw || (!wiki && !is_direct_wiki_foldernote(directory)?) {
        return Ok(None);
    }
    let path = directory
        .strip_prefix(vault)
        .map_err(|_| "dossier escaped selected vault")?
        .to_string_lossy()
        .replace('\\', "/");
    safe_relative(&path)?;
    Ok(Some(Candidate {
        path,
        title: folder_title(directory)?,
    }))
}

fn numbered_root(name: &str) -> bool {
    let mut characters = name.chars();
    let mut digits = 0usize;
    while characters
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        digits += 1;
    }
    digits >= 2 && name.as_bytes().get(digits) == Some(&b'-')
}

fn pruned_discovery_subtree(name: &str) -> bool {
    matches!(
        name,
        "raw" | "wiki" | "attachment" | "attachments" | "archive" | "archives"
    )
}

fn discover(vault: &Path) -> Result<(Vec<Candidate>, bool)> {
    fn visit(
        vault: &Path,
        directory: &Path,
        depth: usize,
        directories: &mut usize,
        candidates: &mut Vec<Candidate>,
    ) -> Result<bool> {
        if depth > MAX_DISCOVERY_DEPTH {
            return Ok(true);
        }
        for entry in fs::read_dir(directory)
            .map_err(|error| format!("cannot list {}: {error}", directory.display()))?
        {
            let entry = entry.map_err(|error| error.to_string())?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || name == ".git" {
                continue;
            }
            let kind = entry.file_type().map_err(|error| error.to_string())?;
            if kind.is_symlink() || !kind.is_dir() {
                continue;
            }
            *directories += 1;
            if *directories > MAX_DISCOVERY_ENTRIES {
                return Ok(false);
            }
            let child = entry.path();
            if let Some(candidate) = candidate_at(vault, &child)? {
                candidates.push(candidate);
                if candidates.len() > MAX_CANDIDATES {
                    return Ok(false);
                }
                continue;
            }
            if pruned_discovery_subtree(&name) {
                continue;
            }
            if !visit(vault, &child, depth + 1, directories, candidates)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    let mut directories = 0;
    let mut candidates = Vec::new();
    let mut complete = true;
    for root in
        fs::read_dir(vault).map_err(|error| format!("cannot list {}: {error}", vault.display()))?
    {
        let root = root.map_err(|error| error.to_string())?;
        let name = root.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || name == ".git" || !numbered_root(&name) {
            continue;
        }
        let kind = root.file_type().map_err(|error| error.to_string())?;
        if kind.is_symlink() || !kind.is_dir() {
            continue;
        }
        directories += 1;
        if directories > MAX_DISCOVERY_ENTRIES {
            complete = false;
            break;
        }
        let path = root.path();
        if let Some(candidate) = candidate_at(vault, &path)? {
            candidates.push(candidate);
            if candidates.len() > MAX_CANDIDATES {
                complete = false;
                break;
            }
        } else if !pruned_discovery_subtree(&name)
            && !visit(vault, &path, 1, &mut directories, &mut candidates)?
        {
            complete = false;
            break;
        }
    }
    candidates.sort();
    candidates.dedup();
    Ok((candidates, complete))
}

fn lexical_terms(text: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut current = String::new();
    for character in text.to_lowercase().chars() {
        if character.is_alphanumeric() {
            current.push(character);
        } else if !current.is_empty() {
            terms.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        terms.push(current);
    }
    terms
}

fn lexical_match(candidate: &Candidate, terms: &[String]) -> bool {
    if terms.is_empty() {
        return false;
    }
    let available: BTreeSet<_> = lexical_terms(&format!("{} {}", candidate.path, candidate.title))
        .into_iter()
        .collect();
    terms.iter().all(|term| available.contains(term))
}

fn clear_query_title_match(candidate: &Candidate, query_terms: &[String]) -> bool {
    let title: BTreeSet<_> = lexical_terms(&candidate.title).into_iter().collect();
    let query: BTreeSet<_> = query_terms.iter().cloned().collect();
    title.len() >= 2 && title.is_subset(&query)
}

fn nearest_context_candidate(vault: &Path, context: &str) -> Result<Option<Candidate>> {
    safe_relative(context)?;
    let context = checked_child(vault, context)?;
    let metadata = fs::symlink_metadata(&context)
        .map_err(|error| format!("cannot inspect context {}: {error}", context.display()))?;
    let mut cursor = if metadata.is_dir() {
        context
    } else {
        context
            .parent()
            .ok_or("context has no parent")?
            .to_path_buf()
    };
    loop {
        if let Some(candidate) = candidate_at(vault, &cursor)? {
            return Ok(Some(candidate));
        }
        if cursor == vault {
            return Ok(None);
        }
        cursor = cursor
            .parent()
            .ok_or("context escaped selected vault")?
            .to_path_buf();
        if !cursor.starts_with(vault) {
            return Err("context escaped selected vault".into());
        }
    }
}

/// Select one existing, eligible dossier inside an explicit vault without writing sources.
pub fn select_dossier(
    vault_arg: &Path,
    explicit: Option<&str>,
    context: Option<&str>,
    topic: Option<&str>,
    query: &str,
) -> Result<Value> {
    if vault_arg.as_os_str().is_empty() {
        return Err("--vault is required".into());
    }
    let vault = absolute_without_symlinks(vault_arg)?;
    if let Some(explicit) = explicit {
        safe_relative(explicit)?;
        let path = dossier_root(&vault, explicit)?;
        return match candidate_at(&vault, &path)? {
            Some(candidate) => Ok(response(
                "selected",
                Some(&candidate),
                "Selected the explicit eligible dossier.",
                std::slice::from_ref(&candidate),
            )),
            None => Ok(response(
                "not_applicable",
                None,
                "The explicit path is not an eligible dossier with actual Raw/Wiki information.",
                &[],
            )),
        };
    }
    if let Some(context) = context
        && let Some(candidate) = nearest_context_candidate(&vault, context)?
    {
        return Ok(response(
            "selected",
            Some(&candidate),
            "Selected the nearest enclosing eligible dossier from context.",
            std::slice::from_ref(&candidate),
        ));
    }

    let (candidates, discovery_complete) = discover(&vault)?;
    if !discovery_complete {
        return Ok(incomplete_discovery_response(&candidates));
    }
    if let Some(topic) = topic {
        let terms = lexical_terms(topic);
        let matches: Vec<_> = candidates
            .iter()
            .filter(|candidate| lexical_match(candidate, &terms))
            .cloned()
            .collect();
        if matches.len() == 1 {
            return Ok(response(
                "selected",
                matches.first(),
                "Selected the unique lexical topic match.",
                &matches,
            ));
        }
        return Ok(response(
            "needs_selection",
            None,
            if matches.is_empty() {
                "No dossier uniquely matched the supplied topic."
            } else {
                "More than one dossier matched the supplied topic."
            },
            &candidates,
        ));
    }

    let query_terms = lexical_terms(query);
    let matches: Vec<_> = candidates
        .iter()
        .filter(|candidate| clear_query_title_match(candidate, &query_terms))
        .cloned()
        .collect();
    if matches.len() == 1 {
        return Ok(response(
            "selected",
            matches.first(),
            "Selected the unique clear title match from the query.",
            &matches,
        ));
    }
    Ok(response(
        "needs_selection",
        None,
        if matches.len() > 1 {
            "More than one dossier clearly matched the query title."
        } else {
            "A dossier selection is required."
        },
        &candidates,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn root() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "dossier-selection-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn standard(vault: &std::path::Path, relative: &str, title: &str) {
        let dossier = vault.join(relative);
        fs::create_dir_all(dossier.join("wiki")).unwrap();
        fs::create_dir_all(dossier.join("raw")).unwrap();
        let note = dossier.join(format!(
            "{}.md",
            dossier.file_name().unwrap().to_string_lossy()
        ));
        fs::write(note, format!("# {title}\n")).unwrap();
    }

    #[test]
    fn explicit_eligible_dossier_wins_over_context_and_topic() {
        let root = root();
        let vault = root.join("vault");
        standard(&vault, "30-topics/fictieve-garantie", "Fictieve garantie");
        standard(&vault, "30-topics/andere-zaak", "Andere zaak");

        let selected = select_dossier(
            &vault,
            Some("30-topics/andere-zaak"),
            Some("30-topics/fictieve-garantie/wiki/note.md"),
            Some("fictieve garantie"),
            "fictieve garantie",
        )
        .unwrap();

        assert_eq!(selected["schema"], "viggobsidian.dossier-selection.v1");
        assert_eq!(selected["status"], "selected");
        assert_eq!(selected["dossier"], "30-topics/andere-zaak");
        assert_eq!(selected["candidates"][0]["path"], "30-topics/andere-zaak");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn explicit_plain_folder_is_not_applicable_without_fallback() {
        let root = root();
        let vault = root.join("vault");
        fs::create_dir_all(vault.join("30-topics/plain-folder")).unwrap();
        standard(&vault, "30-topics/real-dossier", "Real dossier");

        let result = select_dossier(
            &vault,
            Some("30-topics/plain-folder"),
            None,
            Some("real dossier"),
            "real dossier",
        )
        .unwrap();

        assert_eq!(result["status"], "not_applicable");
        assert!(result["candidates"].as_array().unwrap().is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn context_uses_the_nearest_real_dossier_scope() {
        let root = root();
        let vault = root.join("vault");
        standard(&vault, "30-topics/parent", "Parent");
        standard(&vault, "30-topics/parent/wiki/nested", "Nested");
        fs::write(
            vault.join("30-topics/parent/wiki/nested/raw/source.md"),
            "# Fictieve bron\n",
        )
        .unwrap();

        let result = select_dossier(
            &vault,
            None,
            Some("30-topics/parent/wiki/nested/raw/source.md"),
            None,
            "irrelevant",
        )
        .unwrap();

        assert_eq!(result["status"], "selected");
        assert_eq!(result["dossier"], "30-topics/parent/wiki/nested");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn direct_wiki_foldernote_with_raw_is_eligible_but_plain_folder_is_not() {
        let root = root();
        let vault = root.join("vault");
        let direct = vault.join("30-topics/direct-wiki");
        fs::create_dir_all(direct.join("raw")).unwrap();
        fs::write(
            direct.join("direct-wiki.md"),
            "---\nlayer: wiki\ncssclasses: [vault-wiki, vault-wiki-direct]\n---\n# Direct Wiki\n",
        )
        .unwrap();
        fs::create_dir_all(vault.join("30-topics/plain/raw")).unwrap();

        let direct_result =
            select_dossier(&vault, Some("30-topics/direct-wiki"), None, None, "").unwrap();
        let plain_result = select_dossier(&vault, Some("30-topics/plain"), None, None, "").unwrap();

        assert_eq!(direct_result["status"], "selected");
        assert_eq!(plain_result["status"], "not_applicable");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn topic_selects_only_a_unique_lexical_candidate_and_keeps_sorted_choices() {
        let root = root();
        let vault = root.join("vault");
        standard(&vault, "30-topics/alpha-garantie", "Alpha garantie");
        standard(&vault, "30-topics/beta-garantie", "Beta garantie");
        standard(&vault, "30-topics/onderhoud", "Onderhoudsoverzicht");

        let unique = select_dossier(&vault, None, None, Some("onderhoud"), "").unwrap();
        let ambiguous = select_dossier(&vault, None, None, Some("garantie"), "").unwrap();

        assert_eq!(unique["status"], "selected");
        assert_eq!(unique["dossier"], "30-topics/onderhoud");
        assert_eq!(ambiguous["status"], "needs_selection");
        assert_eq!(
            ambiguous["candidates"]
                .as_array()
                .unwrap()
                .iter()
                .map(|candidate| candidate["path"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec![
                "30-topics/alpha-garantie",
                "30-topics/beta-garantie",
                "30-topics/onderhoud"
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unrelated_files_do_not_consume_the_directory_discovery_budget() {
        let root = root();
        let vault = root.join("vault");
        fs::create_dir_all(vault.join("30-topics")).unwrap();
        for index in 0..=MAX_DISCOVERY_ENTRIES {
            fs::write(
                vault
                    .join("30-topics")
                    .join(format!("unrelated-{index:04}.txt")),
                "fictieve bijlage",
            )
            .unwrap();
        }
        standard(&vault, "30-topics/fictieve-garantie", "Fictieve garantie");

        let result = select_dossier(&vault, None, None, Some("fictieve garantie"), "").unwrap();
        assert_eq!(result["status"], "selected");
        assert_eq!(result["dossier"], "30-topics/fictieve-garantie");
        assert_eq!(result["discovery_complete"], true);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn incomplete_discovery_returns_structured_selection_instead_of_selecting_partial_results() {
        let root = root();
        let vault = root.join("vault");
        for index in 0..=MAX_DISCOVERY_ENTRIES {
            fs::create_dir_all(vault.join(format!("30-topics/unrelated-{index:04}"))).unwrap();
        }

        let result = select_dossier(&vault, None, None, Some("fictieve garantie"), "").unwrap();
        assert_eq!(result["status"], "needs_selection");
        assert_eq!(result["discovery_complete"], false);
        assert!(
            result["reason"]
                .as_str()
                .unwrap()
                .contains("--dossier or --context-path")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn explicit_and_context_selection_bypass_incomplete_discovery() {
        let root = root();
        let vault = root.join("vault");
        standard(&vault, "30-topics/fictieve-garantie", "Fictieve garantie");
        fs::write(
            vault.join("30-topics/fictieve-garantie/wiki/context.md"),
            "# Fictieve context\n",
        )
        .unwrap();
        for index in 0..=MAX_DISCOVERY_ENTRIES {
            fs::create_dir_all(vault.join(format!("30-topics/unrelated-{index:04}"))).unwrap();
        }

        let explicit =
            select_dossier(&vault, Some("30-topics/fictieve-garantie"), None, None, "").unwrap();
        let context = select_dossier(
            &vault,
            None,
            Some("30-topics/fictieve-garantie/wiki/context.md"),
            None,
            "",
        )
        .unwrap();
        assert_eq!(explicit["dossier"], "30-topics/fictieve-garantie");
        assert_eq!(context["dossier"], "30-topics/fictieve-garantie");
        assert_eq!(explicit["discovery_complete"], true);
        assert_eq!(context["discovery_complete"], true);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn query_can_select_a_clear_unique_title_but_not_a_tied_match() {
        let root = root();
        let vault = root.join("vault");
        standard(&vault, "30-topics/alpha", "Fictieve garantie");
        standard(&vault, "30-topics/beta", "Andere garantie");

        let unique = select_dossier(
            &vault,
            None,
            None,
            None,
            "Wat is bekend over de fictieve garantie?",
        )
        .unwrap();
        let ambiguous = select_dossier(&vault, None, None, None, "garantie").unwrap();

        assert_eq!(unique["status"], "selected");
        assert_eq!(unique["dossier"], "30-topics/alpha");
        assert_eq!(ambiguous["status"], "needs_selection");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn discovery_skips_symlinks_and_hidden_roots() {
        let root = root();
        let vault = root.join("vault");
        standard(&vault, "30-topics/visible", "Visible");
        standard(&vault, ".hidden/hidden", "Hidden");
        std::os::unix::fs::symlink(
            vault.join("30-topics/visible"),
            vault.join("30-topics/linked"),
        )
        .unwrap();

        let result = select_dossier(&vault, None, None, None, "").unwrap();
        assert_eq!(result["status"], "needs_selection");
        assert_eq!(result["candidates"].as_array().unwrap().len(), 1);
        assert_eq!(result["candidates"][0]["path"], "30-topics/visible");
        fs::remove_dir_all(root).unwrap();
    }
}
