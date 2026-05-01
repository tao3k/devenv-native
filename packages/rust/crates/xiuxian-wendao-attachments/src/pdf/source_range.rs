pub(crate) fn source_page_range_all_page_indices(page_count: u32) -> Vec<i32> {
    (0..page_count)
        .filter_map(|page_index| i32::try_from(page_index).ok())
        .collect()
}

pub(crate) fn source_page_range_validate_page_index(
    page_index: i32,
    page_count: u32,
) -> Result<u32, String> {
    let page_index = u32::try_from(page_index).map_err(|_| {
        format!("source page-range selector produced negative page index {page_index}")
    })?;
    if page_index >= page_count {
        return Err(format!(
            "source page-range selector produced out-of-range page index {page_index} for {page_count} pages"
        ));
    }
    Ok(page_index)
}

#[cfg(test)]
#[path = "../../tests/unit/pdf/source_range.rs"]
mod tests;
