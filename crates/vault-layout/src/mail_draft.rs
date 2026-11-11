//! Explicit per-draft opt-in; archival email and task metadata remain untouched.
const START: &str = "<!-- viggobsidian:mail-draft:v1 recipients=";
const END: &str = "<!-- /viggobsidian:mail-draft -->";

fn quoted(mut line: &str) -> (usize, &str) {
    let mut depth = 0;
    while let Some(rest) = line.strip_prefix('>') {
        depth += 1;
        line = rest.strip_prefix(' ').unwrap_or(rest);
    }
    (depth, line)
}

fn address(value: &str) -> bool {
    value.split_once('@').is_some_and(|(local, domain)| {
        !local.is_empty()
            && !local.contains('@')
            && !local.starts_with('.')
            && !local.ends_with('.')
            && !local.contains("..")
            && domain.contains('.')
            && !domain.starts_with('.')
            && !domain.ends_with('.')
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".!#$%&'*+-/=?^_`{|}~@".contains(&b))
            && !domain.contains('@')
            && domain.split('.').all(|part| {
                !part.is_empty()
                    && !part.starts_with('-')
                    && !part.ends_with('-')
                    && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            })
    })
}

pub(crate) fn findings(body: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut bubble = false;
    let mut active = false;
    let mut resolved = false;
    let mut depth = 0;
    let mut fence = 0;
    let mut label = String::new();
    let mut fields: Vec<(String, String)> = Vec::new();
    let mut content = String::new();
    let mut source = false;
    let mut warning = false;
    let mut state = false;
    let mut example_fence: Option<(u8, usize)> = None;
    for raw in body.lines() {
        let (mut level, mut line) = quoted(raw);
        if active && fence > 0 {
            level = 0;
            line = raw;
            for _ in 0..depth {
                if let Some(rest) = line.strip_prefix('>') {
                    level += 1;
                    line = rest.strip_prefix(' ').unwrap_or(rest);
                } else {
                    break;
                }
            }
        }
        if !active {
            let candidate = line.trim_start_matches(' ');
            let character = candidate.as_bytes().first().copied().unwrap_or(0);
            let width = candidate.bytes().take_while(|b| *b == character).count();
            if let Some((kind, opening)) = example_fence {
                if character == kind && width >= opening && candidate[width..].trim().is_empty() {
                    example_fence = None;
                }
                continue;
            }
            if matches!(character, b'`' | b'~') && width >= 3 {
                example_fence = Some((character, width));
                continue;
            }
        }
        if active && (level == 0 || crate::activity_layout::message_heading(raw).is_some()) {
            result.push("mail draft must remain inside one named Activity bubble".into());
            active = false;
            fence = 0;
        }
        if level == 0 {
            bubble = false;
        }
        if crate::activity_layout::message_heading(raw).is_some() {
            bubble = true;
        }
        if fence == 0 && line.starts_with(START) {
            if active
                || !bubble
                || !matches!(
                    line,
                    "<!-- viggobsidian:mail-draft:v1 recipients=resolved -->"
                        | "<!-- viggobsidian:mail-draft:v1 recipients=unresolved -->"
                )
            {
                result.push(
                    "mail draft needs a valid v1 marker inside a named Activity bubble".into(),
                );
            }
            active = true;
            resolved = line.contains("=resolved");
            depth = level;
            fields.clear();
            content.clear();
            label.clear();
            source = false;
            warning = false;
            state = false;
            continue;
        }
        if !active {
            continue;
        }
        if level != depth {
            result.push(
                "mail fields, fences and provenance must share the marker quote depth".into(),
            );
        }
        if fence > 0 {
            let indent = line.bytes().take_while(|b| *b == b' ').count().min(3);
            let candidate = &line[indent..];
            let closing = candidate.bytes().take_while(|b| *b == b'`').count();
            if closing >= fence && candidate[closing..].trim().is_empty() {
                fields.push((label.clone(), content.trim_end_matches('\n').into()));
                content.clear();
                fence = 0;
            } else {
                content.push_str(line);
                content.push('\n');
            }
            continue;
        }
        if line == END {
            let roles: Vec<_> = fields.iter().map(|(key, _)| key.as_str()).collect();
            let expected: Vec<_> = ["To", "Cc", "Bcc", "Onderwerp", "Body"]
                .into_iter()
                .filter(|role| match *role {
                    "To" => resolved || roles.contains(role),
                    "Cc" | "Bcc" => roles.contains(role),
                    _ => true,
                })
                .collect();
            if roles != expected {
                result.push("use separate ordered To, optional Cc/Bcc, Onderwerp and Body text blocks; unresolved To may be omitted".into());
            }
            let mut seen = std::collections::BTreeSet::new();
            for (role, value) in &fields {
                if value.trim().is_empty() {
                    result.push(format!("{role}: empty copy block"));
                }
                if matches!(role.as_str(), "To" | "Cc" | "Bcc") {
                    for item in value.split("; ") {
                        if !address(item) || !seen.insert(item.to_ascii_lowercase()) {
                            result.push(format!("{role}: use unique verified addresses separated by semicolon-space, without names or placeholders"));
                        }
                    }
                } else if value.contains("[[")
                    || value.contains("**")
                    || value.contains("](")
                    || value.starts_with(&format!("{role}:"))
                    || (role == "Onderwerp" && value.contains('\n'))
                {
                    result.push(format!(
                        "{role}: copy plain text without labels or Markdown formatting"
                    ));
                }
            }
            if !source || !state || (!resolved && !warning) {
                result.push("keep Bron: with a source link, Mailstatus: concept, and any Ontbreekt: recipient warning outside the copy blocks inside this bubble".into());
            }
            active = false;
            continue;
        }
        let width = line.bytes().take_while(|b| *b == b'`').count();
        if width >= 3 {
            if line[width..].trim() != "text" {
                result.push("mail copy fences must use text".into());
            }
            fence = width;
            continue;
        }
        if let Some(value) = line.strip_prefix("**").and_then(|v| v.strip_suffix("**")) {
            label = value.into();
        } else if !line.is_empty()
            && !line.starts_with("Bron: ")
            && !line.starts_with("Ontbreekt: ")
            && line != "Mailstatus: concept"
        {
            result.push("keep copy text inside its field; only provenance, concept status and recipient warnings belong outside the fields".into());
        }
        if line.starts_with("Bron: ")
            && ((line.contains("[[") && line.contains("]]")) || line.contains("https://"))
        {
            source = true;
        }
        if line == "Mailstatus: concept" {
            state = true;
        }
        if line.starts_with("Ontbreekt: ") && line.len() > "Ontbreekt: ".len() {
            warning = true;
        }
        if !resolved
            && (line.to_lowercase().contains("verzendklaar")
                || line.to_lowercase().contains("ready to send"))
        {
            result.push("unresolved recipients cannot be marked ready to send".into());
        }
    }
    if active {
        result.push(
            "close the mail draft marker and all text fences inside its Activity bubble".into(),
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn draft() -> String {
        "> [!activity-message|assistant] Assistant\n> <!-- viggobsidian:mail-draft:v1 recipients=resolved -->\n> Mailstatus: concept\n> Bron: [[fictional-contact]]\n> **To**\n> ```text\n> team@example.org\n> ```\n> **Onderwerp**\n> ```text\n> Afspraak\n> ```\n> **Body**\n> ````text\n> Hallo,\n>\n> Bekijk https://example.org en typ ``` als voorbeeld.\n> Groet\n> ````\n> <!-- /viggobsidian:mail-draft -->".into()
    }
    #[test]
    fn independent_plaintext_fields_and_long_fences() {
        assert!(findings(&draft()).is_empty());
    }
    #[test]
    fn optional_roles_and_folded_draft() {
        let text = draft().replace("> **Onderwerp**", "> **Cc**\n> ```text\n> copy@example.org\n> ```\n> **Bcc**\n> ```text\n> blind@example.org\n> ```\n> **Onderwerp**");
        assert!(findings(&text).is_empty());
        let (heading, rest) = text.split_once('\n').unwrap();
        assert!(
            findings(&format!(
                "{heading}\n> > [!info]- Meer\n{}",
                rest.lines()
                    .map(|l| format!("> {l}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
            .is_empty()
        );
    }
    #[test]
    fn missing_recipient_is_explicit_not_placeholder() {
        let text = draft().replace("=resolved", "=unresolved").replace(
            "> **To**\n> ```text\n> team@example.org\n> ```\n",
            "> Ontbreekt: adres voor de bedoelde ontvanger\n",
        );
        assert!(findings(&text).is_empty());
        assert!(
            !findings(&text.replace("Mailstatus: concept", "Mailstatus: verzendklaar")).is_empty()
        );
        assert!(
            !findings(&text.replace("> Ontbreekt: adres voor de bedoelde ontvanger\n", ""))
                .is_empty()
        );
    }
    #[test]
    fn rejects_format_provenance_and_containment_failures() {
        for (from, to) in [
            ("team@example.org", "Team <team@example.org>"),
            ("team@example.org", "nog invullen"),
            ("Bron: [[fictional-contact]]", "Bron: onbekend"),
            ("Afspraak", "Onderwerp: Afspraak"),
            ("Hallo,", "**Hallo**, "),
            ("> **Body**\n", ""),
            ("> <!-- /viggobsidian:mail-draft -->", ""),
            ("> **To**", "**To**"),
        ] {
            assert!(!findings(&draft().replace(from, to)).is_empty(), "{from}");
        }
    }
    #[test]
    fn history_is_untouched_and_multiple_drafts_are_independent() {
        assert!(findings("legacy mail **To** not formatted").is_empty());
        assert!(findings(&format!("{}\n\n{}", draft(), draft())).is_empty());
    }
    #[test]
    fn longer_closing_fences_follow_renderer_rules() {
        assert!(findings(&draft().replace("> ````\n> <!--", "> `````  \n> <!--")).is_empty());
        assert!(!findings(&draft().replace("> Hallo,", "> `````\n> Hallo,")).is_empty());
        assert!(!findings(&draft().replace("team@example.org", "team@@example.org")).is_empty());
        assert!(!findings(&draft().replace("> Hallo,", ">   `````\n> Hallo,")).is_empty());
        assert!(
            findings(&draft().replace("> Hallo,", "> > geciteerde reply\n> Hallo,")).is_empty()
        );
    }
    #[test]
    fn fenced_historical_examples_are_not_live_drafts() {
        for fence in ["````", "~~~~"] {
            assert!(
                findings(&format!(
                    "{fence}markdown\n{}\n{fence}\n",
                    draft().replace("team@example.org", "not-an-address")
                ))
                .is_empty()
            );
        }
        assert!(
            findings(&format!(
                "> ```markdown\n{}\n> ```",
                draft().replace("team@example.org", "not-an-address")
            ))
            .is_empty()
        );
    }
}
