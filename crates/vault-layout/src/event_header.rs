use crate::{finder_targets, iso_date, strings, text};
use serde_json::Value;
use std::path::Path;

pub(crate) fn check(
    vault: &Path,
    package: &Path,
    data: &Value,
    body: &str,
) -> Result<(), &'static str> {
    let raw = package.parent().ok_or("event needs a Raw parent")?;
    let dossier = raw.parent().ok_or("event needs a dossier")?;
    let raw_relative = raw
        .strip_prefix(vault)
        .map_err(|_| "Raw must be in the vault")?;
    let classes = &data["cssclasses"];
    if ["vault-navigation", "vault-dashboard", "vault-event"]
        .iter()
        .any(|class| !strings(classes).any(|value| value == *class))
    {
        return Err("use the vault-navigation, vault-dashboard and vault-event classes");
    }
    let lines: Vec<_> = body.lines().map(str::trim_end).collect();
    let mut blocks = lines
        .split(|line| line.is_empty())
        .filter(|block| !block.is_empty());
    if !matches!(blocks.next(), Some([title]) if title.starts_with("# ") && title.len() > 2) {
        return Err("start the event with its title");
    }
    let expected_back = format!("> - [[{}/raw|Back]]", raw_relative.display());
    let Some(["> [!navigation|shortcuts]", back, finder]) = blocks.next() else {
        return Err("use one shortcut row containing Back and Finder below the title");
    };
    if *back != expected_back {
        return Err("put Back to this dossier's Raw index below the title");
    }
    if !finder_targets(finder, package) {
        return Err("point Finder to this event's package folder");
    }
    if !matches!(blocks.next(), Some(["> [!wiki-facts|dossier]", line]) if breadcrumb(vault, dossier, line))
    {
        return Err("show an ordered, clickable Dossier path ending at the dossier foldernote");
    }
    let task = text(data, "type") == Some("task");
    if task {
        if text(data, "status").is_none() {
            return Err("task cards require a status");
        }
        if data
            .get("work_date")
            .is_some_and(|date| !date.is_null() && !date.as_str().is_some_and(iso_date))
        {
            return Err("work_date must be a valid YYYY-MM-DD date when present");
        }
    }
    if strings(classes).any(|class| class == "vault-event-v1-1") {
        if blocks.next() != Some(&["```vault-event-detail", "```"]) {
            return Err("use the Event Note v1.1 cards below the dossier path");
        }
        if !matches!(blocks.next(), Some(["> [!event-context] Context", content @ ..])
            if content.iter().all(|line| line.starts_with('>'))
                && content.iter().any(|line| line.strip_prefix("> ").is_some_and(|text| !text.trim().is_empty())))
        {
            return Err("use a non-collapsible Context bubble below the Event Note v1.1 cards");
        }
        if blocks.next() != Some(&["## Activity"]) {
            return Err("put Activity immediately after the Event Note v1.1 header");
        }
        return crate::activity_layout::check_v11(body);
    }
    if task {
        if blocks.next() != Some(&["```vault-task-detail", "```"]) {
            return Err("use the vault-task-detail cards for task status and action date");
        }
    } else {
        let Some(["> [!wiki-facts]", cards @ ..]) = blocks.next() else {
            return Err("use source fact cards below the dossier path");
        };
        let name = package
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let mut status = false;
        let mut date = false;
        for line in cards {
            let Some((label, value)) = line
                .strip_prefix("> - ")
                .and_then(|line| line.strip_suffix("**"))
                .and_then(|line| line.split_once(" **"))
            else {
                return Err("format source facts as Label **Value**");
            };
            let expected_status = match text(data, "wiki_status") {
                Some("processed") => Some("Verwerkt"),
                Some("pending") => Some("Onverwerkt"),
                _ => None,
            };
            match label {
                "Wiki-status" if !status && expected_status == Some(value) => status = true,
                "Verwerkt op"
                    if !status
                        && expected_status == Some("Verwerkt")
                        && iso_date(value)
                        && text(data, "wiki_processed_on") == Some(value) =>
                {
                    status = true
                }
                "Gebeurtenis"
                    if !date && iso_date(value) && text(data, "event_date") == Some(value) =>
                {
                    date = true
                }
                "Vastgelegd"
                    if !date && iso_date(value) && text(data, "created") == Some(value) =>
                {
                    date = true
                }
                "Pakketdatum"
                    if !date
                        && text(data, "event_date").is_none()
                        && text(data, "created").is_none()
                        && iso_date(value)
                        && value
                            .bytes()
                            .filter(|byte| *byte != b'-')
                            .eq(name.bytes().take(8)) =>
                {
                    date = true
                }
                _ => return Err("source fact cards must match the note metadata and package date"),
            }
        }
        if !status || !date {
            return Err("show the source's Wiki status or processed date, and its source date");
        }
    }

    if blocks
        .flatten()
        .copied()
        .take_while(|line| *line != "## Activity")
        .any(|line| {
            matches!(
                line,
                "> [!navigation|shortcuts]"
                    | "> [!wiki-facts|dossier]"
                    | "> [!wiki-facts]"
                    | "```vault-task-detail"
            )
        })
    {
        return Err("include only one header and one set of cards before Activity");
    }
    Ok(())
}

fn breadcrumb(vault: &Path, dossier: &Path, line: &str) -> bool {
    let Some(path) = line
        .strip_prefix("> - Dossier **")
        .and_then(|line| line.strip_suffix("**"))
    else {
        return false;
    };
    path.rsplit(" / ")
        .zip(dossier.ancestors())
        .all(|(link, folder)| {
            let Some((target, label)) = link
                .strip_prefix("[[")
                .and_then(|link| link.strip_suffix("]]"))
                .and_then(|link| link.split_once('|'))
            else {
                return false;
            };
            folder.strip_prefix(vault).is_ok_and(|relative| {
                !relative.as_os_str().is_empty()
                    && folder.file_name().is_some_and(|name| {
                        target == relative.join(name).to_string_lossy()
                            && !label.trim().is_empty()
                            && !label.contains(['[', ']', '|'])
                    })
            })
        })
        && path.split(" / ").count()
            <= dossier
                .ancestors()
                .take_while(|folder| *folder != vault)
                .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const HEADER: &str = "# Event\n\n> [!navigation|shortcuts]\n> - [[topics/car/raw/raw|Back]]\n> - [Finder](file:///vault/topics/car/raw/20261108-source)\n\n> [!wiki-facts|dossier]\n> - Dossier **[[topics/topics|Topics]] / [[topics/car/car|Car]]**\n\n";
    const SOURCE_CARDS: &str =
        "> [!wiki-facts]\n> - Wiki-status **Verwerkt**\n> - Gebeurtenis **2026-11-08**\n";
    const TASK_CARDS: &str = "```vault-task-detail\n```\n";
    const EVENT_CARDS: &str = "```vault-event-detail\n```\n";

    fn validate(data: &Value, body: &str) -> Result<(), &'static str> {
        check(
            Path::new("/vault"),
            Path::new("/vault/topics/car/raw/20261108-source"),
            data,
            body,
        )
    }

    #[test]
    fn enforces_order_and_exact_header_destinations() {
        let data = json!({
            "type": "event", "wiki_status": "processed", "event_date": "2026-11-08",
            "cssclasses": ["vault-navigation", "vault-dashboard", "vault-event"]
        });
        let valid = format!("{HEADER}{SOURCE_CARDS}\n## Activity\n");
        assert_eq!(validate(&data, &valid), Ok(()));
        for invalid in [
            valid.replace("> - [[topics/car/raw/raw|Back]]\n", ""),
            valid.replace("car/raw/raw|Back", "car/car|Back"),
            valid.replace(
                "file:///vault/topics/car/raw/20261108-source",
                "file:///vault/topics/car/raw",
            ),
            valid.replace("[[topics/car/car|Car]]", "[[topics/other/other|Other]]"),
            valid.replace("\n\n> [!wiki-facts|dossier]", "\n> [!wiki-facts|dossier]"),
            valid.replace("2026-11-08**", "2026-11-09**"),
            valid.replace("**Verwerkt**", "**Onverwerkt**"),
            valid.replace(SOURCE_CARDS, TASK_CARDS),
            format!("{HEADER}{SOURCE_CARDS}\n{SOURCE_CARDS}\n## Activity\n"),
            format!("# Event\n\n## Activity\n\n{HEADER}{SOURCE_CARDS}"),
        ] {
            assert!(validate(&data, &invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn tasks_use_dynamic_cards_without_inventing_an_action_date() {
        let mut task = json!({
            "type": "task", "status": "🟢 done",
            "cssclasses": ["vault-navigation", "vault-dashboard", "vault-event"]
        });
        let body = format!("{HEADER}{TASK_CARDS}\n## Activity\n");
        assert_eq!(validate(&task, &body), Ok(()));
        task["work_date"] = json!("2026-11-09");
        assert_eq!(validate(&task, &body), Ok(()));
        assert!(validate(&task, &body.replace(TASK_CARDS, SOURCE_CARDS)).is_err());
        task["work_date"] = json!("2026-11-31");
        assert!(validate(&task, &body).is_err());
    }

    #[test]
    fn event_v11_requires_shared_cards_and_fixed_context_above_activity() {
        let mut data = json!({
            "type": "task", "status": "🟢 done", "wiki_status": "pending",
            "cssclasses": ["vault-navigation", "vault-dashboard", "vault-event", "vault-event-v1-1"]
        });
        let context = "> [!event-context] Context\n> **Afhankelijk van**\n>\n> [[Andere taak]]";
        let task = format!(
            "{HEADER}{EVENT_CARDS}\n{context}\n\n## Activity\n\n> [!activity-day|2026-11-11] 11 november 2026\n\n> [!activity-message|codex] Codex\n> Afgerond.\n>\n> *Codex*\n"
        );
        let without_context = format!("{HEADER}{EVENT_CARDS}\n## Activity\n");
        assert_eq!(validate(&data, &task), Ok(()));
        assert!(validate(&data, &without_context).is_err());
        assert!(
            validate(
                &data,
                &task.replace("[!event-context]", "[!event-context]-")
            )
            .is_err()
        );
        assert!(validate(&data, &task.replace(EVENT_CARDS, TASK_CARDS)).is_err());
        assert!(
            validate(
                &data,
                &task.replace(context, "## Afhankelijk van\n\n[[Andere taak]]")
            )
            .is_err()
        );
        data["type"] = json!("note");
        assert!(validate(&data, &without_context).is_err());
        assert_eq!(validate(&data, &task), Ok(()));
    }
}
