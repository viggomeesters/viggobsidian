fn valid_date(date: &str) -> bool {
    let parts: Vec<_> = date.split(' ').collect();
    let [day, month, year] = parts.as_slice() else {
        return false;
    };
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    year.len() == 4
        && months
            .iter()
            .position(|name| name == month)
            .is_some_and(|index| crate::valid_date(year, &(index + 1).to_string(), day))
}

fn valid_stamp(line: &str) -> bool {
    let width = line.bytes().take_while(|&byte| byte == b'`').count();
    if width == 0 || line.len() <= width * 2 || !line.ends_with(&line[..width]) {
        return false;
    }
    let parts: Vec<_> = line[width..line.len() - width].split(" · ").collect();
    let [time, actor, channel, rest @ ..] = parts.as_slice() else {
        return false;
    };
    let clock = time.split_once(':').is_some_and(|(hour, minute)| {
        hour.len() == 2
            && minute.len() == 2
            && hour.parse::<u8>().is_ok_and(|hour| hour < 24)
            && minute.parse::<u8>().is_ok_and(|minute| minute < 60)
    });
    (clock || *time == "—")
        && !actor.trim().is_empty()
        && actor
            .chars()
            .all(|c| c.is_alphabetic() || matches!(c, ' ' | '-' | '.'))
        && !channel.trim().is_empty()
        && (matches!(rest, [] | ["confirmed"])
            || matches!(rest, [change] if change.split_once(" → ").is_some_and(|(from, to)| !from.is_empty() && !to.is_empty())))
}

pub(crate) fn message_heading(line: &str) -> Option<(&str, &str)> {
    line.strip_prefix("> [!activity-message|")?
        .split_once("] ")
        .map(|(role, actor)| {
            (
                role,
                actor
                    .split_once(" <span class=\"activity-meta\">")
                    .map_or(actor, |(name, _)| name),
            )
        })
        .filter(|(role, actor)| {
            role.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                && role
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
                && actor == &actor.trim()
                && actor.chars().any(char::is_alphabetic)
        })
}

pub(crate) fn check_task_event(body: &str) -> Result<(), &'static str> {
    let visible = crate::index_navigation::visible_body(body);
    let (_, activity) = visible
        .split_once("\n## Activity\n")
        .ok_or("Raw tasks need an Activity section with Event Note v1 bubbles")?;
    if activity.lines().any(|line| line == "## Activity") {
        return Err("use one Activity section for Raw tasks");
    }
    let lines: Vec<_> = activity.lines().collect();
    let mut dated = false;
    let mut has_message = false;
    for block in lines.split(|line| line.trim().is_empty()) {
        let Some((&heading, lines)) = block.split_first() else {
            continue;
        };
        if heading.starts_with("## ") {
            break;
        }
        if let Some((date, label)) = heading
            .strip_prefix("> [!activity-day|")
            .and_then(|day| day.split_once("] "))
        {
            if !crate::iso_date(date) || label.trim().is_empty() || !lines.is_empty() {
                return Err("use a dated Event Note v1 Activity separator");
            }
            dated = true;
        } else if message_heading(heading).is_some() {
            if !dated
                || !lines.iter().any(|line| {
                    line.strip_prefix("> ")
                        .is_some_and(|text| !text.trim().is_empty())
                })
                || !lines.iter().all(|line| line.starts_with('>'))
            {
                return Err(
                    "put each named, nonempty Event Note v1 bubble under a dated separator",
                );
            }
            has_message = true;
        } else if !heading.starts_with("### ") || !lines.is_empty() {
            return Err(
                "use Event Note v1 bubbles for Raw task Activity instead of legacy history",
            );
        }
    }
    if !has_message {
        return Err("Raw tasks need at least one Event Note v1 Activity bubble");
    }
    Ok(())
}

/// V1.1 has no unwrapped tail: provenance and archives are messages too.
pub(crate) fn check_v11(body: &str) -> Result<(), &'static str> {
    let (_, activity) = body
        .split_once("\n## Activity\n")
        .ok_or("Event Note v1.1 needs Activity")?;
    let lines: Vec<_> = activity.lines().collect();
    let mut dated = false;
    let mut messages = 0;
    for block in lines
        .split(|line| line.trim().is_empty())
        .filter(|block| !block.is_empty())
    {
        let (first, rest) = block.split_first().unwrap();
        if let Some((date, label)) = first
            .strip_prefix("> [!activity-day|")
            .and_then(|v| v.split_once("] "))
        {
            if !crate::iso_date(date) || label.trim().is_empty() || !rest.is_empty() {
                return Err("use a valid, standalone Activity day separator");
            }
            dated = true;
        } else if message_heading(first).is_some() {
            if !dated || rest.is_empty() || !rest.iter().all(|line| line.starts_with('>')) {
                return Err("keep every Activity message fully inside its dated bubble");
            }
            let mut direct = 0;
            let mail_payload = crate::activity_delivery::visible_mail_payload_lines(block);
            let mut folded = false;
            let mut fence = None;
            let mut nested = false;
            for (index, line) in rest.iter().enumerate() {
                let was_nested = nested;
                nested = line.starts_with("> >");
                if !nested {
                    folded = false;
                    fence = None;
                }
                if !was_nested
                    && fence.is_none()
                    && matches!(*line, "> > [!info]- Meer" | "> > [!activity-more]- Meer")
                {
                    folded = true;
                }
                if !folded && !mail_payload.contains(&(index + 1)) {
                    direct += line.len();
                }
                let content = crate::index_navigation::quote_content(line);
                let marker = content
                    .as_bytes()
                    .first()
                    .copied()
                    .filter(|byte| matches!(byte, b'`' | b'~'));
                let width = marker.map_or(0, |marker| {
                    content.bytes().take_while(|byte| *byte == marker).count()
                });
                if width >= 3 {
                    match fence {
                        None => fence = marker.map(|marker| (marker, width)),
                        Some((opening, minimum))
                            if marker == Some(opening)
                                && width >= minimum
                                && content[width..].trim().is_empty() =>
                        {
                            fence = None
                        }
                        _ => {}
                    }
                }
            }
            if direct > 1600 {
                return Err("put long Activity details inside a nested, folded Info bubble (Meer)");
            }
            messages += 1;
        } else {
            return Err(
                "after Activity use only dated message bubbles; put provenance, source content and archives inside bubbles",
            );
        }
    }
    if messages == 0 {
        return Err("Event Note v1.1 needs at least one Activity message");
    }
    Ok(())
}

pub fn findings(body: &str) -> Vec<String> {
    let mut findings = Vec::new();
    let mut lines = body.lines().enumerate().peekable();
    let mut activity = false;
    let mut fence = None;
    let mut count = 0;
    while let Some((line, value)) = lines.next() {
        let trimmed = value.trim_start();
        let marker = trimmed
            .as_bytes()
            .first()
            .copied()
            .filter(|byte| matches!(byte, b'`' | b'~'));
        if let Some(marker) = marker {
            let length = trimmed.bytes().take_while(|&byte| byte == marker).count();
            if length >= 3 {
                match fence {
                    None => fence = Some((marker, length)),
                    Some((open, width))
                        if open == marker
                            && length >= width
                            && trimmed[length..].trim().is_empty() =>
                    {
                        fence = None
                    }
                    _ => {}
                }
                continue;
            }
        }
        if fence.is_some() {
            continue;
        }
        if value == "## Activity" {
            activity = true;
        } else if activity && let Some(heading) = value.strip_prefix("# ") {
            count += 1;
            if !heading
                .split_once(" · ")
                .is_some_and(|(title, date)| !title.trim().is_empty() && valid_date(date))
            {
                findings.push(format!(
                    "body line {}: use an H1 with title · D Mon YYYY",
                    line + 1
                ));
            }
            if !lines.peek().is_some_and(|(_, stamp)| valid_stamp(stamp)) {
                findings.push(format!(
                    "body line {}: follow the H1 with time · author · channel",
                    line + 1
                ));
            }
        }
    }
    if count == 0 {
        findings.push("chat-v1 requires an Activity section with at least one message".into());
    }
    findings
}

#[cfg(test)]
mod tests {
    #[test]
    fn v11_rejects_unwrapped_tails_for_every_note_type() {
        let valid = "# Note\n\n## Activity\n\n> [!activity-day|2026-11-11] 11 november 2026\n\n> [!activity-message|codex] Codex\n> Herkomst.\n>\n> > [!info]- Meer\n> > ### Details\n> > | A | B |\n> > ```text\n> > source\n> > ```\n>\n> *Codex*\n";
        assert_eq!(super::check_v11(valid), Ok(()));
        for tail in [
            "## Herkomst",
            "### Verwerking in Wiki",
            "losse tekst",
            "| A | B |",
            "```text\nbron\n```",
            "> [!info]- Meer\n> bron",
            "## Activity",
        ] {
            assert!(
                super::check_v11(&format!("{valid}\n{tail}\n")).is_err(),
                "{tail}"
            );
        }
        assert!(super::check_v11(&valid.replace("Herkomst.", &"x".repeat(1601))).is_err());
        let large = format!("> > {}\n", "x".repeat(2000));
        assert!(
            super::check_v11(
                &valid.replace("> > [!info]- Meer", &format!("{large}> > [!info]- Meer"))
            )
            .is_err()
        );
        assert!(
            super::check_v11(&valid.replace(
                "> > [!info]- Meer",
                &format!("> > ```text\n> > [!info]- Meer\n{large}> > ```")
            ))
            .is_err()
        );
        assert!(
            super::check_v11(&valid.replace(
                "> > [!info]- Meer",
                &format!("> > ordinary paragraph\n> > [!info]- Meer\n{large}")
            ))
            .is_err()
        );
        assert_eq!(
            super::check_v11(&valid.replace("> > ### Details", &format!("{large}> > ### Details"))),
            Ok(())
        );
    }
    #[test]
    fn raw_task_activity_requires_real_bubbles_and_rejects_legacy_messages() {
        const DAY: &str = "> [!activity-day|2026-11-08] 8 november 2026";
        const BUBBLE: &str =
            "> [!activity-message|codex] Codex\n> Taak afgerond.\n>\n> *14:47 · vault-task*";
        const LEGACY: &str =
            "# 🟢 done · 8 Nov 2026\n`14:47 · Agent · vault-task`\n\nTaak afgerond.";
        let note = |history: &str| format!("# Task\n\n## Activity\n\n{history}\n");
        let valid = format!("{DAY}\n\n{BUBBLE}");
        assert_eq!(super::check_task_event(&note(&valid)), Ok(()));
        for invalid in [
            LEGACY.to_owned(),
            format!("{valid}\n\n{LEGACY}"),
            format!("{valid}\n\n## Activity\n\n{LEGACY}"),
            format!("{DAY}\n\n```md\n{BUBBLE}\n```"),
            format!("{DAY}\n\n## Evidence\n\n{BUBBLE}"),
            BUBBLE.to_owned(),
            valid.replace("2026-11-08", "2026-11-31"),
            valid.replace("|codex]", "| ]"),
            format!("{DAY}\n\n> [!activity-message|codex] Codex"),
        ] {
            assert!(
                super::check_task_event(&note(&invalid)).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn activity_dates_must_exist_on_the_calendar() {
        assert!(super::valid_date("29 Feb 2024"));
        assert!(!super::valid_date("29 Feb 2026"));
        assert!(!super::valid_date("31 Apr 2026"));
        assert!(!super::valid_date("1 Jan 26"));
    }
}
