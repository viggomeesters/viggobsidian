use dossier_retrieval_core::{
    ask_cli::{ASK_HELP, run_ask},
    build_index, query_index,
};
use std::path::Path;

fn argument<'a>(args: &'a [String], name: &str) -> Result<&'a str, String> {
    let position = args
        .iter()
        .position(|arg| arg == name)
        .ok_or_else(|| format!("missing {name}"))?;
    args.get(position + 1)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value for {name}"))
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("build") => {
            let output = build_index(
                Path::new(argument(&args, "--vault")?),
                argument(&args, "--dossier")?,
                Path::new(argument(&args, "--index")?),
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?
            );
        }
        Some("query") => {
            let limit = args
                .iter()
                .position(|arg| arg == "--limit")
                .map(|position| {
                    args.get(position + 1)
                        .ok_or("missing value for --limit")?
                        .parse::<usize>()
                        .map_err(|_| "--limit must be an integer")
                })
                .transpose()?
                .unwrap_or(8);
            let output = query_index(
                Path::new(argument(&args, "--vault")?),
                argument(&args, "--dossier")?,
                Path::new(argument(&args, "--index")?),
                argument(&args, "--query")?,
                limit,
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?
            );
        }
        Some("ask") => {
            let output = run_ask(&args[1..])?;
            println!(
                "{}",
                serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?
            );
        }
        Some("help") | Some("--help") | None => println!(
            "dossier-retrieve\nbuild --vault DIR --dossier RELATIVE/PATH --index /absolute/external/index.json\nquery --vault DIR --dossier RELATIVE/PATH --index /absolute/external/index.json --query TEXT [--limit 1..20]\n{ASK_HELP}\n\nThe command is read-only for --vault. Build writes only the disposable external index; query revalidates every indexed source hash before returning ranked excerpts."
        ),
        _ => return Err("unknown command; use --help".into()),
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}
