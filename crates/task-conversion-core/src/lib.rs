//! Prepare task properties while retaining the caller's frontmatter and body.
//! The caller establishes eligibility, identity, allocation and Activity.

use serde_json::json;
pub use task_status_core::is_status_label;

pub const DEFAULT_STATUS: &str = "🔴 to-do";

/// Split a complete LF frontmatter block, leaving all body bytes untouched.
/// The local writer decides whether a missing block or an unsupported header is allowed.
pub fn split_frontmatter(source: &str) -> Option<(Vec<String>, &str)> {
    let tail = source.strip_prefix("---\n")?;
    if let Some(body) = tail.strip_prefix("---\n") {
        return Some((Vec::new(), body));
    }
    let (head, body) = tail.split_once("\n---\n")?;
    Some((head.lines().map(str::to_owned).collect(), body))
}

/// Replace a property at the end, preserving the order and spelling of other lines.
pub fn set_scalar(fields: &mut Vec<String>, key: &str, value: &str) {
    let prefix = format!("{key}:");
    fields.retain(|line| !line.starts_with(&prefix));
    fields.push(format!("{key}: {}", json!(value)));
}

pub fn promote(fields: &mut Vec<String>, task_id: &str, status: &str) {
    set_scalar(fields, "type", "task");
    set_scalar(fields, "task-id", task_id);
    set_scalar(fields, "status", status);
}

pub fn document(fields: &[String], body: &str) -> String {
    format!("---\n{}\n---\n{body}", fields.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_exact_source_bytes_and_property_order() {
        let source = "---\ntype: note\n# retain\ncategory: capture\ndue: '2026-12-01'\nstatus: old\n---\n# Café 🧪\r\n\r\n[[source]]";
        let (mut fields, body) = split_frontmatter(source).unwrap();
        promote(&mut fields, "task:fixture", "🔵 waiting");
        assert_eq!(
            document(&fields, body),
            "---\n# retain\ncategory: capture\ndue: '2026-12-01'\ntype: \"task\"\ntask-id: \"task:fixture\"\nstatus: \"🔵 waiting\"\n---\n# Café 🧪\r\n\r\n[[source]]"
        );
        assert!(is_status_label(DEFAULT_STATUS));
    }

    #[test]
    fn distinguishes_an_empty_header_from_an_incomplete_header() {
        assert_eq!(
            split_frontmatter("---\n---\nplain\r\n"),
            Some((vec![], "plain\r\n"))
        );
        assert_eq!(split_frontmatter("---\ntype: note\n"), None);
        assert_eq!(split_frontmatter("---\r\n---\r\nplain\r\n"), None);
        assert_eq!(split_frontmatter("plain\r\n"), None);
    }
}
