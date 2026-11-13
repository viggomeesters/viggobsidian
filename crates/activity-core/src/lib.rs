//! Canonical Activity text checks. No identity, filesystem or execution claims.
pub mod activity_delivery;
pub mod activity_layout;
pub mod mail_draft;
mod snapshot;
pub use snapshot::{MAX_NOTE_BYTES, MAX_REQUEST_BYTES, respond};
#[cfg(target_arch = "wasm32")]
mod wasm;
mod index_navigation {
    pub use link_style_core::{quote_content, visible_body};
}

fn iso_date(date: &str) -> bool {
    let parts: Vec<_> = date.split('-').collect();
    matches!(parts.as_slice(), [year, month, day] if year.len() == 4 && month.len() == 2 && day.len() == 2 && valid_date(year, month, day))
}
fn valid_date(year: &str, month: &str, day: &str) -> bool {
    if [year, month, day]
        .iter()
        .any(|part| !part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        year.parse::<u32>(),
        month.parse::<u32>(),
        day.parse::<u32>(),
    ) else {
        return false;
    };
    let max = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        _ => 0,
    };
    year > 0 && (1..=max).contains(&day)
}

#[cfg(test)]
mod frozen {
    use super::*;
    #[test]
    fn original_text_outputs_are_unchanged() {
        let corpus: serde_json::Value =
            serde_json::from_str(include_str!("../fixtures/activity-legacy.json")).unwrap();
        for case in corpus["cases"].as_array().unwrap() {
            let body = case["body"].as_str().unwrap();
            for (key, actual) in [
                ("chat", serde_json::json!(activity_layout::findings(body))),
                (
                    "provenance",
                    serde_json::json!(activity_delivery::provenance_findings(body)),
                ),
                (
                    "delivery",
                    serde_json::json!(activity_delivery::findings(body)),
                ),
                ("mail", serde_json::json!(mail_draft::findings(body))),
                (
                    "event",
                    serde_json::json!(activity_layout::check_v11(body).err()),
                ),
                (
                    "raw_task",
                    serde_json::json!(activity_layout::check_task_event(body).err()),
                ),
            ] {
                assert_eq!(actual, case[key], "{} {key}", case["id"]);
            }
        }
    }
}
