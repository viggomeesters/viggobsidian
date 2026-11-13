// Identical quote handling to the pre-extraction native helper.
fn quote_content(mut line: &str) -> &str {
    loop {
        let trimmed = line.trim_start_matches(' ');
        if line.len() - trimmed.len() > 3 {
            return line;
        }
        let Some(rest) = trimmed.strip_prefix('>') else {
            return line;
        };
        line = rest.strip_prefix(' ').unwrap_or(rest);
    }
}

pub fn anchor_count(markdown: &str, anchor: &str) -> usize {
    let mut lines = markdown.lines().peekable();
    if lines.peek() == Some(&"---") {
        lines.next();
        for line in lines.by_ref() {
            if line == "---" {
                break;
            }
        }
    }
    let headings: Vec<_> = anchor.split('#').map(decoded_heading).collect();
    let mut stack = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut comment = false;
    let mut previous: Option<String> = None;
    let mut count = 0;
    for line in lines {
        let line = quote_content(line).trim_end();
        let trimmed = line.trim_start_matches(' ');
        if line.len() - trimmed.len() <= 3
            && let Some(marker @ (b'`' | b'~')) = trimmed.bytes().next()
        {
            let length = trimmed.bytes().take_while(|&byte| byte == marker).count();
            if let Some((opening, minimum)) = fence {
                if marker == opening && length >= minimum && trimmed[length..].trim().is_empty() {
                    fence = None;
                }
                continue;
            }
            if !comment && length >= 3 {
                fence = Some((marker, length));
                previous = None;
                continue;
            }
        }
        if fence.is_some() {
            continue;
        }
        let mut rest = line;
        let mut visible = String::new();
        loop {
            if comment {
                let Some((_, after)) = rest.split_once("-->") else {
                    break;
                };
                comment = false;
                rest = after;
            } else if let Some((before, after)) = rest.split_once("<!--") {
                visible.push_str(before);
                comment = true;
                rest = after;
            } else {
                visible.push_str(rest);
                break;
            }
        }
        if let Some(block) = anchor.strip_prefix('^') {
            count += usize::from(
                !block.is_empty()
                    && visible
                        .split_whitespace()
                        .last()
                        .is_some_and(|word| word == anchor),
            );
            continue;
        }
        let trimmed = visible.trim_start_matches(' ');
        let prior = previous.take();
        let Some((level, title)) =
            heading(trimmed, prior.as_deref()).filter(|_| visible.len() - trimmed.len() <= 3)
        else {
            if !trimmed.trim().is_empty() && visible.len() - trimmed.len() <= 3 {
                previous = Some(trimmed.trim().to_owned());
            }
            continue;
        };
        stack.truncate(
            stack
                .iter()
                .position(|(old, _)| *old >= level)
                .unwrap_or(stack.len()),
        );
        stack.push((level, heading_text(title)));
        if !title.is_empty()
            && stack.len() >= headings.len()
            && stack[stack.len() - headings.len()..]
                .iter()
                .map(|(_, title)| title.as_str())
                .eq(headings.iter().map(String::as_str))
        {
            count += 1;
        }
    }
    count
}

pub fn heading<'a>(line: &'a str, previous: Option<&'a str>) -> Option<(usize, &'a str)> {
    let level = line.bytes().take_while(|&byte| byte == b'#').count();
    if (1..=6).contains(&level) && line[level..].starts_with([' ', '\t']) {
        let title = line[level..].trim();
        let tail = title.trim_end_matches('#');
        return Some((
            level,
            if tail.ends_with([' ', '\t']) {
                tail.trim_end()
            } else {
                title
            },
        ));
    }
    let marker = line.as_bytes().first()?;
    if !line.bytes().all(|byte| byte == *marker) {
        return None;
    }
    let level = match marker {
        b'=' => 1,
        b'-' => 2,
        _ => return None,
    };
    previous.map(|title| (level, title))
}

fn decoded_heading(heading: &str) -> String {
    decode_percent(heading).trim().to_owned()
}

pub fn decode_percent(heading: &str) -> String {
    let mut bytes = heading.as_bytes().iter().copied().peekable();
    let mut decoded = Vec::with_capacity(heading.len());
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let mut lookahead = bytes.clone();
            let hex = |byte: u8| (byte as char).to_digit(16);
            if let (Some(high), Some(low)) = (
                lookahead.next().and_then(hex),
                lookahead.next().and_then(hex),
            ) {
                decoded.push((high * 16 + low) as u8);
                bytes = lookahead;
                continue;
            }
        }
        decoded.push(byte);
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn heading_text(title: &str) -> String {
    let mut title = title.replace('`', "");
    for marker in ["**", "__", "~~", "*", "_"] {
        let mut offset = 0;
        while let Some(start) = title[offset..].find(marker) {
            let start = offset + start;
            let content = start + marker.len();
            let Some(end) = title[content..].find(marker) else {
                break;
            };
            let end = content + end;
            title.replace_range(end..end + marker.len(), "");
            title.replace_range(start..content, "");
            offset = end - marker.len();
        }
    }
    title
}

#[cfg(test)]
mod tests {
    use super::anchor_count;

    #[test]
    fn counts_live_headings_and_rejects_examples() {
        let note = "---\n# YAML\n---\n# Parent\n## Child ##\n<!--\n## Hidden\n-->\n```md\n## Example\n```\n# Other\n## Child\n";
        assert_eq!(anchor_count(note, "Parent#Child"), 1);
        assert_eq!(anchor_count(note, "Child"), 2);
        assert_eq!(anchor_count("## **Mijn kop**", "Mijn%20kop"), 1);
        assert_eq!(
            anchor_count("Intro\n=====\nDetails\n-------", "Intro#Details"),
            1
        );
        for absent in ["YAML", "Hidden", "Example", "Missing", ""] {
            assert_eq!(anchor_count(note, absent), 0);
        }
    }

    #[test]
    fn percent_unicode_and_duplicate_targets_keep_legacy_meaning() {
        let note = "# Café\n## 東京\n# Other\n## 東京\n## `%` and **bold**\n";
        assert_eq!(anchor_count(note, "Caf%C3%A9#%E6%9D%B1%E4%BA%AC"), 1);
        assert_eq!(anchor_count(note, "%E6%9D%B1%E4%BA%AC"), 2);
        assert_eq!(anchor_count(note, "%25%20and%20bold"), 1);
        assert_eq!(anchor_count("## percent%GG", "percent%GG"), 1);
        assert_eq!(anchor_count("## Café", "Caf%E9"), 0);
    }
}
