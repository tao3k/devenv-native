//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use std::collections::BTreeSet;

pub(super) fn sorted_strings<I, J, K>(primary: I, secondary: J, tertiary: K) -> Vec<String>
where
    I: IntoIterator<Item = String>,
    J: IntoIterator<Item = String>,
    K: IntoIterator<Item = String>,
{
    let mut values = BTreeSet::new();
    values.extend(primary);
    values.extend(secondary);
    values.extend(tertiary);
    values.into_iter().collect()
}

pub(super) fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}
