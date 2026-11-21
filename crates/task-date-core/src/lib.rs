//! Pure calendar preparation. The caller supplies its local day and policy.

fn month_length(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        _ => 0,
    }
}

fn parts(value: &str, minimum_year: u32) -> Option<(u32, u32, u32)> {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year = value[..4].parse::<u32>().ok()?;
    let month = value[5..7].parse::<u32>().ok()?;
    let day = value[8..].parse::<u32>().ok()?;
    (year >= minimum_year && (1..=month_length(year, month)).contains(&day))
        .then_some((year, month, day))
}

/// Validate the fixed-width date shape and numeric behavior of the task writers.
pub fn is_iso_date(value: &str, minimum_year: u32) -> bool {
    parts(value, minimum_year).is_some()
}

fn following_day(today: &str, minimum_year: u32) -> Option<String> {
    let (mut year, mut month, mut day) = parts(today, minimum_year)?;
    day += 1;
    if day > month_length(year, month) {
        day = 1;
        month += 1;
        if month == 13 {
            month = 1;
            year += 1;
        }
    }
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

/// Resolve a relative value against the supplied local day. A caller enables
/// this only for eligible actions, validates the result and retains the raw
/// input for replay. An invalid base day prevents resolving tomorrow.
pub fn resolve_input(input: &str, today: &str, minimum_year: u32) -> Option<String> {
    match input {
        "today" => Some(today.to_owned()),
        "tomorrow" => following_day(today, minimum_year),
        _ => Some(input.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_calendar_and_retains_both_year_policies() {
        for value in ["2024-02-29", "2000-02-29", "2026-+1-+1"] {
            assert!(is_iso_date(value, 1900));
        }
        for value in [
            "1900-02-29",
            "2100-02-29",
            "2026-04-31",
            "2026-00-01",
            "2026-01-00",
            "2026-01-é",
            "10000-01-01",
        ] {
            assert!(!is_iso_date(value, 1));
        }
        assert!(is_iso_date("0001-01-01", 1));
        assert!(!is_iso_date("0001-01-01", 1900));
        assert!(!is_iso_date("0000-01-01", 1));
    }

    #[test]
    fn resolves_local_aliases_and_preserves_literal_requests() {
        for (today, tomorrow) in [
            ("2024-02-28", "2024-02-29"),
            ("2024-02-29", "2024-03-01"),
            ("2100-02-28", "2100-03-01"),
            ("2026-12-31", "2027-01-01"),
            ("9999-12-31", "10000-01-01"),
        ] {
            assert_eq!(resolve_input("today", today, 1900).as_deref(), Some(today));
            assert_eq!(
                resolve_input("tomorrow", today, 1900).as_deref(),
                Some(tomorrow)
            );
        }
        assert_eq!(resolve_input("tomorrow", "1899-12-31", 1900), None);
        for input in ["clear", "2026-+1-01", " invalid "] {
            assert_eq!(resolve_input(input, "unused", 1900).as_deref(), Some(input));
        }
    }
}
