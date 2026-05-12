//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use chrono::{DateTime, NaiveDate, TimeZone, Utc};

#[derive(Clone, Copy)]
enum TimeFilterSlot {
    CreatedAfter,
    CreatedBefore,
    ModifiedAfter,
    ModifiedBefore,
}

pub(in crate::parsers::link_graph::query) fn parse_timestamp(raw: &str) -> Option<i64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(epoch) = trimmed.parse::<i64>() {
        return Some(epoch);
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(dt.timestamp());
    }
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return date
            .and_hms_opt(0, 0, 0)
            .map(|naive| Utc.from_utc_datetime(&naive).timestamp());
    }
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y/%m/%d") {
        return date
            .and_hms_opt(0, 0, 0)
            .map(|naive| Utc.from_utc_datetime(&naive).timestamp());
    }
    None
}

pub(in crate::parsers::link_graph::query) fn parse_time_filter(
    token: &str,
    created_after: &mut Option<i64>,
    created_before: &mut Option<i64>,
    modified_after: &mut Option<i64>,
    modified_before: &mut Option<i64>,
) -> bool {
    let lower = token.trim().to_lowercase();
    let rules = [
        ("created>=", TimeFilterSlot::CreatedAfter),
        ("created<=", TimeFilterSlot::CreatedBefore),
        ("created>", TimeFilterSlot::CreatedAfter),
        ("created<", TimeFilterSlot::CreatedBefore),
        ("modified>=", TimeFilterSlot::ModifiedAfter),
        ("modified<=", TimeFilterSlot::ModifiedBefore),
        ("modified>", TimeFilterSlot::ModifiedAfter),
        ("modified<", TimeFilterSlot::ModifiedBefore),
        ("updated>=", TimeFilterSlot::ModifiedAfter),
        ("updated<=", TimeFilterSlot::ModifiedBefore),
        ("updated>", TimeFilterSlot::ModifiedAfter),
        ("updated<", TimeFilterSlot::ModifiedBefore),
    ];
    let Some(&(prefix, slot)) = rules.iter().find(|(prefix, _)| lower.starts_with(prefix)) else {
        return false;
    };
    let value = token[prefix.len()..].trim().trim_start_matches(':');
    let Some(parsed) = parse_timestamp(value) else {
        return false;
    };
    match slot {
        TimeFilterSlot::CreatedAfter => *created_after = Some(parsed),
        TimeFilterSlot::CreatedBefore => *created_before = Some(parsed),
        TimeFilterSlot::ModifiedAfter => *modified_after = Some(parsed),
        TimeFilterSlot::ModifiedBefore => *modified_before = Some(parsed),
    }
    true
}
