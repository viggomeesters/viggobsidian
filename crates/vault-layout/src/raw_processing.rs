use crate::{
    activity_layout::message_heading,
    index_navigation::{visible_body, wiki_links},
    shortcut_links::{anchor_count, decode_percent, heading},
};

const TITLE: &str = "Verwerking in Wiki";

pub(crate) fn check_event(
    body: &str,
    note: &str,
    anchor: &str,
    wiki: &str,
) -> Result<(), &'static str> {
    let visible = visible_body(body);
    let (_, activity) = visible
        .split_once("\n## Activity\n")
        .ok_or("processed Raw needs an Activity section")?;
    let activity = activity.split("\n## ").next().unwrap_or(activity);
    let internal = activity
        .match_indices("> [!activity-message|")
        .find_map(|(offset, _)| {
            if offset > 0 && !activity[..offset].ends_with('\n') {
                return None;
            }
            let block = activity[offset..].split("\n\n").next()?;
            block
                .lines()
                .any(|line| line == "> **Verwerking in Wiki**")
                .then_some((&activity[..offset], block))
        });
    let (before, processing) = internal
        .or_else(|| {
            activity
                .split_once("\n### Verwerking in Wiki\n")
                .or_else(|| {
                    let (before, section) = activity.split_once("\n### Verwerking\n")?;
                    let section = section.split("\n### ").next()?;
                    let (_, processing) = section.split_once("\n#### Verwerking in Wiki\n")?;
                    Some((before, processing))
                })
        })
        .ok_or("Activity needs a Verwerking in Wiki subsection")?;
    let date = before.lines().rev().find_map(|line| {
        let (date, label) = line.strip_prefix("> [!activity-day|")?.split_once("] ")?;
        (!label.trim().is_empty()).then_some(date)
    });
    if !date.is_some_and(super::iso_date) {
        return Err("Wiki processing needs a valid Activity day separator");
    }
    let mut lines = processing.trim_start_matches('\n').lines();
    let heading = lines.next().ok_or("Wiki processing needs a named bubble")?;
    let (role, actor) = message_heading(heading)
        .ok_or("Wiki processing needs a bubble with a role and named author")?;
    let message: Vec<_> = lines.take_while(|line| line.starts_with('>')).collect();
    match crate::activity_delivery::header_stamp(heading, actor) {
        Some(false) => return Err("Wiki processing has invalid header provenance"),
        Some(true) => {}
        None => {
            let footer = message
                .last()
                .and_then(|line| line.strip_prefix("> *"))
                .and_then(|line| line.strip_suffix('*'))
                .ok_or("Wiki processing needs an author in the bubble footer")?;
            let valid_footer = crate::activity_delivery::valid_stamp(footer, actor)
                || footer == actor
                || footer.split_once(" · ").is_some_and(|(time, name)| {
                    name == actor
                        && time.split_once(':').is_some_and(|(hour, minute)| {
                            hour.len() == 2
                                && minute.len() == 2
                                && hour
                                    .bytes()
                                    .chain(minute.bytes())
                                    .all(|byte| byte.is_ascii_digit())
                                && hour.parse::<u8>().is_ok_and(|value| value < 24)
                                && minute.parse::<u8>().is_ok_and(|value| value < 60)
                        })
                });
            if !valid_footer {
                return Err(
                    "use HH:MM · author · channel (— for unknown time), or a supported legacy author footer",
                );
            }
        }
    }
    check(
        &format!(
            "### {TITLE}\n\n> [!activity-message|{role}] {actor}\n{}",
            message.join("\n")
        ),
        note,
        anchor,
        wiki,
    )
}

pub(crate) fn target(link: &str) -> Option<(&str, &str)> {
    let inner = link.strip_prefix("[[")?.strip_suffix("]]")?;
    let (note, anchor) = inner
        .split('|')
        .next()?
        .trim_end_matches('\\')
        .split_once('#')?;
    let anchor = anchor.trim();
    (!note.is_empty() && !anchor.is_empty() && !anchor.starts_with('^')).then_some((note, anchor))
}

fn section(body: &str) -> Option<String> {
    let visible = visible_body(body);
    let mut level = None;
    let mut previous = None;
    let mut result = String::new();
    let mut activity_message = false;
    for line in visible.lines() {
        let trimmed = line.trim_start_matches(' ');
        let head = (line.len() - trimmed.len() <= 3)
            .then(|| heading(trimmed, previous))
            .flatten();
        if let Some((depth, title)) = head {
            if level.is_some_and(|level| depth <= level) {
                break;
            }
            if level.is_none()
                && (title == TITLE
                    || title.strip_prefix(TITLE).is_some_and(|suffix| {
                        suffix.starts_with(" · ") || suffix.starts_with(" — ")
                    }))
            {
                level = Some(depth);
            }
            previous = None;
            continue;
        }
        previous = (!trimmed.is_empty() && !trimmed.starts_with('>')).then_some(trimmed);
        if level.is_some() && line.len() - trimmed.len() <= 3 {
            if trimmed.starts_with("> [!activity-message|") {
                activity_message = true;
            } else if let Some(content) = trimmed.strip_prefix('>') {
                let content = content.trim_start();
                if content.starts_with("[!") {
                    activity_message = false;
                }
                if activity_message && !content.starts_with(['>', '!', '*']) {
                    result.push_str(content);
                    result.push('\n');
                }
            } else {
                activity_message = false;
                result.push_str(line);
                result.push('\n');
            }
        }
    }
    level.map(|_| result)
}

pub(crate) fn check(body: &str, note: &str, anchor: &str, wiki: &str) -> Result<(), &'static str> {
    if anchor_count(wiki, anchor) != 1 {
        return Err("wiki_target must refer to an existing, unambiguous Wiki heading");
    }
    let section = section(body).ok_or("add a Verwerking in Wiki section to the Raw note")?;
    let note = note.strip_suffix(".md").unwrap_or(note);
    let decoded = decode_percent(anchor);
    if !wiki_links(&section).any(|link| {
        let markup = format!("[[{link}]]");
        if !section
            .match_indices(&markup)
            .any(|(offset, _)| !section[..offset].ends_with('!'))
        {
            return false;
        }
        let Some((path, heading)) = link
            .split('|')
            .next()
            .and_then(|link| link.trim_end_matches('\\').split_once('#'))
        else {
            return false;
        };
        path.strip_suffix(".md").unwrap_or(path) == note
            && decode_percent(heading).trim() == decoded.trim()
    }) {
        return Err("link from Verwerking in Wiki to the same Wiki heading as wiki_target");
    }
    let mut rest = section.as_str();
    while let Some((before, tail)) = rest.split_once("[[") {
        if before.chars().any(char::is_alphabetic) {
            return Ok(());
        }
        let Some((_, after)) = tail.split_once("]]") else {
            return Err("describe what was carried into Wiki or why the source added nothing");
        };
        rest = after;
    }
    if rest.chars().any(char::is_alphabetic) {
        Ok(())
    } else {
        Err("describe what was carried into Wiki or why the source added nothing")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn processed_event_accepts_validated_header_without_requiring_duplicate_footer() {
        let note = "dossier/wiki/wiki";
        let wiki = "## Bronnen\n";
        let body = "# Bron\n\n## Activity\n\n> [!activity-day|2026-11-12] 12 november\n\n> [!activity-message|codex] Codex <span class=\"activity-meta\">— · Codex-chat</span>\n> **Verwerking in Wiki**\n> Broninhoud verwerkt in [[dossier/wiki/wiki#Bronnen]].\n";
        assert_eq!(check_event(body, note, "Bronnen", wiki), Ok(()));
        let footer = body.replace(" <span class=\"activity-meta\">— · Codex-chat</span>", "")
            + ">\n> *— · Codex · Codex-chat*\n";
        assert_eq!(check_event(&footer, note, "Bronnen", wiki), Ok(()));
        assert_eq!(
            check_event(&footer.replace("*— ·", "*09:42 ·"), note, "Bronnen", wiki),
            Ok(())
        );
        assert!(
            check_event(
                &footer.replace("· Codex ·", "· Other ·"),
                note,
                "Bronnen",
                wiki
            )
            .is_err()
        );
        assert_eq!(
            check_event(&body.replace("— ·", "23:59 ·"), note, "Bronnen", wiki),
            Ok(())
        );
        for invalid in [
            body.replace("— ·", "25:99 ·"),
            body.replace("Codex-chat</span>", "</span>"),
            body.replace("Codex-chat</span>", "<script>bad</script></span>"),
            body.replace("Codex <span", "Codex<span"),
            body.replace("Codex-chat</span>", "Codex-chat</span><span>extra</span>"),
        ] {
            assert!(
                check_event(&invalid, note, "Bronnen", wiki).is_err(),
                "{invalid}"
            );
        }
        assert!(
            check_event(
                &body.replace("2026-11-12", "2026-13-12"),
                note,
                "Bronnen",
                wiki
            )
            .is_err()
        );
        assert!(check_event(&body.replace("#Bronnen", "#Other"), note, "Bronnen", wiki).is_err());
    }
    use super::*;

    #[test]
    fn processing_label_lives_in_the_bubble_without_becoming_evidence() {
        let body = "# Source\n\n## Activity\n\n> [!activity-day|2026-11-11] 11 november\n\n> [!activity-message|codex] Codex\n> **Verwerking in Wiki**\n> Contract gelezen en verwerkt in [[dossier/wiki/wiki#Contract|Contract]].\n>\n> *Codex*\n";
        assert_eq!(
            check_event(body, "dossier/wiki/wiki", "Contract", "## Contract\n"),
            Ok(())
        );
        for invalid in [
            body.replace("> **Verwerking in Wiki**", "> > **Verwerking in Wiki**"),
            body.replace("Contract gelezen en verwerkt in ", ""),
            body.replace("wiki#Contract|", "wiki#Ontbreekt|"),
            body.replace("*Codex*", "*99:99 · Codex*"),
        ] {
            assert!(
                check_event(&invalid, "dossier/wiki/wiki", "Contract", "## Contract\n").is_err()
            );
        }
    }

    #[test]
    fn checks_visible_section_prose_and_exact_heading() {
        let note = "dossier/wiki/wiki";
        let wiki = "# Wiki\n\n## Actuele stand\n\n## Historie\n";
        let valid = "## Verwerking in Wiki · 8 november 2026\n\nDe looptijd staat in [[dossier/wiki/wiki.md#Actuele%20stand|de Wiki]].";
        assert_eq!(check(valid, note, "Actuele stand", wiki), Ok(()));
        assert!(check(valid, note, "Historie", wiki).is_err());
        assert!(check(&valid.replace("[[", "![["), note, "Actuele stand", wiki).is_err());
        assert!(check(valid, note, "Ontbreekt", wiki).is_err());
        assert!(check(&format!("```md\n{valid}\n```"), note, "Actuele stand", wiki).is_err());
        assert!(
            check(
                "## Verwerking in Wiki\n\n[[dossier/wiki/wiki#Actuele stand|Verwerkt]]",
                note,
                "Actuele stand",
                wiki
            )
            .is_err()
        );
        assert!(check("## Verwerking in Wiki\n\nSamenvatting.\n\n## Overig\n\n[[dossier/wiki/wiki#Actuele stand]]", note, "Actuele stand", wiki).is_err());
        assert!(
            check(
                valid,
                note,
                "Actuele stand",
                &format!("{wiki}\n## Actuele stand")
            )
            .is_err()
        );
        assert!(target("[[dossier/wiki/wiki]]").is_none());
        assert!(target("[[dossier/wiki/wiki#^bewijs]]").is_none());
    }

    #[test]
    fn strict_processing_requires_day_actor_time_and_link_inside_bubble() {
        let note = "dossier/wiki/wiki";
        let wiki = "## Bronnen\n";
        let valid = "# Bron\n\n## Activity\n\n> [!activity-day|2026-11-08] 8 november\n\n### Verwerking in Wiki\n\n> [!activity-message|codex] Codex\n> Bron gecontroleerd. [[dossier/wiki/wiki#Bronnen]]\n>\n> *18:23 · Codex*";
        assert_eq!(check_event(valid, note, "Bronnen", wiki), Ok(()));
        assert_eq!(
            check_event(
                &valid.replace(
                    "### Verwerking in Wiki",
                    "### Verwerking\n\n#### Verwerking in Wiki"
                ),
                note,
                "Bronnen",
                wiki
            ),
            Ok(())
        );
        assert_eq!(
            check_event(&valid.replace("18:23 · ", ""), note, "Bronnen", wiki),
            Ok(())
        );
        for invalid in [
            valid.replace("> [!activity-day|2026-11-08] 8 november\n", ""),
            valid.replace("2026-11-08", "2026-02-30"),
            valid.replace("18:23", "99:99"),
            valid.replace("18:23 · Codex", "18:23 · Iemand"),
            valid.replace("] Codex", "] "),
            valid.replace("> Bron gecontroleerd.", "Bron gecontroleerd."),
            valid.replace("wiki#Bronnen", "wiki#Overig"),
            "## Verwerking in Wiki\n\nBron gecontroleerd. [[dossier/wiki/wiki#Bronnen]]".into(),
        ] {
            assert!(
                check_event(&invalid, note, "Bronnen", wiki).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn reads_processing_from_activity_message_without_counting_its_footer() {
        let note = "dossier/wiki/wiki";
        let wiki = "## Bronnen\n";
        let message = "## Activity\n\n> [!activity-day|2026-11-08] 8 november\n\n### Verwerking in Wiki\n\n> [!activity-message|codex] Codex\n> De index ontsluit de originele bronnen.\n>\n> [[dossier/wiki/wiki#Bronnen|Open Wiki]]\n>\n> *18:23 · Codex*";
        assert_eq!(check(message, note, "Bronnen", wiki), Ok(()));
        assert!(
            check(
                &message.replace("> De index ontsluit de originele bronnen.\n", ""),
                note,
                "Bronnen",
                wiki
            )
            .is_err()
        );
        assert!(
            check(
                &message.replace("activity-message|codex", "quote|codex"),
                note,
                "Bronnen",
                wiki
            )
            .is_err()
        );
    }
}
