//! Shared strict CLI parsing for the managed one-command dossier ask route.
//!
//! Both native entrypoints call [`run_ask`] so option spelling and rejection
//! behavior cannot drift between `dossier-retrieve ask` and `viggobsidian
//! dossier-ask`.
use crate::{Result, managed};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const DEFAULT_LIMIT: usize = 8;
const MAX_LIMIT: usize = 20;

struct AskArguments {
    vault: PathBuf,
    dossier: Option<String>,
    context_path: Option<String>,
    topic: Option<String>,
    cache_root: PathBuf,
    query: String,
    limit: usize,
}

fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(message.into())
}

fn default_cache_root() -> Result<PathBuf> {
    if let Some(root) = std::env::var_os("XDG_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        if !root.is_absolute() {
            return fail("XDG_CACHE_HOME must be an absolute path");
        }
        return Ok(root.join("viggobsidian/dossier-retrieval"));
    }
    if let Some(home) = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        if !home.is_absolute() {
            return fail("HOME must be an absolute path when XDG_CACHE_HOME is unset");
        }
        return Ok(home.join(".cache/viggobsidian/dossier-retrieval"));
    }
    fail("cannot determine cache root: set absolute XDG_CACHE_HOME or HOME, or pass --cache-dir")
}

fn next_value<'a>(args: &'a [String], index: &mut usize, flag: &str) -> Result<&'a str> {
    *index += 1;
    let value = args
        .get(*index)
        .map(String::as_str)
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| format!("missing value for {flag}"))?;
    Ok(value)
}

fn parse_limit(value: &str) -> Result<usize> {
    let limit = value
        .parse::<usize>()
        .map_err(|_| "--limit must be an integer from 1 through 20".to_owned())?;
    if !(1..=MAX_LIMIT).contains(&limit) {
        return fail("--limit must be from 1 through 20");
    }
    Ok(limit)
}

fn parse(args: &[String]) -> Result<AskArguments> {
    let mut seen = BTreeSet::new();
    let mut vault = None;
    let mut dossier = None;
    let mut context_path = None;
    let mut topic = None;
    let mut cache_root = None;
    let mut query = None;
    let mut limit = None;
    let mut index = 0;

    while index < args.len() {
        let flag = args[index].as_str();
        if !flag.starts_with("--") {
            return fail(format!("unexpected positional argument: {flag}"));
        }
        if !seen.insert(flag.to_owned()) {
            return fail(format!("duplicate option: {flag}"));
        }
        match flag {
            "--vault" => vault = Some(PathBuf::from(next_value(args, &mut index, flag)?)),
            "--dossier" => dossier = Some(next_value(args, &mut index, flag)?.to_owned()),
            "--context-path" => context_path = Some(next_value(args, &mut index, flag)?.to_owned()),
            "--topic" => topic = Some(next_value(args, &mut index, flag)?.to_owned()),
            "--cache-dir" => {
                let value = PathBuf::from(next_value(args, &mut index, flag)?);
                if !value.is_absolute() {
                    return fail("--cache-dir must be an absolute path");
                }
                cache_root = Some(value);
            }
            "--query" => {
                let value = next_value(args, &mut index, flag)?;
                if value.trim().is_empty() {
                    return fail("--query must not be empty");
                }
                query = Some(value.to_owned());
            }
            "--limit" => limit = Some(parse_limit(next_value(args, &mut index, flag)?)?),
            _ => return fail(format!("unknown option: {flag}")),
        }
        index += 1;
    }

    Ok(AskArguments {
        vault: vault.ok_or("missing --vault")?,
        dossier,
        context_path,
        topic,
        cache_root: match cache_root {
            Some(root) => root,
            None => default_cache_root()?,
        },
        query: query.ok_or("missing --query")?,
        limit: limit.unwrap_or(DEFAULT_LIMIT),
    })
}

/// Run a managed read-only ask from arguments after the command name.
///
/// Accepted options are exactly `--vault`, `--query`, `--dossier`,
/// `--context-path`, `--topic`, `--cache-dir`, and `--limit`; each is accepted
/// at most once. The selected vault is always explicit.
pub fn run_ask(args: &[String]) -> Result<Value> {
    let args = parse(args)?;
    managed::ask_dossier(
        Path::new(&args.vault),
        args.dossier.as_deref(),
        args.context_path.as_deref(),
        args.topic.as_deref(),
        Path::new(&args.cache_root),
        &args.query,
        args.limit,
    )
}

/// Help text used by both native entrypoints.
pub const ASK_HELP: &str = "ask --vault DIR --query TEXT [--dossier RELATIVE/PATH] [--context-path RELATIVE/PATH] [--topic LABEL] [--cache-dir ABS] [--limit 1..20]\n\nSelects one dossier only when the explicit owner/context is unambiguous, then returns read-only cited evidence. --vault is mandatory; the default cache is user-local and never inside the vault or repository.";

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn parser_requires_explicit_vault_and_query() {
        assert!(parse(&args(&["--query", "fictieve vraag"])).is_err());
        assert!(parse(&args(&["--vault", "/fixture/vault"])).is_err());
    }

    #[test]
    fn parser_refuses_duplicate_unknown_and_positional_options() {
        for input in [
            args(&[
                "--vault",
                "/fixture/vault",
                "--vault",
                "/other",
                "--query",
                "vraag",
            ]),
            args(&[
                "--vault",
                "/fixture/vault",
                "--unknown",
                "x",
                "--query",
                "vraag",
            ]),
            args(&["--vault", "/fixture/vault", "ask", "--query", "vraag"]),
            args(&[
                "--vault",
                "/fixture/vault",
                "--query",
                "vraag",
                "--cache-dir",
                "relative",
            ]),
            args(&[
                "--vault",
                "/fixture/vault",
                "--query",
                "vraag",
                "--limit",
                "21",
            ]),
        ] {
            assert!(parse(&input).is_err(), "input: {input:?}");
        }
    }

    #[test]
    fn parser_preserves_explicit_selection_and_cache_arguments() {
        let parsed = parse(&args(&[
            "--vault",
            "/fixture/vault",
            "--dossier",
            "30-topics/example",
            "--context-path",
            "30-topics/example/wiki/wiki.md",
            "--topic",
            "fictief",
            "--cache-dir",
            "/fixture/cache",
            "--query",
            "Welke fictieve garantie geldt?",
            "--limit",
            "3",
        ]))
        .unwrap();
        assert_eq!(parsed.vault, PathBuf::from("/fixture/vault"));
        assert_eq!(parsed.dossier.as_deref(), Some("30-topics/example"));
        assert_eq!(
            parsed.context_path.as_deref(),
            Some("30-topics/example/wiki/wiki.md")
        );
        assert_eq!(parsed.topic.as_deref(), Some("fictief"));
        assert_eq!(parsed.cache_root, PathBuf::from("/fixture/cache"));
        assert_eq!(parsed.limit, 3);
    }
}
