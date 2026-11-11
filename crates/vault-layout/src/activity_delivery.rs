//! Opt-in delivery chronology, separate from task state and historical text.
use std::collections::BTreeMap;
const DELIVERY: &str = "<!-- viggobsidian:mail-delivery:v1 ";
const INPUT: &str = "<!-- viggobsidian:input:v1 author=human -->";

fn visible_lines(body: &str) -> Vec<&str> {
    let mut fence = None;
    body.lines()
        .filter(|line| {
            let (depth, outer) = unquote(line, usize::MAX);
            let content = if let Some((_, _, opening_depth)) = fence {
                unquote(line, opening_depth).1
            } else {
                outer
            };
            let indent = content.bytes().take_while(|b| *b == b' ').count().min(3);
            let content = &content[indent..];
            let byte = content.as_bytes().first().copied().unwrap_or(0);
            let width = content.bytes().take_while(|b| *b == byte).count();
            if let Some((opening, minimum, _)) = fence {
                if byte == opening && width >= minimum && content[width..].trim().is_empty() {
                    fence = None;
                }
                return false;
            }
            if matches!(byte, b'`' | b'~') && width >= 3 {
                fence = Some((byte, width, depth));
                return false;
            }
            true
        })
        .collect()
}

fn unquote(mut line: &str, limit: usize) -> (usize, &str) {
    let mut depth = 0;
    while depth < limit {
        if let Some(rest) = line.strip_prefix('>') {
            depth += 1;
            line = rest.strip_prefix(' ').unwrap_or(rest);
        } else {
            break;
        }
    }
    (depth, line)
}

pub(crate) fn visible_mail_payload_lines(block: &[&str]) -> std::collections::BTreeSet<usize> {
    let mut payload = std::collections::BTreeSet::new();
    if !visible_mail_delivery(block) {
        return payload;
    }
    let mut active = false;
    let mut fence = 0;
    for (index, line) in block.iter().enumerate() {
        let (_, content) = unquote(line, 1);
        if fence > 0 {
            payload.insert(index);
            let candidate = content.trim_start_matches(' ');
            let width = candidate.bytes().take_while(|b| *b == b'`').count();
            if content.len() - candidate.len() <= 3
                && width >= fence
                && candidate[width..].trim().is_empty()
            {
                fence = 0;
            }
        } else if content.starts_with("<!-- viggobsidian:mail-draft:v1 recipients=") {
            active = true;
        } else if content == "<!-- /viggobsidian:mail-draft -->" {
            active = false;
        } else if active {
            let width = content.bytes().take_while(|b| *b == b'`').count();
            if width >= 3 && content[width..].trim() == "text" {
                fence = width;
                payload.insert(index);
            }
        }
    }
    payload
}

fn identity(line: &str) -> Option<(&str, u32)> {
    let text = line
        .strip_prefix("> ")?
        .strip_prefix(DELIVERY)?
        .strip_suffix(" -->")?;
    let (id, version) = text.strip_prefix("id=")?.split_once(" version=")?;
    if id.is_empty()
        || id.len() > 80
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return None;
    }
    let version = version.parse::<u32>().ok().filter(|v| *v > 0)?;
    Some((id, version))
}

pub(crate) fn visible_mail_delivery(block: &[&str]) -> bool {
    let body = block.join("\n");
    let lines = visible_lines(&body);
    let marker: Vec<_> = lines
        .iter()
        .filter(|l| unquote(l, usize::MAX).1.starts_with(DELIVERY))
        .collect();
    marker.len() == 1
        && identity(marker[0]).is_some()
        && identity(marker[0]).is_some_and(|(_, v)| {
            lines
                .iter()
                .any(|line| *line == format!("> **Mailconcept v{v}**"))
        })
        && lines
            .iter()
            .filter(|l| l.starts_with("> <!-- viggobsidian:mail-draft:v1 recipients="))
            .count()
            == 1
        && lines
            .iter()
            .any(|l| *l == "> **To**" || l.starts_with("> Ontbreekt: "))
        && lines.contains(&"> **Onderwerp**")
        && lines.contains(&"> **Body**")
        && crate::mail_draft::findings(&body).is_empty()
        && lines
            .first()
            .and_then(|l| crate::activity_layout::message_heading(l))
            .is_some_and(|(role, _)| !matches!(role, "human" | "user"))
}

pub(crate) fn findings(body: &str) -> Vec<String> {
    let lines = visible_lines(body);
    let live_addresses = lines
        .iter()
        .map(|line| line.as_ptr() as usize)
        .collect::<std::collections::BTreeSet<_>>();
    let mut result = Vec::new();
    let mut versions = BTreeMap::new();
    let mut current: Vec<&str> = Vec::new();
    let mut role = None;
    for line in &lines {
        if line.is_empty() {
            role = None;
        }
        if let Some((next, _)) = crate::activity_layout::message_heading(line) {
            role = Some(next);
        }
        if unquote(line, usize::MAX).1 == INPUT
            && !(*line == format!("> {INPUT}") && role == Some("human"))
        {
            result.push("literal human input must use its own canonical human author bubble, regardless of the writing tool".into());
        }
    }
    // Check each live bubble with its original fences and copy payload intact.
    for line in body.lines().chain(std::iter::once("")) {
        if line.is_empty() {
            if !current.is_empty() {
                for marker in current.iter().filter(|l| {
                    live_addresses.contains(&(l.as_ptr() as usize))
                        && unquote(l, usize::MAX).1.starts_with(DELIVERY)
                }) {
                    if let Some((id, version)) = identity(marker) {
                        let previous = versions.insert(id.to_owned(), version);
                        if previous.is_some_and(|p| p >= version) {
                            result.push(format!("delivery {id}: append a complete new version with a strictly increasing number; preserve earlier versions"));
                        }
                    } else {
                        result.push(
                            "mail delivery needs a direct, valid stable id and positive version"
                                .into(),
                        );
                    }
                    if !visible_mail_delivery(&current) {
                        result.push("mail delivery must be a complete, directly visible mail in one named agent bubble; source detail may be folded separately".into());
                    }
                }
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn delivery(version: u32) -> String {
        format!(
            "> [!activity-message|codex] Codex\n> <!-- viggobsidian:mail-delivery:v1 id=follow-up version={version} -->\n> **Mailconcept v{version}**\n> <!-- viggobsidian:mail-draft:v1 recipients=resolved -->\n> Mailstatus: concept\n> Bron: [[fictional-contact]]\n> **To**\n> ```text\n> team@example.org\n> ```\n> **Onderwerp**\n> ```text\n> Afspraak v{version}\n> ```\n> **Body**\n> ```text\n> Hallo team,\n> Dit is de volledige versie {version}.\n> Groet, Alex\n> ```\n> <!-- /viggobsidian:mail-draft -->"
        )
    }
    #[test]
    fn complete_versions_alternate_with_human_input_without_rewriting_history() {
        let human =
            format!("> [!activity-message|human] Alex\n> {INPUT}\n> Graag de afspraak aanpassen.");
        let body = format!(
            "{}\n\n{human}\n\n{}\n\n{human}\n\n{}",
            delivery(1),
            delivery(2),
            delivery(3)
        );
        assert!(findings(&body).is_empty());
        assert!(body.ends_with("<!-- /viggobsidian:mail-draft -->"));
        assert!(
            findings(&body.replace("version=3", "version=2"))
                .iter()
                .any(|s| s.contains("strictly increasing"))
        );
        assert!(!findings(&human.replace("|human]", "|codex]")).is_empty());
    }
    #[test]
    fn length_exception_requires_complete_visible_delivery_not_a_marker() {
        let good = delivery(1);
        assert!(visible_mail_delivery(&good.lines().collect::<Vec<_>>()));
        for broken in [
            good.replace("> **Body**", "> **Wijziging**"),
            good.replace("team@example.org", "onbekend"),
            good.replace(
                "> <!-- viggobsidian:mail-draft",
                "> > <!-- viggobsidian:mail-draft",
            ),
            good.replace("version=1", "version=0"),
        ] {
            assert!(!visible_mail_delivery(&broken.lines().collect::<Vec<_>>()));
            assert!(!findings(&broken).is_empty());
        }
        assert!(
            findings(&format!(
                "````markdown\n{}\n````",
                good.replace("version=1", "version=0")
            ))
            .is_empty()
        );
    }
    #[test]
    fn long_complete_delivery_stays_visible_but_examples_never_activate() {
        let long = delivery(1).replace(
            "Dit is de volledige versie 1.",
            &"Volledige bruikbare mailtekst. ".repeat(100),
        );
        let event = format!("\n## Activity\n\n> [!activity-day|2026-11-11] 11 Nov 2026\n\n{long}");
        assert!(crate::activity_layout::check_v11(&event).is_ok());
        assert!(
            crate::activity_layout::check_v11(&event.replace("> **Body**", "> **Wijziging**"))
                .is_err()
        );
        assert!(findings(&format!("````markdown\n\n{}\n\n````", delivery(0))).is_empty());
        for extra in [
            format!("{event}\n> {}", "Losse toelichting. ".repeat(200)),
            event.replace(
                "> <!-- viggobsidian:mail-draft:v1",
                &format!(
                    "> {}\n> <!-- viggobsidian:mail-draft:v1",
                    "Losse toelichting. ".repeat(200)
                ),
            ),
        ] {
            assert!(crate::activity_layout::check_v11(&extra).is_err());
        }
        let literal=delivery(1).replace("> Hallo team,", "> > ```\n> <!-- viggobsidian:mail-delivery:v1 id=example version=0 -->\n> <!-- viggobsidian:input:v1 author=human -->\n> Hallo team,");
        assert!(findings(&literal).is_empty());
        assert!(
            findings(&format!(
                "Uitleg: `{INPUT}` en `{DELIVERY}id=example version=0 -->`."
            ))
            .is_empty()
        );
    }
}
