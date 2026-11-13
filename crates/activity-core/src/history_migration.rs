//! Explicit text-only migration of historical Activity into a current wrapper.
//! The wrapper's author dates the formatting action, never the archived source.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn field<'a>(v: &'a Value, key: &str) -> Result<&'a str, &'static str> {
    v[key].as_str().ok_or("missing string request field")
}
fn safe_field(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.trim() == s
        && !s.chars().any(|c| {
            c.is_control()
                || matches!(
                    c,
                    '\u{2028}' | '\u{2029}' | '<' | '>' | '&' | '*' | '[' | ']' | '`' | '|'
                )
        })
}

fn literal_width(source: &str) -> usize {
    source
        .split(|c| c != '`')
        .map(str::len)
        .max()
        .unwrap_or(0)
        .max(2)
        + 1
}

fn control_line(line: &str) -> bool {
    let mut text = line.trim();
    while let Some(rest) = text.strip_prefix('>') {
        text = rest.trim_start();
    }
    text.starts_with("<!-- viggobsidian:") || text.starts_with("<!-- /viggobsidian:")
}

fn history(text: &str, width: usize) -> String {
    let fence = "`".repeat(width);
    text.split_inclusive('\n')
        .map(|line| {
            if control_line(line) {
                let ending = if line.ends_with("\r\n") { "\r\n" } else { "\n" };
                format!(
                    "> > {fence} {} {fence}{ending}",
                    line.trim_end_matches(['\r', '\n'])
                )
            } else {
                format!("> > {line}{}", if line.ends_with('\n') { "" } else { "\n" })
            }
        })
        .collect()
}

fn history_size(text: &str, width: usize) -> Result<usize, &'static str> {
    let mut size = 0usize;
    for line in text.split_inclusive('\n') {
        let overhead = if control_line(line) {
            width.checked_mul(2).and_then(|n| n.checked_add(6))
        } else {
            Some(4)
        }
        .ok_or("candidate output budget exceeded")?;
        size = size
            .checked_add(line.len())
            .and_then(|n| n.checked_add(overhead))
            .and_then(|n| n.checked_add(usize::from(!line.ends_with('\n'))))
            .ok_or("candidate output budget exceeded")?;
        if size > crate::MAX_NOTE_BYTES {
            return Err("candidate output budget exceeded");
        }
    }
    Ok(size)
}

fn unquote(mut line: &str, limit: usize) -> (usize, &str) {
    let mut depth = 0;
    while depth < limit {
        let trimmed = line.trim_start_matches(' ');
        if line.len() - trimmed.len() > 3 {
            break;
        }
        let Some(rest) = trimmed.strip_prefix('>') else {
            break;
        };
        depth += 1;
        line = rest.strip_prefix(' ').unwrap_or(rest);
    }
    (depth, line)
}

fn atx(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let level = trimmed.bytes().take_while(|b| *b == b'#').count();
    if !(1..=6).contains(&level) || !trimmed[level..].starts_with([' ', '\t']) {
        return None;
    }
    let title = trimmed[level..].trim();
    let without = title.trim_end_matches('#');
    Some((
        level,
        if without.ends_with([' ', '\t']) {
            without.trim_end()
        } else {
            title
        },
    ))
}

/// Only live ATX headings are eligible; quoted examples and fences cannot be
/// selected as original native navigation. The same scan identifies headings
/// in derived excerpt rendering so they can be demoted without duplicating IDs.
fn headings<'a>(lines: &[&'a str]) -> Vec<(usize, usize, &'a str)> {
    let mut result = Vec::new();
    let mut fence: Option<(u8, usize, usize)> = None;
    let mut comment = false;
    for (i, line) in lines.iter().enumerate() {
        let (depth, outer) = unquote(line, usize::MAX);
        if fence.is_some_and(|(_, _, d)| depth < d) {
            fence = None;
        }
        let text = if let Some((_, _, d)) = fence {
            unquote(line, d).1
        } else {
            outer
        };
        let trimmed = text.trim_start_matches(' ');
        let marker = trimmed.as_bytes().first().copied().unwrap_or(0);
        let width = trimmed.bytes().take_while(|b| *b == marker).count();
        if let Some((opening, min, _)) = fence {
            if marker == opening && width >= min && trimmed[width..].trim().is_empty() {
                fence = None;
            }
            continue;
        }
        if text.len() - trimmed.len() <= 3 && matches!(marker, b'`' | b'~') && width >= 3 {
            fence = Some((marker, width, depth));
            continue;
        }
        if comment {
            if trimmed.contains("-->") {
                comment = false;
            }
            continue;
        }
        if trimmed.contains("<!--") && !trimmed.contains("-->") {
            comment = true;
            continue;
        }
        if let Some((level, title)) = atx(outer) {
            result.push((i + 1, level, title));
        }
    }
    result
}

/// Supply the actual current action date/time/author/channel. This pure preview
/// cannot establish their truth, source ownership or permission to write.
/// `processing_lines` is an optional inclusive source line range containing the
/// already recorded processing payload (without its historical heading/footer).
/// The caller reviews that range; no prose or source links are manufactured.
pub fn preview(source: &str, request: &Value) -> Result<String, &'static str> {
    let object = request.as_object().ok_or("request object required")?;
    if object.keys().any(|k| {
        ![
            "schema",
            "activity_line",
            "date",
            "time",
            "author",
            "channel",
            "processing_lines",
            "passages",
        ]
        .contains(&k.as_str())
    }) {
        return Err("unknown request field");
    }
    if request["schema"] != "viggobsidian.activity-history-migration.v1" {
        return Err("unsupported migration schema");
    }
    if source.len() > crate::MAX_NOTE_BYTES {
        return Err("migration requires bounded UTF-8 text");
    }
    let date = field(request, "date")?;
    let author = field(request, "author")?;
    let channel = field(request, "channel")?;
    let time = field(request, "time")?;
    if !crate::iso_date(date)
        || ![author, channel, time].iter().all(|s| safe_field(s))
        || !crate::activity_delivery::valid_stamp(&format!("{time} · {author} · {channel}"), author)
    {
        return Err("invalid migration action provenance");
    }
    let yaml_end = source
        .strip_prefix("---\n")
        .and_then(|s| s.find("\n---\n"))
        .map(|i| i + 4)
        .ok_or("existing YAML frontmatter required")?;
    let yaml = &source[4..yaml_end];
    // Do not rearchive an already opted-in note, or silently reinterpret YAML.
    let options = serde_saphyr::options! {duplicate_keys:serde_saphyr::DuplicateKeyPolicy::Error,merge_keys:serde_saphyr::MergeKeyPolicy::Error};
    let metadata: Value = serde_saphyr::from_str_with_options(yaml, options)
        .map_err(|_| "valid unique-key YAML mapping required")?;
    if !metadata.is_object() {
        return Err("YAML mapping required");
    }
    if metadata.get("activity_contract").is_some() {
        return Err("existing activity contract requires separate review");
    }
    let lines: Vec<_> = source.split_inclusive('\n').collect();
    let selected = request["activity_line"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .filter(|n| *n > 0 && *n <= lines.len())
        .ok_or("explicit Activity line required")?;
    let activities: Vec<_> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| **l == "## Activity\n")
        .map(|(i, _)| i + 1)
        .collect();
    if activities != [selected] {
        return Err("select the sole original outer Activity heading");
    }
    // The selected heading must be live Markdown, not a source-code example.
    let prefix_len: usize = lines[..selected - 1].iter().map(|s| s.len()).sum();
    let prefix = &source[..prefix_len];
    if prefix_len < yaml_end + 5 {
        return Err("Activity heading is inside frontmatter");
    }
    let body_prefix = &prefix[yaml_end + 5..];
    if prefix.contains('\r') {
        return Err("existing header requires LF; historical payload may retain CRLF");
    }
    let mut fence = None;
    for line in body_prefix.lines() {
        let trimmed = line.trim_start();
        if line.len() - trimmed.len() > 3 {
            continue;
        }
        let marker = trimmed.as_bytes().first().copied().unwrap_or(0);
        let width = trimmed.bytes().take_while(|b| *b == marker).count();
        if matches!(marker, b'`' | b'~') && width >= 3 {
            match fence {
                None => fence = Some((marker, width)),
                Some((opening, min))
                    if opening == marker && width >= min && trimmed[width..].trim().is_empty() =>
                {
                    fence = None
                }
                _ => {}
            }
        }
    }
    if fence.is_some() {
        return Err("Activity heading is inside a code fence");
    }
    let tail = &source[prefix_len + "## Activity\n".len()..];
    if tail.trim().is_empty() {
        return Err("historical Activity is empty");
    }
    if tail
        .as_bytes()
        .windows(2)
        .any(|w| w[0] == b'\r' && w[1] != b'\n')
        || tail.ends_with('\r')
    {
        return Err("standalone historical carriage return is unsupported");
    }
    let mut processing = String::new();
    if !request["processing_lines"].is_null() {
        let range = request["processing_lines"]
            .as_array()
            .filter(|r| r.len() == 2)
            .ok_or("processing range requires first and last source lines")?;
        let first = range[0]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("invalid processing first line")?;
        let last = range[1]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("invalid processing last line")?;
        if first <= selected || last < first || last > lines.len() {
            return Err("processing range is outside historical Activity");
        }
        for line in &lines[first - 1..last] {
            let content = line
                .strip_prefix("> ")
                .or_else(|| line.strip_prefix('>'))
                .ok_or("processing payload must be originally quoted")?;
            if content.starts_with("[!activity-")
                || content.trim_start().starts_with("<!-- viggobsidian:")
                || content.starts_with('*')
            {
                return Err("select processing payload without historical author or control lines");
            }
            processing.push_str("> ");
            processing.push_str(content);
            if !content.ends_with('\n') {
                processing.push('\n');
            }
        }
        if !processing.contains("[[") || processing.len() > 1200 {
            return Err("review a compact original processing summary and its source link");
        }
    }
    let width = literal_width(source);
    let expected = prefix_len
        .checked_add(history_size(tail, width)?)
        .and_then(|n| n.checked_add(processing.len() + 1000))
        .ok_or("candidate output budget exceeded")?;
    if expected > crate::MAX_NOTE_BYTES {
        return Err("candidate output budget exceeded");
    }
    let mut result = String::new();
    result.push_str(&source[..yaml_end + 1]);
    result.push_str("activity_contract: provenance-v1\n");
    result.push_str(&source[yaml_end + 1..prefix_len]);
    result.push_str("## Activity\n\n");
    result.push_str(&format!("> [!activity-day|{date}] {date}\n\n> [!activity-message|assistant] {author}\n> <!-- viggobsidian:activity:v1 kind=result goal=activity-format-migration -->\n> Opmaakmigratie. De historische inhoud hieronder is letterlijk behouden; auteurs en momenten gelden zoals oorspronkelijk vastgelegd.\n"));
    if !processing.is_empty() {
        result.push_str(">\n> **Verwerking in Wiki**\n> Historisch vastgelegde verwerking, zonder nieuwe inhoudelijke beoordeling:\n");
        result.push_str(&processing);
        result.push_str(&format!(">\n> *{time} · {author} · {channel}*\n\n> [!activity-message|assistant] {author}\n> <!-- viggobsidian:activity:v1 kind=result goal=activity-history-preservation -->\n"));
    }
    result.push_str(&format!(
        "> <!-- viggobsidian:history:v1 literal-width={width} final-newline={} -->\n>\n> > [!activity-more]- Meer\n",usize::from(tail.ends_with('\n'))
    ));
    result.push_str(&history(tail, width));
    result.push_str(&format!(">\n> *{time} · {author} · {channel}*\n"));
    if !request["passages"].is_null() {
        let passages = request["passages"]
            .as_array()
            .ok_or("passages must be an array")?;
        if passages.len() > 128 {
            return Err("at most 128 explicit passage copies per note");
        }
        let original_headings = headings(&lines);
        let heading_by_line: BTreeMap<_, _> = original_headings
            .iter()
            .map(|(n, level, title)| (*n, (*level, *title)))
            .collect();
        let mut title_counts = BTreeMap::<&str, usize>::new();
        for (_, _, title) in &original_headings {
            *title_counts.entry(title).or_default() += 1;
        }
        let mut ids = BTreeSet::new();
        let mut selections = BTreeSet::new();
        for passage in passages {
            let p = passage.as_object().ok_or("passage object required")?;
            if p.keys()
                .any(|k| !["first_line", "last_line", "block_id"].contains(&k.as_str()))
            {
                return Err("unknown passage field");
            }
            let first = passage["first_line"]
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or("passage first line required")?;
            let last = passage["last_line"]
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or("passage last line required")?;
            let id = field(passage, "block_id")?;
            if id.is_empty()
                || id.len() > 64
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                || !ids.insert(id)
                || source.contains(&format!("^{id}"))
            {
                return Err("invalid, duplicated or preexisting passage block ID");
            }
            if first <= selected || last < first || last > lines.len() || !selections.insert(first)
            {
                return Err("invalid or repeated passage range");
            }
            let (level, title) = atx(lines[first - 1])
                .ok_or("passage must start at an original unquoted ATX heading")?;
            if !heading_by_line.contains_key(&first) || title_counts.get(title) != Some(&1) {
                return Err("passage heading is fenced or ambiguous");
            }
            let boundary = original_headings
                .iter()
                .filter(|(n, l, _)| *n > first && *l <= level && atx(lines[*n - 1]).is_some())
                .map(|(n, _, _)| *n - 1)
                .next()
                .unwrap_or(lines.len());
            if last != boundary {
                return Err(
                    "passage must preserve its complete original section, including children",
                );
            }
            let mut excerpt = String::new();
            for number in first..=last {
                let line = lines[number - 1];
                if let Some((_, text)) = heading_by_line.get(&number) {
                    let (_, content) = unquote(line, usize::MAX);
                    excerpt.push_str(&line[..line.len() - content.len()]);
                    excerpt.push_str(&format!("**{text}**\n"));
                } else {
                    excerpt.push_str(line);
                }
            }
            let expected = result
                .len()
                .checked_add(history_size(&excerpt, width)?)
                .and_then(|n| {
                    n.checked_add(
                        author.len() * 2 + channel.len() + time.len() + id.len() * 2 + 600,
                    )
                })
                .ok_or("candidate output budget exceeded")?;
            if expected > crate::MAX_NOTE_BYTES {
                return Err("candidate output budget exceeded");
            }
            result.push_str(&format!("\n> [!activity-message|assistant] {author}\n> <!-- viggobsidian:activity:v1 kind=result goal=passage-{id} -->\n> Historische bronpassage, als navigatiekopie van de ongewijzigde vastlegging hierboven.\n>\n> > [!activity-more]- Meer\n"));
            result.push_str(&history(&excerpt, width));
            result.push_str(&format!(
                ">\n> ^{id}\n>\n> *{time} · {author} · {channel}*\n"
            ));
        }
    }
    let (_, body) = result
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n"))
        .ok_or("invalid candidate frontmatter")?;
    if !crate::activity_delivery::provenance_findings(body).is_empty() {
        return Err("candidate fails Activity provenance checks");
    }
    if !crate::activity_delivery::findings(body).is_empty()
        || !crate::mail_draft::findings(body).is_empty()
    {
        return Err("candidate changes active input or delivery contracts");
    }
    if result.len() > crate::MAX_NOTE_BYTES {
        return Err("candidate output budget exceeded");
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    const SOURCE: &str = "---\ntype: task\ntask-id: task:00000000-0000-4000-8000-000000000001\nstatus: 🟠 in progress\ncreated: 2020-01-01\nwork_date: 2020-01-02\n---\n# Fictional task\n\n## Activity\n\n> [!activity-day|2020-01-01] 1 januari 2020\n\n> [!activity-message|agent] Unknown historical agent\n> Original input.\n\n## Oorspronkelijke taak\n\nLiteral original task text and source-id=fictional-1.\n\n### Verwerking in Wiki\n\n> [!activity-message|codex] Previous author\n> Original processing summary.\n> [[fictional/wiki/wiki#Decision|Decision]]\n>\n> *Previous author*\n";
    fn request() -> Value {
        json!({"schema":"viggobsidian.activity-history-migration.v1","activity_line":10,"date":"2026-11-13","time":"—","author":"Codex","channel":"Codex-chat"})
    }
    #[test]
    fn preserves_prefix_and_every_historical_byte_without_claiming_historical_provenance() {
        let result = preview(SOURCE, &request()).unwrap();
        let original_prefix = SOURCE.split("## Activity\n").next().unwrap();
        assert!(result.starts_with(&original_prefix.replacen(
            "\n---\n",
            "\nactivity_contract: provenance-v1\n---\n",
            1
        )));
        let archive = result
            .split("> > [!activity-more]- Meer\n")
            .nth(1)
            .unwrap()
            .split(">\n> *— · Codex · Codex-chat*\n")
            .next()
            .unwrap();
        let restored: String = archive
            .split_inclusive('\n')
            .map(|line| line.strip_prefix("> > ").unwrap())
            .collect();
        assert_eq!(restored, SOURCE.split_once("## Activity\n").unwrap().1);
        assert!(
            crate::activity_delivery::provenance_findings(result.split_once("\n---\n").unwrap().1)
                .is_empty()
        );
        assert!(crate::activity_delivery::findings(&result).is_empty());
        assert_eq!(result.lines().filter(|l| *l == "## Activity").count(), 1);
    }
    #[test]
    fn rejects_ambiguous_scope_and_false_fields() {
        for source in [
            SOURCE.replace('\n', "\r\n"),
            format!("{SOURCE}\n## Activity\n"),
            SOURCE.replacen(
                "type: task",
                "activity_contract: provenance-v1\ntype: task",
                1,
            ),
            SOURCE.replacen("# Fictional task", "```markdown\n# Fictional task", 1),
        ] {
            assert!(preview(&source, &request()).is_err());
        }
        for (key, value) in [
            ("date", json!("2026-02-30")),
            ("time", json!("24:01")),
            ("author", json!("<b>Actor</b>")),
            ("activity_line", json!(13)),
            ("unknown", json!(true)),
        ] {
            let mut r = request();
            r[key] = value;
            assert!(preview(SOURCE, &r).is_err(), "{r}");
        }
    }
    #[test]
    fn direct_processing_is_an_exact_selected_source_excerpt() {
        let lines: Vec<_> = SOURCE.lines().collect();
        let first = lines
            .iter()
            .position(|l| *l == "> Original processing summary.")
            .unwrap()
            + 1;
        let mut r = request();
        r["processing_lines"] = json!([first, first + 1]);
        let result = preview(SOURCE, &r).unwrap();
        assert!(result.contains("> **Verwerking in Wiki**\n> Historisch vastgelegde verwerking, zonder nieuwe inhoudelijke beoordeling:\n> Original processing summary.\n> [[fictional/wiki/wiki#Decision|Decision]]\n"));
        r["processing_lines"] = json!([first - 1, first + 1]);
        assert!(preview(SOURCE, &r).is_err());
    }

    #[test]
    fn historical_control_markers_are_literal_and_roundtrip_with_unambiguous_delimiters() {
        let source = format!(
            "{SOURCE}\n> <!-- viggobsidian:input:v1 author=human -->\r\n> <!-- viggobsidian:activity:v1 kind=result goal=historical -->\n> <!-- viggobsidian:mail-delivery:v1 id=historical version=1 -->\r\n> Original `inline` and ``` fence examples.\n> <!-- /viggobsidian:mail-draft -->"
        );
        let result = preview(&source, &request()).unwrap();
        let width = literal_width(&source);
        let fence = "`".repeat(width);
        let archive = result
            .split("> > [!activity-more]- Meer\n")
            .nth(1)
            .unwrap()
            .split(">\n> *— · Codex · Codex-chat*\n")
            .next()
            .unwrap();
        let mut restored = String::new();
        for line in archive.split_inclusive('\n') {
            let text = line.strip_prefix("> > ").unwrap();
            if let Some(text) = text.strip_prefix(&format!("{fence} ")) {
                let ending = if text.ends_with("\r\n") { "\r\n" } else { "\n" };
                restored.push_str(text.strip_suffix(&format!(" {fence}{ending}")).unwrap());
                restored.push_str(ending);
            } else {
                restored.push_str(text);
            }
        }
        assert!(result.contains("final-newline=0"));
        assert_eq!(
            restored.strip_suffix('\n').unwrap(),
            source.split_once("## Activity\n").unwrap().1
        );
        assert!(crate::activity_delivery::findings(&result).is_empty());
    }

    #[test]
    fn navigation_copies_preserve_parent_extent_and_keep_one_source_heading() {
        let source = format!(
            "{SOURCE}\n## Parent section\n\nParent paragraph.\n\n### Child section\n\nChild paragraph.\n\n## Next section\n\nOther paragraph.\n"
        );
        let lines: Vec<_> = source.lines().collect();
        let first = lines
            .iter()
            .position(|l| *l == "## Parent section")
            .unwrap()
            + 1;
        let child = lines
            .iter()
            .position(|l| *l == "### Child section")
            .unwrap()
            + 1;
        let last = lines.iter().position(|l| *l == "## Next section").unwrap();
        let mut r = request();
        r["passages"] = json!([
            {"first_line":first,"last_line":last,"block_id":"historical-parent"},
            {"first_line":child,"last_line":last,"block_id":"historical-child"}
        ]);
        let result = preview(&source, &r).unwrap();
        let parent = result
            .split("goal=passage-historical-parent -->")
            .nth(1)
            .unwrap()
            .split("\n> [!activity-message|")
            .next()
            .unwrap();
        assert!(parent.contains("Parent paragraph."));
        assert!(parent.contains("Child paragraph."));
        assert!(!parent.contains("Other paragraph."));
        assert!(parent.contains("> ^historical-parent\n>\n> *— · Codex · Codex-chat*"));
        assert_eq!(
            result
                .lines()
                .filter(|l| l.ends_with("## Parent section"))
                .count(),
            1
        );
        r["passages"][0]["last_line"] = json!(child - 1);
        assert!(preview(&source, &r).is_err());
        r["passages"][0]["last_line"] = json!(last);
        r["passages"][1]["block_id"] = json!("historical-parent");
        assert!(preview(&source, &r).is_err());
        let ambiguous = source.replace("## Next section", "## Parent section");
        r["passages"] =
            json!([{"first_line":first,"last_line":last,"block_id":"historical-parent"}]);
        assert!(preview(&ambiguous, &r).is_err());
    }

    #[test]
    fn retains_crlf_payload_and_records_absent_final_newline() {
        let source = format!("{SOURCE}\nOriginal mail\r\nSecond original line");
        let result = preview(&source, &request()).unwrap();
        assert!(result.contains("final-newline=0"));
        let archive = result
            .split("> > [!activity-more]- Meer\n")
            .nth(1)
            .unwrap()
            .split(">\n> *— · Codex · Codex-chat*\n")
            .next()
            .unwrap();
        let restored: String = archive
            .split_inclusive('\n')
            .map(|l| l.strip_prefix("> > ").unwrap())
            .collect();
        assert_eq!(
            restored.strip_suffix('\n').unwrap(),
            source.split_once("## Activity\n").unwrap().1
        );
        for broken in [
            SOURCE.replace("Original input.", "Original\rinput."),
            format!("{SOURCE}\r"),
            format!("{SOURCE}bad\r\r\n"),
        ] {
            assert!(preview(&broken, &request()).is_err());
        }
    }

    #[test]
    fn refuses_semantically_existing_contracts_and_bounded_expansion_before_encoding() {
        for key in [
            "\"activity_contract\"",
            "activity_contract ",
            "'activity_contract'",
        ] {
            let source = SOURCE.replacen(
                "type: task",
                &format!("{key}: provenance-v1\ntype: task"),
                1,
            );
            assert!(preview(&source, &request()).is_err());
        }
        let source = format!(
            "{SOURCE}\n{} example\n{}",
            "`".repeat(4096),
            "> <!-- viggobsidian:input:v1 author=human -->\n".repeat(200)
        );
        assert!(source.len() < crate::MAX_NOTE_BYTES);
        assert_eq!(
            preview(&source, &request()),
            Err("candidate output budget exceeded")
        );
        let mut r = request();
        r["author"] = json!("Bad\u{001b}Actor");
        assert!(preview(SOURCE, &r).is_err());
        let merged = SOURCE.replacen(
            "type: task",
            "defaults: &defaults {activity_contract: provenance-v1}\n<<: *defaults\ntype: task",
            1,
        );
        assert!(preview(&merged, &request()).is_err());
        let literal = SOURCE.replacen(
            "type: task",
            "description: '<< is literal source prose'\ntype: task",
            1,
        );
        let mut r = request();
        r["activity_line"] = json!(11);
        assert!(preview(&literal, &r).is_ok());
    }

    #[test]
    fn many_headings_use_indexed_excerpt_lookup_and_passage_requests_are_capped() {
        let mut source = SOURCE.to_owned();
        for i in 0..4000 {
            source.push_str(&format!("\n### Source subsection {i}\n\nParagraph {i}.\n"));
        }
        source.push_str("\n## Next source section\n");
        let first = source
            .lines()
            .position(|l| l == "## Oorspronkelijke taak")
            .unwrap()
            + 1;
        let last = source
            .lines()
            .position(|l| l == "## Next source section")
            .unwrap();
        let mut r = request();
        r["passages"] = json!([{"first_line":first,"last_line":last,"block_id":"many-headings"}]);
        assert!(preview(&source, &r).is_ok());
        r["passages"] = json!(vec![
            json!({"first_line":first,"last_line":last,"block_id":"too-many"});
            129
        ]);
        assert_eq!(
            preview(&source, &r),
            Err("at most 128 explicit passage copies per note")
        );
    }
}
