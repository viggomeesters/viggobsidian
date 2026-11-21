//! Pure Task from Note request preparation.

use std::{collections::BTreeSet, path::Path};
use task_status_core::literal_text;

/// Map a UTF-16 index to a UTF-8 byte boundary.
pub fn utf16_offset(text: &str, offset: u64) -> Option<usize> {
    let mut units = 0;
    for (byte, ch) in text.char_indices() {
        if units == offset {
            return Some(byte);
        }
        units += ch.len_utf16() as u64;
        if units > offset {
            return None;
        }
    }
    (units == offset).then_some(text.len())
}

/// The first top-level H1 after properties, or the lexical filename stem.
pub fn note_title(source: &str, path: &str) -> Result<String, &'static str> {
    use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
    let mut title = String::new();
    let mut heading = false;
    let mut container = 0;
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let body = if source.starts_with("---\n") || source.starts_with("---\r\n") {
        let mut end = 0;
        let mut found = false;
        for (index, line) in source.split_inclusive('\n').enumerate() {
            end += line.len();
            if index > 0 && line.trim_end_matches(['\r', '\n']) == "---" {
                found = true;
                break;
            }
        }
        if !found {
            return Err("Sluit de properties voordat je een taak bij de notitie maakt");
        }
        &source[end..]
    } else {
        source
    };
    for event in Parser::new(body) {
        match event {
            Event::Start(Tag::BlockQuote(_) | Tag::Item) => container += 1,
            Event::End(TagEnd::BlockQuote(_) | TagEnd::Item) => container -= 1,
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) if container == 0 => heading = true,
            Event::End(TagEnd::Heading(HeadingLevel::H1)) if heading => break,
            Event::Text(text) | Event::Code(text) if heading => title.push_str(&text),
            Event::SoftBreak | Event::HardBreak if heading => title.push(' '),
            _ => {}
        }
    }
    if title.trim().is_empty() {
        title = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Notitietitel ontbreekt")?
            .to_owned();
    }
    let title = title.trim().chars().take(100).collect::<String>();
    if title.is_empty() || title.contains('\0') {
        return Err("Notitietitel ontbreekt");
    }
    Ok(title)
}

/// The first nonblank line of a selection, with the existing scalar limit.
pub fn selection_title(selected: &str) -> Result<String, &'static str> {
    let first = selected
        .lines()
        .find(|line| !line.trim().is_empty())
        .ok_or("Selecteer eerst tekst")?;
    Ok(first.trim().chars().take(100).collect())
}

/// Make the title safe as literal Markdown while retaining its existing prefix.
pub fn display_title(title: &str) -> String {
    let prefixed;
    let title = if title.chars().any(|c| c.is_ascii_alphanumeric()) {
        title
    } else {
        prefixed = format!("Taak · {title}");
        &prefixed
    };
    literal_text(title)
        .replace('~', "\\~")
        .replace('%', "\\%")
        .replace('$', "\\$")
}

/// Render an already escaped title with the caller's resolved source link.
pub fn note_request(title: &str, source_link: &str) -> String {
    format!("{title}\n\nBron: {source_link}\n")
}

/// Retain the exact selection bytes and the first occurrence of each
/// caller-resolved attachment reference.
pub fn selection_request(
    title: &str,
    source_link: &str,
    selected: &str,
    links: &[String],
) -> String {
    let width = selected
        .split(|c| c != '`')
        .map(str::len)
        .max()
        .unwrap_or(0)
        .max(2)
        + 1;
    let fence = "`".repeat(width);
    let mut body = note_request(title, source_link);
    body.push_str(&format!("\n{fence}text\n{selected}\n{fence}\n"));
    let mut seen = BTreeSet::new();
    for resolved in links {
        if seen.insert(resolved.as_str()) {
            body.push('\n');
            body.push_str(resolved);
            body.push('\n');
        }
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_offsets_reject_surrogate_splits_and_out_of_range_positions() {
        let text = "A🌍é\r\n";
        let expected = [
            Some(0),
            Some(1),
            None,
            Some(5),
            Some(7),
            Some(8),
            Some(9),
            None,
        ];
        for (offset, byte) in expected.into_iter().enumerate() {
            assert_eq!(utf16_offset(text, offset as u64), byte);
        }
    }

    #[test]
    fn note_title_ignores_metadata_quotes_lists_and_code() {
        let source = "\u{feff}---\r\nname: Fictional metadata\r\n---\r\n> # Quote\r\n\r\n- # List\r\n\r\n```text\r\n# Code\r\n```\r\n\r\n# Réel `item` 🌍\r\n\r\n# Later\r\n";
        assert_eq!(note_title(source, "named.md"), Ok("Réel item 🌍".into()));
        assert_eq!(note_title("Body only", "test/file.md"), Ok("file".into()));
        assert_eq!(
            note_request(&display_title("Réel item 🌍"), "[[Named source]]"),
            "Réel item 🌍\n\nBron: [[Named source]]\n"
        );
        assert!(note_title("---\nmissing: end\n", "fallback.md").is_err());
    }

    #[test]
    fn literal_selection_keeps_bytes_and_distinct_links_in_encounter_order() {
        let selected = "\r\n  ## Fictional `heading` [$~%] &  \r\n`````\r\nLast\r";
        let title = display_title(&selection_title(selected).unwrap());
        assert_eq!(title, r"\#\# Fictional \`heading\` \[\$\~\%\] &amp;");
        let links = [
            "[[z/file.pdf|file.pdf]]",
            "[[a.md#^part|a.md]]",
            "[[z/file.pdf|file.pdf]]",
        ]
        .map(str::to_owned);
        let request = selection_request(&title, "[[Original source]]", selected, &links);
        assert_eq!(
            request,
            format!(
                "{title}\n\nBron: [[Original source]]\n\n``````text\n{selected}\n``````\n\n[[z/file.pdf|file.pdf]]\n\n[[a.md#^part|a.md]]\n"
            )
        );
        assert_eq!(display_title("🚀"), "Taak · 🚀");
        let long = "界".repeat(101);
        assert_eq!(selection_title(&long).unwrap().chars().count(), 100);
        assert!(selection_title("\r\n   ").is_err());
    }
}
