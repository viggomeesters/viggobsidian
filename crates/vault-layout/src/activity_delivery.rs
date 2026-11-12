//! Opt-in delivery chronology, separate from task state and historical text.
use std::collections::BTreeMap;
const DELIVERY: &str = "<!-- viggobsidian:mail-delivery:v1 ";
const INPUT: &str = "<!-- viggobsidian:input:v1 author=human -->";

/// Canonical, renderer-native header. No arbitrary HTML or Markdown is allowed
/// in provenance fields; the author remains the callout's accessible title.
pub(crate) fn header_stamp(line: &str, author: &str) -> Option<bool> {
    let (_, title) = line
        .strip_prefix("> [!activity-message|")?
        .split_once("] ")?;
    if !title.contains("activity-meta") && !title.contains('<') {
        return None;
    }
    Some(
        title
            .strip_prefix(author)
            .and_then(|s| s.strip_prefix(" <span class=\"activity-meta\">"))
            .and_then(|s| s.strip_suffix("</span>"))
            .and_then(|s| s.split_once(" · "))
            .is_some_and(|(time, channel)| {
                !channel.contains(" · ")
                    && !title.chars().any(|c| matches!(c, '\n' | '\r'))
                    && !author
                        .chars()
                        .any(|c| matches!(c, '<' | '>' | '&' | '`' | '*' | '[' | ']'))
                    && !channel
                        .chars()
                        .any(|c| matches!(c, '<' | '>' | '&' | '`' | '*' | '[' | ']'))
                    && valid_stamp(&format!("{time} · {author} · {channel}"), author)
            }),
    )
}

pub(crate) fn valid_stamp(stamp: &str, author: &str) -> bool {
    let parts: Vec<_> = stamp.split(" · ").collect();
    let [time, name, channel] = parts.as_slice() else {
        return false;
    };
    let clock = *time == "—"
        || time.split_once(':').is_some_and(|(hour, minute)| {
            hour.len() == 2
                && minute.len() == 2
                && hour
                    .bytes()
                    .chain(minute.bytes())
                    .all(|b| b.is_ascii_digit())
                && hour.parse::<u8>().is_ok_and(|n| n < 24)
                && minute.parse::<u8>().is_ok_and(|n| n < 60)
        });
    clock
        && *name == author
        && !channel.trim().is_empty()
        && *channel == channel.trim()
        && !channel.contains('`')
}

/// Whole-note opt-in: validate visible provenance without interpreting source prose.
pub(crate) fn provenance_findings(body: &str) -> Vec<String> {
    if let Err(message) = crate::activity_layout::check_v11(&format!("\n{body}")) {
        return vec![message.into()];
    }
    let lines = visible_lines(body);
    let sections: Vec<_> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "## Activity")
        .map(|(i, _)| i)
        .collect();
    if sections.len() != 1 {
        return vec!["provenance-v1 requires exactly one Activity section".into()];
    }
    let lines = &lines[sections[0] + 1..];
    let mut result = Vec::new();
    let mut day = None;
    let mut goals = BTreeMap::new();
    let mut messages = 0;
    for (index, line) in lines.iter().enumerate() {
        if let Some((date, _)) = line
            .strip_prefix("> [!activity-day|")
            .and_then(|v| v.split_once("] "))
        {
            if !crate::iso_date(date) || day.is_some_and(|previous| previous > date) {
                result.push("use valid local Activity dates in chronological order".into());
            }
            day = Some(date);
        }
        let Some((role, author)) = crate::activity_layout::message_heading(line) else {
            continue;
        };
        messages += 1;
        if !day.is_some_and(crate::iso_date) {
            result.push("every stamped Activity bubble needs its original local day".into());
        }
        let end = lines[index + 1..]
            .iter()
            .position(|l| {
                l.is_empty()
                    || crate::activity_layout::message_heading(l).is_some()
                    || l.starts_with("> [!activity-day|")
            })
            .map_or(lines.len(), |n| index + 1 + n);
        let message = &lines[index..end];
        let raw_lines: Vec<_> = body.lines().collect();
        let start = raw_lines
            .iter()
            .position(|raw| raw.as_ptr() == line.as_ptr())
            .unwrap();
        let raw_message: Vec<_> = raw_lines[start..]
            .iter()
            .copied()
            .take_while(|raw| !raw.trim().is_empty())
            .collect();
        let stamp = raw_message
            .get(1)
            .and_then(|l| l.strip_prefix("> `"))
            .and_then(|s| s.strip_suffix('`'));
        // Only direct, visible final italic paragraphs are provenance. Nested
        // details and fenced/copyable source text must never supply a footer.
        let footers: Vec<_> = message
            .iter()
            .filter_map(|line| {
                line.strip_prefix("> *")?
                    .strip_suffix('*')
                    .filter(|text| text.split(" · ").count() >= 3)
            })
            .collect();
        let footer = footers.first().copied();
        let header = header_stamp(line, author);
        let legacy = stamp.is_some_and(|s| s.split(" · ").count() == 3);
        if header == Some(false)
            || usize::from(header.is_some()) + usize::from(legacy) + footers.len() > 1
        {
            result.push(format!(
                "{author}: invalid or duplicate Activity provenance"
            ));
        } else if let Some(footer) = footer {
            let expected = format!("> *{footer}*");
            let last = raw_message.iter().rposition(|line| *line != ">");
            if !valid_stamp(footer, author)
                || footer
                    .chars()
                    .any(|c| matches!(c, '<' | '>' | '&' | '*' | '[' | ']' | '`'))
                || !last.is_some_and(|i| {
                    raw_message[i] == expected && i > 1 && raw_message[i - 1] == ">"
                })
            {
                result.push(format!("{author}: end the outer bubble with a separate *HH:mm · {author} · channel* paragraph; use — for unknown time"));
            }
        } else if header.is_none() && !stamp.is_some_and(|stamp| valid_stamp(stamp, author)) {
            result.push(format!("{author}: end the bubble with *HH:mm · {author} · channel*, or use — for unknown time (legacy top stamps and headers remain supported)"));
        }
        if role == "human" {
            continue;
        }
        let prefix = "> <!-- viggobsidian:activity:v1 ";
        let markers: Vec<_> = message
            .iter()
            .filter(|l| {
                unquote(l, usize::MAX)
                    .1
                    .starts_with("<!-- viggobsidian:activity:")
            })
            .collect();
        let goal = (markers.len() == 1)
            .then(|| markers[0])
            .and_then(|l| l.strip_prefix(prefix))
            .and_then(|v| v.strip_suffix(" -->"))
            .and_then(|v| v.strip_prefix("kind=result goal="));
        let Some(goal) = goal.filter(|g| {
            !g.is_empty()
                && g.len() <= 80
                && g.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        }) else {
            result.push(format!("{author}: bind one result bubble to a valid activity goal; merge standalone process messages into that result"));
            continue;
        };
        // Mail deliveries may repeat a goal only as complete, versioned results.
        let delivered = visible_mail_delivery(&raw_message);
        if delivered
            && !message
                .iter()
                .any(|l| identity(l).is_some_and(|(id, _)| id == goal))
        {
            result.push(format!(
                "goal {goal}: mail delivery id must match the result goal"
            ));
        }
        if let Some(previous) = goals.insert(goal, delivered)
            && !(previous && delivered)
        {
            result.push(format!(
                "goal {goal}: group process history in one result bubble instead of repeating it"
            ));
        }
    }
    if messages == 0 {
        result.push("provenance-v1 requires named Activity bubbles".into());
    }
    result
}

fn visible_lines(body: &str) -> Vec<&str> {
    let mut fence = None;
    body.lines()
        .filter(|line| {
            let (depth, outer) = unquote(line, usize::MAX);
            // A quoted fence ends with its containing blockquote, unlike a
            // top-level fence. It cannot hide the next Activity bubble.
            if fence.is_some_and(|(_, _, opening_depth)| opening_depth > 0 && depth < opening_depth)
            {
                fence = None;
            }
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
            lines.iter().any(|line| {
                *line == format!("> **Mailconcept v{v}**")
                    || *line == format!("> ### Mailconcept v{v}")
            })
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
    #[test]
    fn footer_is_final_direct_unique_and_preserves_italic_source() {
        let good = "## Activity\n\n> [!activity-day|2026-11-11] 11 Nov 2026\n\n> [!activity-message|human] Alexandra van Voorbeeld\n> *Gewone cursieve inhoud.*\n> [Source](https://example.org/message)\n>\n> > [!activity-more]- Meer\n> > Detail.\n>\n> *— · Alexandra van Voorbeeld · Mail (summary)*\n";
        assert!(provenance_findings(good).is_empty());
        for broken in [
            good.replace("· Alexandra van Voorbeeld ·", "· Someone Else ·"),
            good.replace("*— ·", "*25:00 ·"),
            good.replace("> *— ·", "> > *— ·"),
            good.replace(">\n> *— ·", "> *— ·"),
            format!("{good}> After the footer.\n"),
            format!("{good}>\n> *12:00 · Alexandra van Voorbeeld · Chat*\n"),
            good.replace(
                "> *Gewone",
                "> `— · Alexandra van Voorbeeld · Chat`\n> *Gewone",
            ),
            good.replace(
                "Alexandra van Voorbeeld\n",
                "Alexandra van Voorbeeld <span class=\"activity-meta\">— · Chat</span>\n",
            ),
            good.replace("> *— ·", "> ```text\n> *— ·")
                .replace("(summary)*", "(summary)*\n> ```"),
            good.replace("Mail (summary)", "<b>Mail</b>"),
            good.replace("Mail (summary)", "Mail · extra"),
        ] {
            assert!(!provenance_findings(&broken).is_empty(), "{broken}");
        }
        let mail = include_str!("../fixtures/activity-deliveries.md");
        for check in [provenance_findings, findings, crate::mail_draft::findings] {
            assert!(check(mail).is_empty(), "{mail}");
        }
    }
    fn stamped() -> String {
        format!(
            "## Activity\n\n> [!activity-day|2026-11-11] 11 Nov 2026\n\n> [!activity-message|human] Alex\n> `23:59 · Alex · Codex-chat`\n> {INPUT}\n> Graag aanpassen.\n\n> [!activity-day|2026-11-12] 12 Nov 2026\n\n> [!activity-message|codex] Codex\n> `00:01 · Codex · draft`\n> <!-- viggobsidian:activity:v1 kind=result goal=follow-up -->\n> Aangepast resultaat."
        )
    }
    #[test]
    fn header_provenance_retains_legacy_and_rejects_ambiguous_or_unsafe_headers() {
        let legacy = stamped();
        let header = legacy
            .replace(
                "Alex\n> `23:59 · Alex · Codex-chat`",
                "Alex <span class=\"activity-meta\">23:59 · Codex-chat</span>",
            )
            .replace(
                "Codex\n> `00:01 · Codex · draft`",
                "Codex <span class=\"activity-meta\">00:01 · draft</span>",
            );
        for good in [&legacy, &header, &header.replace("23:59", "—")] {
            assert!(provenance_findings(good).is_empty(), "{good}");
        }
        for broken in [
            header.replace("23:59", "24:00"),
            header.replace("23:59", "01:+1"),
            header.replace("23:59", "9:01"),
            header.replace("Codex-chat</span>", "</span>"),
            header.replace("Codex-chat</span>", " chat </span>"),
            header.replace("Codex-chat</span>", "chat · Alex</span>"),
            header.replace("Codex-chat</span>", "chat</span><script>x</script>"),
            header.replace("Codex-chat</span>", "chat</span> extra"),
            header.replace("Codex-chat</span>", "<b>chat</b></span>"),
            header.replace(
                "Codex-chat</span>",
                "chat</span><span class=\"activity-meta\">00:01 · chat</span>",
            ),
            header.replace(
                "Codex-chat</span>\n",
                "Codex-chat</span>\n> `23:59 · Alex · chat`\n",
            ),
        ] {
            assert!(!provenance_findings(&broken).is_empty(), "{broken}");
        }
        let long = header
            .replace("Alex", "Alexandra van Voorbeeld")
            .replace("Codex-chat", "Workshop met een lange kanaalnaam");
        assert!(provenance_findings(&long).is_empty());
        let inline_input = header.replace("> Graag aanpassen.", "> `git status`");
        assert!(provenance_findings(&inline_input).is_empty());
        assert!(
            !provenance_findings(&header.replace(
                "Codex-chat</span>\n",
                "Codex-chat</span>\n> `25:00 · Someone · chat`\n"
            ))
            .is_empty()
        );
    }
    #[test]
    fn complete_header_deliveries_keep_input_and_version_checks() {
        let fixture = include_str!("../fixtures/activity-deliveries.md");
        assert!(provenance_findings(fixture).is_empty());
        assert!(findings(fixture).is_empty());
        assert!(crate::mail_draft::findings(fixture).is_empty());
        assert!(!findings(&fixture.replace("version=3", "version=1")).is_empty());
        assert!(!findings(&fixture.replace("|human] Alex", "|agent] Alex")).is_empty());
    }
    #[test]
    fn provenance_requires_real_author_clock_channel_and_local_day() {
        let good = stamped();
        assert!(provenance_findings(&good).is_empty());
        assert!(provenance_findings(&good.replace("23:59", "—")).is_empty());
        for broken in [
            good.replace("> `23:59 · Alex · Codex-chat`\n", ""),
            good.replace(
                "> `23:59 · Alex · Codex-chat`",
                "> ```text\n> Example\n> ```\n> `23:59 · Alex · Codex-chat`",
            ),
            good.replace("23:59", "24:00"),
            good.replace("23:59", "23:60"),
            good.replace("23:59", "9:04"),
            good.replace("23:59", "+1:00"),
            good.replace("23:59", "01:+1"),
            good.replace("· Alex ·", "· Codex ·"),
            good.replace(" · Codex-chat", ""),
            good.replace("Codex-chat`", " `"),
            good.replace("2026-11-12", "2026-11-10"),
            good.replace("> [!activity-day|2026-11-11] 11 Nov 2026\n", ""),
        ] {
            assert!(!provenance_findings(&broken).is_empty(), "{broken}");
        }
    }
    #[test]
    fn process_bubbles_must_be_grouped_by_explicit_result_goal() {
        let good = stamped();
        for broken in [
            good.replace("kind=result", "kind=process"),
            good.replace("goal=follow-up", "goal="),
            good.replace(
                "> <!-- viggobsidian:activity:v1 kind=result goal=follow-up -->",
                "",
            ),
            good.replace(
                "> <!-- viggobsidian:activity:v1",
                "> > <!-- viggobsidian:activity:v1",
            ),
            format!(
                "{good}\n\n> [!activity-message|codex] Codex\n> `00:02 · Codex · draft`\n> <!-- viggobsidian:activity:v1 kind=result goal=follow-up -->\n> Nog een losse melding."
            ),
        ] {
            assert!(!provenance_findings(&broken).is_empty(), "{broken}");
        }
        let with_example = good.replace("> Aangepast resultaat.", "> Aangepast resultaat.\n> ```markdown\n> <!-- viggobsidian:activity:v1 kind=process goal=bad -->\n> [!activity-message|human] Fake\n> ```");
        assert!(provenance_findings(&with_example).is_empty());
        let escaped = "## Activity\n\n> [!activity-day|2026-11-12] 12 Nov 2026\n\n> [!activity-message|human] Alex\n> `00:01 · Alex · Codex-chat`\n> > ```text\n> > Literal source.\n\n> [!activity-message|codex] Codex\n> Missing stamp and goal.";
        assert!(!provenance_findings(escaped).is_empty());
        assert!(!provenance_findings(&escaped.replace("> > ```text", "> ```text")).is_empty());
        assert!(!provenance_findings(&stamped().replace("|human] Alex", "|human] ")).is_empty());
    }
    #[test]
    fn versioned_mail_results_can_share_a_goal_without_losing_provenance() {
        let delivery_with_stamp = |v| {
            delivery(v).replace("> [!activity-message|codex] Codex\n", "> [!activity-message|codex] Codex\n> `00:01 · Codex · draft`\n> <!-- viggobsidian:activity:v1 kind=result goal=follow-up -->\n")
        };
        let body = format!(
            "## Activity\n\n> [!activity-day|2026-11-12] 12 Nov 2026\n\n{}\n\n{}",
            delivery_with_stamp(1),
            delivery_with_stamp(2)
        );
        assert!(provenance_findings(&body).is_empty());
        assert!(findings(&body).is_empty());
        assert!(
            findings(
                &body
                    .replace("> **Mailconcept v", "> ### Mailconcept v")
                    .replace("v1**", "v1")
                    .replace("v2**", "v2")
            )
            .is_empty()
        );
        assert!(!provenance_findings(&body.replace("> **Body**", "> **Missing**")).is_empty());
    }
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
