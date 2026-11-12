//! Explicit, read-only conversion preview. Source bytes are never written.
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, env, fs};

fn stamp(text: &str, author: &str) -> bool {
    let parts: Vec<_> = text.split(" · ").collect();
    let [time, name, channel] = parts.as_slice() else {
        return false;
    };
    let clock = *time == "—"
        || time.split_once(':').is_some_and(|(h, m)| {
            h.len() == 2
                && m.len() == 2
                && h.bytes().chain(m.bytes()).all(|b| b.is_ascii_digit())
                && h.parse::<u8>().is_ok_and(|n| n < 24)
                && m.parse::<u8>().is_ok_and(|n| n < 60)
        });
    clock
        && *name == author
        && !channel.is_empty()
        && *channel == channel.trim()
        && !text
            .chars()
            .any(|c| matches!(c, '<' | '>' | '&' | '*' | '[' | ']' | '`' | '\r' | '\n'))
}

fn unquote(mut text: &str, limit: usize) -> (usize, &str) {
    let mut depth = 0;
    while depth < limit {
        let Some(rest) = text.strip_prefix('>') else {
            break;
        };
        depth += 1;
        text = rest.strip_prefix(' ').unwrap_or(rest);
    }
    (depth, text)
}

fn convert(source: &str, selected: &BTreeSet<usize>) -> Result<String, String> {
    if selected.is_empty() || source.contains('\r') {
        return Err("select at least one original heading line; only LF input is supported".into());
    }
    let mut lines: Vec<String> = source.split_inclusive('\n').map(str::to_owned).collect();
    // A selected heading must be live outer Activity, never a fenced example.
    let mut activity = false;
    let mut fence = None;
    let mut eligible = BTreeSet::new();
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim_end_matches('\n');
        let (depth, outer) = unquote(line, usize::MAX);
        // A quoted fence terminates when its containing blockquote ends.
        if fence.is_some_and(|(_, _, opening_depth)| depth < opening_depth) {
            fence = None;
        }
        let text = if let Some((_, _, opening_depth)) = fence {
            unquote(line, opening_depth).1
        } else {
            outer
        };
        let text = text
            .strip_prefix("   ")
            .or_else(|| text.strip_prefix("  "))
            .or_else(|| text.strip_prefix(' '))
            .unwrap_or(text);
        let byte = text.as_bytes().first().copied().unwrap_or(0);
        let width = text.bytes().take_while(|b| *b == byte).count();
        if let Some((opening, minimum, _)) = fence {
            if byte == opening && width >= minimum && text[width..].trim().is_empty() {
                fence = None;
            }
            continue;
        }
        if matches!(byte, b'`' | b'~') && width >= 3 {
            fence = Some((byte, width, depth));
            continue;
        }
        if line == "## Activity" {
            activity = true;
        } else if line.starts_with('#') {
            activity = false;
        }
        if activity && line.starts_with("> [!activity-message|") {
            eligible.insert(i + 1);
        }
    }
    if !selected.is_subset(&eligible) {
        return Err("selected line is not a live outer Activity heading".into());
    }
    for &number in selected.iter().rev() {
        let start = number - 1;
        let end = lines[start + 1..]
            .iter()
            .position(|l| l.trim().is_empty())
            .map_or(lines.len(), |n| start + 1 + n);
        let heading = lines[start].trim_end_matches('\n');
        let (_, title) = heading.split_once("] ").ok_or("malformed heading")?;
        let (author, metadata) = title
            .split_once(" <span class=\"activity-meta\">")
            .map_or((title, None), |(a, m)| (a, Some(m)));
        if author.trim() != author || author.is_empty() {
            return Err("invalid author".into());
        }
        let legacy = lines
            .get(start + 1)
            .and_then(|s| s.trim_end_matches('\n').strip_prefix("> `"))
            .and_then(|s| s.strip_suffix('`'));
        let provenance = match (metadata, legacy) {
            (Some(meta), None) => {
                let meta = meta
                    .strip_suffix("</span>")
                    .ok_or("malformed metadata span")?;
                let (time, channel) = meta.split_once(" · ").ok_or("malformed metadata fields")?;
                format!("{time} · {author} · {channel}")
            }
            (None, Some(meta)) => meta.to_owned(),
            _ => return Err("expected exactly one legacy top stamp or #32 header".into()),
        };
        if !stamp(&provenance, author) {
            return Err(
                "invalid or ambiguous provenance; preserve source and resolve manually".into(),
            );
        }
        // Refuse further provenance candidates, even nested/fenced ones: this
        // deliberately conservative preview must never reinterpret source text.
        for line in &lines[start + 1 + usize::from(legacy.is_some())..end] {
            let line = line.trim_end_matches('\n');
            if !line.starts_with('>') || line.starts_with("> [!activity-") {
                return Err("malformed outer bubble".into());
            }
            if line.contains("activity-meta") || line.split(" · ").count() >= 3 {
                return Err("additional provenance-like source text requires manual review".into());
            }
        }
        let mut block = lines[start..end].to_vec();
        if metadata.is_some() {
            block[0] = format!("{}] {author}\n", heading.split_once("] ").unwrap().0);
        } else {
            block.remove(1);
        }
        if let Some(last) = block.last_mut()
            && !last.ends_with('\n')
        {
            last.push('\n');
        }
        if block.last().is_none_or(|s| s != ">\n") {
            block.push(">\n".into());
        }
        block.push(format!("> *{provenance}*\n"));
        lines.splice(start..end, block);
    }
    Ok(lines.concat())
}

fn preview(bytes: &[u8], expected: &str, selected: &BTreeSet<usize>) -> Result<String, String> {
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual != expected {
        return Err("preimage SHA256 mismatch; reread the current source".into());
    }
    let source = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    convert(source, selected)
}

fn run() -> Result<String, String> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        return Err("usage: activity-footer-preview SOURCE EXPECTED_SHA256 HEADING_LINE[,HEADING_LINE...] (stdout only)".into());
    }
    let bytes = fs::read(&args[1]).map_err(|e| e.to_string())?;
    let selected = args[3]
        .split(',')
        .map(|s| {
            s.parse::<usize>()
                .map_err(|_| "invalid heading line".to_owned())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    preview(&bytes, &args[2], &selected)
}

fn main() {
    match run() {
        Ok(candidate) => print!("{candidate}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_header_and_stamp_preserve_payload_and_unselected_bubbles() {
        let source = "## Activity\n\n> [!activity-message|mail] Sender <span class=\"activity-meta\">— · Mail (summary)</span>\n> [Source](https://example.org/message)\n>\n> > [!activity-more]- Meer\n> > *Literal source.*\n\n> [!activity-message|human] Alex\n> `12:30 · Alex · Chat`\n> Original source-id=example version=3\n";
        let result = convert(source, &BTreeSet::from([3, 9])).unwrap();
        assert!(result.contains("> > *Literal source.*\n>\n> *— · Sender · Mail (summary)*\n"));
        assert!(
            result.contains("> Original source-id=example version=3\n>\n> *12:30 · Alex · Chat*\n")
        );
        let partial = convert(source, &BTreeSet::from([3])).unwrap();
        assert!(
            partial.ends_with(
                source
                    .split("\n\n> [!activity-message|human]")
                    .nth(1)
                    .map(|s| format!("> [!activity-message|human]{s}"))
                    .unwrap()
                    .as_str()
            )
        );
    }
    #[test]
    fn refuses_ambiguous_invalid_or_non_live_selections() {
        let valid =
            "## Activity\n\n> [!activity-message|human] Alex\n> `— · Alex · Chat`\n> Input.\n";
        for source in [
            valid.replace("· Alex ·", "· Blake ·"),
            valid.replace("—", "24:00"),
            valid.replace("> Input.", "> *12:00 · Alex · Chat*"),
            format!("```markdown\n{valid}\n```\n"),
        ] {
            assert!(convert(&source, &BTreeSet::from([3])).is_err());
        }
        assert!(convert(valid, &BTreeSet::from([0])).is_err());
        assert!(convert(valid, &BTreeSet::from([5])).is_err());
        let quoted = "## Activity\n\n> ```markdown\n> [!activity-message|human] Alex\n> `— · Alex · Chat`\n> Literal source example.\n> ```\n";
        assert!(convert(quoted, &BTreeSet::from([4])).is_err());
        let nested_quote = "## Activity\n\n> ```markdown\n> > ```\n> [!activity-message|human] Alex\n> `— · Alex · Chat`\n> Literal source example.\n> ```\n";
        assert!(convert(nested_quote, &BTreeSet::from([5])).is_err());
        let top_fence = "## Activity\n\n```markdown\n> ```\n> [!activity-message|human] Alex\n> `— · Alex · Chat`\n> Literal source.\n\n```\n";
        assert!(convert(top_fence, &BTreeSet::from([5])).is_err());
        assert!(preview(valid.as_bytes(), "wrong-preimage", &BTreeSet::from([3])).is_err());
        let hash = format!("{:x}", Sha256::digest(valid.as_bytes()));
        assert!(preview(valid.as_bytes(), &hash, &BTreeSet::from([3])).is_ok());
    }
}
