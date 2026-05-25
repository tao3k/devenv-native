//! Internal bounded DMN duration helpers.

use std::cmp::Ordering;

/// Supported bounded ISO 8601 duration families.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DmnDurationFamily {
    /// Day-time duration measured in total nanoseconds.
    DayTime,
    /// Year-month duration measured in total months.
    YearMonth,
}

/// One normalized bounded duration value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DmnDurationValue {
    /// Day-time duration in total nanoseconds.
    DayTime(i128),
    /// Year-month duration in total months.
    YearMonth(i128),
}

impl DmnDurationValue {
    /// Creates one normalized day-time duration value from total nanoseconds.
    #[must_use]
    pub(crate) fn from_total_day_time_nanos(total_nanoseconds: i128) -> Self {
        Self::DayTime(total_nanoseconds)
    }

    /// Creates one normalized year-month duration value from total months.
    #[must_use]
    pub(crate) fn from_total_months(total_months: i128) -> Self {
        Self::YearMonth(total_months)
    }

    /// Returns the bounded duration family.
    #[must_use]
    pub(crate) fn family(self) -> DmnDurationFamily {
        match self {
            Self::DayTime(_) => DmnDurationFamily::DayTime,
            Self::YearMonth(_) => DmnDurationFamily::YearMonth,
        }
    }

    /// Compares two duration values when they belong to the same supported
    /// family.
    #[must_use]
    pub(crate) fn compare(self, other: Self) -> Option<Ordering> {
        match (self, other) {
            (Self::DayTime(left), Self::DayTime(right))
            | (Self::YearMonth(left), Self::YearMonth(right)) => Some(left.cmp(&right)),
            _ => None,
        }
    }
}

/// Parses one bounded ISO 8601 day-time duration.
///
/// Supported subset:
/// - optional leading `-`
/// - `P<n>D`
/// - `P<n>.<fraction>D` or `P<n>,<fraction>D` with up to 9 fractional digits
///   when no time units follow
/// - `PT<n>H`
/// - `PT<n>.<fraction>H` or `PT<n>,<fraction>H` with up to 9 fractional digits
///   when no lower-order units follow
/// - `PT<n>M`
/// - `PT<n>.<fraction>M` or `PT<n>,<fraction>M` with up to 9 fractional digits
///   when no lower-order units follow
/// - `PT<n>S`
/// - `PT<n>.<fraction>S` or `PT<n>,<fraction>S` with up to 9 fractional digits
/// - ordered combinations of `D`, `H`, `M`, and `S`
///
/// Deferred:
/// - year/month duration components (`Y`, date-part `M`)
/// - lower-order units after one fractional day-time component
/// - week notation
pub(crate) fn parse_day_time_duration_str(raw: &str) -> Option<DmnDurationValue> {
    let (sign, value) = parse_duration_sign_prefix(raw)?;
    if value.is_empty() {
        return None;
    }

    let (date_part, time_part) = match value.split_once('T') {
        Some((date, time)) => (date, Some(time)),
        None => (value, None),
    };

    let mut component_count = 0_u8;
    let mut total_nanoseconds = 0_i128;

    if !date_part.is_empty() {
        let days_raw = date_part.strip_suffix('D')?;
        let (days, is_fractional) = parse_day_time_component(days_raw, NANOS_PER_DAY)?;
        if is_fractional && time_part.is_some() {
            return None;
        }
        total_nanoseconds = total_nanoseconds.checked_add(days)?;
        component_count += 1;
    }

    if let Some(time_part) = time_part {
        if time_part.is_empty() {
            return None;
        }
        let mut rest = time_part;
        let mut last_unit_rank = 0_u8;
        while !rest.is_empty() {
            let unit_index = rest.find(char::is_alphabetic)?;
            if unit_index == 0 {
                return None;
            }
            let value_raw = &rest[..unit_index];
            let unit = rest[unit_index..].chars().next()?;
            let next_rest = &rest[unit_index + unit.len_utf8()..];
            let (unit_rank, value, is_fractional) = match unit {
                'H' => {
                    let (value, is_fractional) =
                        parse_day_time_component(value_raw, NANOS_PER_HOUR)?;
                    (1_u8, value, is_fractional)
                }
                'M' => {
                    let (value, is_fractional) =
                        parse_day_time_component(value_raw, NANOS_PER_MINUTE)?;
                    (2_u8, value, is_fractional)
                }
                'S' => {
                    let (value, is_fractional) =
                        parse_day_time_component(value_raw, NANOS_PER_SECOND)?;
                    (3_u8, value, is_fractional)
                }
                _ => {
                    return None;
                }
            };
            if unit_rank <= last_unit_rank {
                return None;
            }
            if is_fractional && !next_rest.is_empty() {
                return None;
            }
            last_unit_rank = unit_rank;
            total_nanoseconds = total_nanoseconds.checked_add(value)?;
            component_count += 1;
            rest = next_rest;
        }
    }

    if component_count == 0 {
        return None;
    }

    Some(DmnDurationValue::from_total_day_time_nanos(
        apply_duration_sign(total_nanoseconds, sign)?,
    ))
}

/// Parses one bounded ISO 8601 year-month duration.
///
/// Supported subset:
/// - optional leading `-`
/// - `P<n>Y`
/// - `P<n>M`
/// - ordered combinations of `Y` and date-part `M`
///
/// Deferred:
/// - fractional duration components
/// - mixed year-month/day-time durations
/// - week notation
pub(crate) fn parse_year_month_duration_str(raw: &str) -> Option<DmnDurationValue> {
    let (sign, value) = parse_duration_sign_prefix(raw)?;
    if value.is_empty() || value.contains('T') {
        return None;
    }

    let mut rest = value;
    let mut last_unit_rank = 0_u8;
    let mut component_count = 0_u8;
    let mut total_months = 0_i128;

    while !rest.is_empty() {
        let digit_count = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digit_count == 0 {
            return None;
        }
        let value_raw = &rest[..digit_count];
        let value = parse_component_number(value_raw)?;
        rest = &rest[digit_count..];
        let unit = rest.chars().next()?;
        let (unit_rank, multiplier) = match unit {
            'Y' => (1_u8, 12_i128),
            'M' => (2_u8, 1_i128),
            _ => return None,
        };
        if unit_rank <= last_unit_rank {
            return None;
        }
        last_unit_rank = unit_rank;
        total_months = checked_add_units(total_months, value, multiplier)?;
        component_count += 1;
        rest = &rest[unit.len_utf8()..];
    }

    if component_count == 0 {
        return None;
    }

    Some(DmnDurationValue::from_total_months(apply_duration_sign(
        total_months,
        sign,
    )?))
}

const NANOS_PER_SECOND: i128 = 1_000_000_000;
const NANOS_PER_MINUTE: i128 = 60 * NANOS_PER_SECOND;
const NANOS_PER_HOUR: i128 = 60 * NANOS_PER_MINUTE;
const NANOS_PER_DAY: i128 = 24 * NANOS_PER_HOUR;

fn parse_duration_sign_prefix(raw: &str) -> Option<(i128, &str)> {
    if let Some(value) = raw.strip_prefix("-P") {
        return Some((-1, value));
    }
    raw.strip_prefix('P').map(|value| (1, value))
}

fn apply_duration_sign(value: i128, sign: i128) -> Option<i128> {
    match sign {
        1 => Some(value),
        -1 => value.checked_neg(),
        _ => None,
    }
}

fn parse_component_number(raw: &str) -> Option<i128> {
    if raw.is_empty() || !raw.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    raw.parse::<i128>().ok()
}

fn parse_day_time_component(raw: &str, unit_nanoseconds: i128) -> Option<(i128, bool)> {
    if raw.is_empty() {
        return None;
    }
    let dot_count = raw.matches('.').count();
    let comma_count = raw.matches(',').count();
    if dot_count + comma_count > 1 || (dot_count > 0 && comma_count > 0) {
        return None;
    }
    let separator = if dot_count == 1 {
        Some('.')
    } else if comma_count == 1 {
        Some(',')
    } else {
        None
    };
    let (whole_raw, fractional_raw) = match separator {
        Some(separator) => {
            let (whole, fractional) = raw.split_once(separator)?;
            (whole, Some(fractional))
        }
        None => (raw, None),
    };
    let whole_units = parse_component_number(whole_raw)?;
    let mut total_nanoseconds = whole_units.checked_mul(unit_nanoseconds)?;
    let Some(fractional_raw) = fractional_raw else {
        return Some((total_nanoseconds, false));
    };
    if fractional_raw.is_empty()
        || fractional_raw.len() > 9
        || !fractional_raw.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }
    let fractional = fractional_raw.parse::<i128>().ok()?;
    let divisor = 10_i128.checked_pow(u32::try_from(fractional_raw.len()).ok()?)?;
    let fractional_unit_nanos = unit_nanoseconds.checked_div(divisor)?;
    if unit_nanoseconds.checked_rem(divisor)? != 0 {
        return None;
    }
    total_nanoseconds =
        total_nanoseconds.checked_add(fractional.checked_mul(fractional_unit_nanos)?)?;
    Some((total_nanoseconds, true))
}

fn checked_add_units(acc: i128, value: i128, multiplier: i128) -> Option<i128> {
    acc.checked_add(value.checked_mul(multiplier)?)
}
