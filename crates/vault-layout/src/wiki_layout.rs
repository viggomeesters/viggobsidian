use crate::shortcut_links::{anchor_count, heading};

const FACTS: &str = "> [!wiki-facts]";
const TOC: &str = "```wiki-toc";

pub fn findings(body: &str) -> (bool, Vec<(&'static str, &'static str)>) {
    let mut findings = Vec::new();
    let lines = body.lines().map(str::trim_end).collect::<Vec<_>>();
    let start = lines
        .iter()
        .position(|line| !line.is_empty())
        .unwrap_or(lines.len());
    let mut intro = lines[start..].iter().copied().peekable();
    let title = intro.next().is_some_and(|line| {
        line.strip_prefix("# ")
            .is_some_and(|title| !title.trim().is_empty())
    });
    let after_title = intro.next_if_eq(&"").is_some();
    while intro.next_if_eq(&"").is_some() {}
    let shortcuts = after_title
        && intro.next() == Some("> [!navigation|shortcuts]")
        && intro
            .next()
            .is_some_and(|line| shortcut(line, "Back") || shortcut(line, "Terug"))
        && intro.next().is_some_and(|line| shortcut(line, "Raw"))
        && intro.next().is_some_and(|line| {
            line.strip_prefix("> - [Finder](file:///")
                .and_then(|target| target.strip_suffix(')'))
                .is_some_and(|target| !target.trim().is_empty())
        })
        && intro.peek().is_none_or(|line| !line.starts_with('>'));
    if !title || !shortcuts {
        findings.push((
            "wiki-preamble",
            "start Wiki with a title and the Back/Raw/Finder shortcuts",
        ));
    }

    let after_shortcuts = intro.next_if_eq(&"").is_some();
    let facts_position =
        after_shortcuts && intro.clone().find(|line| !line.is_empty()) == Some(FACTS);
    if facts_position {
        intro.find(|line| *line == FACTS);
        let mut facts = Vec::new();
        while let Some(line) = intro.next_if(|line| line.starts_with('>')) {
            facts.push(line);
        }
        if !(2..=4).contains(&facts.len()) || !facts.into_iter().all(valid_fact) {
            findings.push((
                "wiki-facts",
                "use two to four facts with a label and a bold value",
            ));
        }
    }
    let after_facts = !facts_position || intro.next_if_eq(&"").is_some();
    let toc_position = after_shortcuts
        && after_facts
        && intro.find(|line| !line.is_empty()) == Some(TOC)
        && intro.next() == Some("```")
        && intro.peek().is_none_or(|line| line.is_empty());

    let mut previous = None;
    let mut h2 = Vec::new();
    let mut facts_count = 0;
    let mut toc_count = 0;
    let mut fence = None;
    let mut comment = None;
    for line in lines {
        let trimmed = line.trim_start_matches(' ');
        let marker = fence_marker(trimmed).filter(|_| line.len() - trimmed.len() <= 3);
        if let Some((opening, minimum)) = fence {
            if marker.is_some_and(|(ch, len)| {
                ch == opening && len >= minimum && trimmed[len..].trim().is_empty()
            }) {
                fence = None;
            }
            continue;
        }

        let visible = visible_line(line, &mut comment);
        let trimmed = visible.trim_start_matches(' ');
        if visible.len() - trimmed.len() > 3 {
            previous = None;
            continue;
        }
        if let Some((ch, len)) = fence_marker(trimmed) {
            if trimmed[len..].split_whitespace().next() == Some("wiki-toc") {
                toc_count += 1;
            }
            fence = Some((ch, len));
            previous = None;
            continue;
        }
        if trimmed.starts_with(FACTS) {
            facts_count += 1;
        }
        if let Some((level, title)) = heading(trimmed, previous.as_deref()) {
            if level == 2 {
                h2.push(title.to_owned());
            }
            previous = None;
        } else {
            previous = (!trimmed.trim().is_empty() && !trimmed.starts_with('>'))
                .then(|| trimmed.trim().to_owned());
        }
    }
    if facts_count > usize::from(facts_position) {
        findings.push((
            "wiki-facts-position",
            "put at most one wiki-facts block directly below the shortcuts",
        ));
    }
    if (h2.len() >= 4 && (toc_count != 1 || !toc_position)) || (h2.len() < 4 && toc_count != 0) {
        findings.push(("wiki-toc", "from four H2 headings, put one empty wiki-toc code block after the shortcuts and optional facts, before the dossier text"));
    }
    if toc_position && h2.iter().any(|title| anchor_count(body, title) > 1) {
        findings.push((
            "wiki-toc-anchor",
            "give each contents entry a distinct heading target",
        ));
    }
    (facts_position, findings)
}

pub(crate) fn shortcut(line: &str, label: &str) -> bool {
    line.strip_prefix("> - [[")
        .and_then(|link| link.strip_suffix("]]"))
        .and_then(|link| link.split_once('|'))
        .is_some_and(|(target, alias)| !target.trim().is_empty() && alias == label)
}

fn valid_fact(line: &str) -> bool {
    line.strip_prefix("> - ")
        .and_then(|item| item.strip_suffix("**"))
        .and_then(|item| item.split_once(" **"))
        .is_some_and(|(label, value)| {
            !label.trim().is_empty() && !value.trim().is_empty() && !value.contains("**")
        })
}

fn fence_marker(line: &str) -> Option<(u8, usize)> {
    let marker = *line.as_bytes().first()?;
    let length = line.bytes().take_while(|&byte| byte == marker).count();
    (matches!(marker, b'`' | b'~') && length >= 3).then_some((marker, length))
}

fn visible_line(line: &str, comment: &mut Option<&'static str>) -> String {
    let mut visible = String::new();
    let mut rest = line;
    loop {
        if let Some(end) = *comment {
            let Some((_, after)) = rest.split_once(end) else {
                break;
            };
            rest = after;
            *comment = None;
        } else if let Some((index, start, end)) = [("<!--", "-->"), ("%%", "%%")]
            .into_iter()
            .filter_map(|(start, end)| rest.find(start).map(|index| (index, start, end)))
            .min_by_key(|&(index, _, _)| index)
        {
            visible.push_str(&rest[..index]);
            rest = &rest[index + start.len()..];
            *comment = Some(end);
        } else {
            visible.push_str(rest);
            break;
        }
    }
    visible
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTRO: &str = "# Wiki\n\n> [!navigation|shortcuts]\n> - [[dossier|Terug]]\n> - [[dossier/raw/raw|Raw]]\n> - [Finder](file:///vault/dossier/wiki)\n\n";
    const CARDS: &str = "> [!wiki-facts]\n> - Perceel **185 m²**\n> - Label **A++++**\n\n";
    const MARKER: &str = "```wiki-toc\n```\n\n";
    const SECTIONS: &str = "## Woning\n\n## Bouw\n\n## Wonen\n\n## Stand\n";

    fn codes(body: &str) -> Vec<&'static str> {
        findings(body).1.into_iter().map(|(code, _)| code).collect()
    }

    #[test]
    fn contents_are_required_at_four_real_headings() {
        let examples = "```md\n## Example\n```\n<!--\n## Hidden\n-->\n### Subheading\n";
        let short = format!("{INTRO}## Een\n## Twee\n## Drie\n{examples}");
        assert!(codes(&short).is_empty());
        assert_eq!(codes(&format!("{INTRO}{SECTIONS}")), ["wiki-toc"]);
        assert!(codes(&format!("{INTRO}{MARKER}{SECTIONS}")).is_empty());
        assert_eq!(codes(&format!("{INTRO}{MARKER}## Een\n")), ["wiki-toc"]);
    }

    #[test]
    fn checks_order_shape_and_duplicates() {
        assert!(codes(&format!("{INTRO}{CARDS}{MARKER}{SECTIONS}")).is_empty());
        assert_eq!(
            codes(&format!("{INTRO}{MARKER}{CARDS}{SECTIONS}")),
            ["wiki-facts-position"]
        );
        assert_eq!(
            codes(&format!("{INTRO}{CARDS}{SECTIONS}{MARKER}")),
            ["wiki-toc"]
        );
        assert_eq!(
            codes(&format!("{INTRO}{MARKER}{SECTIONS}\n~~~wiki-toc\n~~~\n")),
            ["wiki-toc"]
        );
        assert_eq!(
            codes(&format!(
                "{INTRO}> [!wiki-facts]\n> - Perceel **185 m²**\n\n{MARKER}{SECTIONS}"
            )),
            ["wiki-facts"]
        );
        assert_eq!(
            codes(&format!("{INTRO}{MARKER}{SECTIONS}\n## Woning\n")),
            ["wiki-toc-anchor"]
        );
    }
}
