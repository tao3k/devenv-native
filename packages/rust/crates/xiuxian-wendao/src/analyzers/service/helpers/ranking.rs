//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
pub(crate) fn normalized_rank_score(raw_score: u8, worst_bucket: u8) -> f64 {
    let denominator = f64::from(worst_bucket.saturating_add(1));
    let numerator = f64::from(worst_bucket.saturating_add(1).saturating_sub(raw_score));
    (numerator / denominator).clamp(0.0, 1.0)
}

pub(crate) fn module_match_score(query: &str, qualified_name: &str, path: &str) -> Option<u8> {
    if query.is_empty() {
        return Some(0);
    }
    if qualified_name == query {
        return Some(0);
    }
    if qualified_name.starts_with(query) {
        return Some(1);
    }
    if qualified_name.contains(query) {
        return Some(2);
    }
    if path.contains(query) {
        return Some(3);
    }
    None
}

pub(crate) fn symbol_match_score(
    query: &str,
    name: &str,
    qualified_name: &str,
    path: &str,
    signature: &str,
) -> Option<u8> {
    if query.is_empty() {
        return Some(0);
    }
    if name == query {
        return Some(0);
    }
    if qualified_name == query {
        return Some(1);
    }
    if name.starts_with(query) {
        return Some(2);
    }
    if qualified_name.starts_with(query) {
        return Some(3);
    }
    if name.contains(query) {
        return Some(4);
    }
    if qualified_name.contains(query) {
        return Some(5);
    }
    if signature.contains(query) {
        return Some(6);
    }
    if path.contains(query) {
        return Some(7);
    }
    None
}

pub(crate) fn example_match_score(
    query: &str,
    title: &str,
    path: &str,
    summary: &str,
    related_symbols: &[String],
    related_modules: &[String],
) -> Option<u8> {
    if query.is_empty() {
        return Some(0);
    }
    if title == query {
        return Some(0);
    }
    if title.starts_with(query) {
        return Some(1);
    }
    if title.contains(query) {
        return Some(2);
    }
    if related_symbols.iter().any(|candidate| candidate == query) {
        return Some(3);
    }
    if related_modules.iter().any(|candidate| candidate == query) {
        return Some(4);
    }
    if related_symbols
        .iter()
        .any(|candidate| candidate.starts_with(query))
    {
        return Some(5);
    }
    if related_modules
        .iter()
        .any(|candidate| candidate.starts_with(query))
    {
        return Some(6);
    }
    if path.contains(query) {
        return Some(7);
    }
    if summary.contains(query) {
        return Some(8);
    }
    if related_symbols
        .iter()
        .any(|candidate| candidate.contains(query))
    {
        return Some(9);
    }
    if related_modules
        .iter()
        .any(|candidate| candidate.contains(query))
    {
        return Some(10);
    }
    None
}

pub(crate) fn import_match_score(
    package_filter: Option<&str>,
    module_filter: Option<&str>,
    import: &crate::analyzers::ImportRecord,
) -> Option<u8> {
    if package_filter.is_none() && module_filter.is_none() {
        return Some(0);
    }

    let target_lower = import.target_package.to_ascii_lowercase();
    let source_lower = import.source_module.to_ascii_lowercase();

    if let Some(package_filter) = package_filter {
        if package_filter == target_lower {
            if let Some(module_filter) = module_filter {
                if module_filter == source_lower {
                    return Some(0);
                }
                if source_lower.starts_with(module_filter) {
                    return Some(1);
                }
                return None;
            }
            return Some(0);
        }
        if target_lower.starts_with(package_filter) {
            if let Some(module_filter) = module_filter {
                if module_filter == source_lower || source_lower.starts_with(module_filter) {
                    return Some(2);
                }
                return None;
            }
            return Some(1);
        }
        if target_lower.contains(package_filter) {
            return Some(3);
        }
        return None;
    }

    if let Some(module_filter) = module_filter {
        if module_filter == source_lower {
            return Some(0);
        }
        if source_lower.starts_with(module_filter) {
            return Some(1);
        }
        if source_lower.contains(module_filter) {
            return Some(2);
        }
        return None;
    }

    Some(0)
}
