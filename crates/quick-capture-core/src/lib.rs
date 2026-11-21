//! Markdown preparation for quick captures. The caller supplies the identity,
//! local clock and presentation labels, and owns validation and publication.

use serde_json::json;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy)]
pub enum CaptureKind {
    Note,
    Task,
}

/// The existing first-line title rule, including an empty first line.
pub fn capture_title(input: &str) -> String {
    input
        .lines()
        .next()
        .unwrap_or(input)
        .trim()
        .chars()
        .take(100)
        .collect()
}

/// The stem is relative to a destination selected by the caller. The digest
/// uses the original request identity, including hexadecimal letter case.
pub fn capture_stem(date: &str, request_id: &str, title: &str) -> String {
    let mut slug = String::new();
    for character in title.chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() {
            slug.push(character);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
        if slug.chars().count() >= 48 {
            break;
        }
    }
    let slug = slug.trim_end_matches('-');
    let slug = if slug.is_empty() { "notitie" } else { slug };
    let request_hash = format!("{:x}", Sha256::digest(request_id.as_bytes()));
    format!("{}-{slug}-{}", date.replace('-', ""), &request_hash[..20])
}

/// Make a new inbox capture header. The caller supplies its validated ID,
/// date and time, and configured single-line YAML tokens for source/category.
pub fn capture_header(
    kind: CaptureKind,
    title: &str,
    date: &str,
    time: &str,
    request_id: &str,
    source: &str,
    task_category: &str,
) -> String {
    let metadata = match kind {
        CaptureKind::Task => format!(
            "type: task\ncategory: {task_category}\nstatus: \"🔴 to-do\"\ntask-id: {}\n",
            json!(format!("task:{request_id}"))
        ),
        CaptureKind::Note => "type: note\ncategory: capture\n".into(),
    };
    format!(
        "---\n{metadata}name: {}\nwiki_status: pending\ncreated: {date}\nevent_date: {date}\ncssclasses: [vault-navigation, vault-dashboard, vault-event]\ncaptured: \"{date}T{time}\"\ncapture: {source}\ncapture-id: {}\n---\n\n# {title}\n",
        json!(title),
        json!(request_id)
    )
}

/// Evaluate the folding threshold against the original bytes and lines,
/// before newline normalization.
pub fn should_fold(input: &str) -> bool {
    input.len() > 1200 || input.lines().count() > 30
}

/// Render one human message after the caller's Activity heading and day
/// selection. Author, channel, time and fold label are single-line labels.
pub fn human_activity_message(
    input: &str,
    time: &str,
    author: &str,
    channel: &str,
    fold_label: Option<&str>,
    verbatim: bool,
) -> String {
    let mut output = format!("\n\n> [!activity-message|human] {author}\n");
    let prefix = if let Some(label) = fold_label {
        output.push_str(&format!("> > [!activity-more]- {label}\n"));
        "> > "
    } else {
        "> "
    };
    let normalized;
    let text = if verbatim {
        input
    } else {
        normalized = input.replace("\r\n", "\n").replace('\r', "\n");
        &normalized
    };
    for line in text.split_inclusive('\n') {
        output.push_str(prefix);
        output.push_str(line);
    }
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(&format!(">\n> *{time} · {author} · {channel}*\n"));
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "10000000-0000-4000-8000-000000000123";
    const DATE: &str = "2030-01-02";

    #[test]
    fn title_and_stem_keep_existing_unicode_limits_and_fallback() {
        assert_eq!(
            capture_title("  Één: \"idee\"  \r\n# body"),
            "Één: \"idee\""
        );
        assert_eq!(capture_title("\nbody"), "");
        assert_eq!(capture_title(&"界".repeat(102)), "界".repeat(100));
        assert_eq!(
            capture_stem(DATE, ID, "ÉÉN: idee!?"),
            "20300102-één-idee-38903a51b038d0276b07"
        );
        assert_eq!(
            capture_stem(DATE, ID, "..."),
            "20300102-notitie-38903a51b038d0276b07"
        );
        assert_eq!(
            capture_stem(DATE, ID, &"界".repeat(50)),
            format!("20300102-{}-38903a51b038d0276b07", "界".repeat(48))
        );
    }

    #[test]
    fn note_and_task_headers_preserve_exact_metadata() {
        let note_header = capture_header(
            CaptureKind::Note,
            "Een \"idee\"",
            DATE,
            "12:34:56",
            ID,
            "quick-example",
            "work-task",
        );
        assert_eq!(
            note_header,
            concat!(
                "---\ntype: note\ncategory: capture\nname: \"Een \\\"idee\\\"\"\n",
                "wiki_status: pending\ncreated: 2030-01-02\nevent_date: 2030-01-02\n",
                "cssclasses: [vault-navigation, vault-dashboard, vault-event]\n",
                "captured: \"2030-01-02T12:34:56\"\ncapture: quick-example\n",
                "capture-id: \"10000000-0000-4000-8000-000000000123\"\n---\n\n# Een \"idee\"\n"
            )
        );
        let task_header = capture_header(
            CaptureKind::Task,
            "Een \"idee\"",
            DATE,
            "12:34:56",
            ID,
            "quick-example",
            "work-task",
        );
        assert_eq!(
            task_header,
            note_header.replacen(
                "type: note\ncategory: capture\n",
                concat!(
                    "type: task\ncategory: work-task\nstatus: \"🔴 to-do\"\n",
                    "task-id: \"task:10000000-0000-4000-8000-000000000123\"\n"
                ),
                1
            )
        );
    }

    #[test]
    fn folds_at_the_existing_byte_and_line_thresholds() {
        assert!(!should_fold(&"é".repeat(600)));
        assert!(should_fold(&"é".repeat(601)));
        assert!(!should_fold(&"x\n".repeat(30)));
        assert!(should_fold(&"x\n".repeat(31)));
        assert!(!should_fold(&"x\r".repeat(31)));
    }

    #[test]
    fn activity_keeps_literal_quoting_and_trailing_whitespace() {
        assert_eq!(
            human_activity_message(
                "Één\r\n\r# Kop  \n",
                "12:34",
                "Example",
                "Launcher",
                None,
                false
            ),
            "\n\n> [!activity-message|human] Example\n> Één\n> \n> # Kop  \n>\n> *12:34 · Example · Launcher*\n"
        );
        assert_eq!(
            human_activity_message(
                "> detail\r\nlaatste",
                "12:34",
                "Example",
                "Launcher",
                Some("Meer"),
                true
            ),
            "\n\n> [!activity-message|human] Example\n> > [!activity-more]- Meer\n> > > detail\r\n> > laatste\n>\n> *12:34 · Example · Launcher*\n"
        );
    }
}
