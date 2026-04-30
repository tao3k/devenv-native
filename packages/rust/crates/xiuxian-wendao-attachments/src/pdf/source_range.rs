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
mod tests {
    use super::*;

    #[test]
    fn source_range_all_indices_cover_every_page() {
        assert_eq!(source_page_range_all_page_indices(4), vec![0, 1, 2, 3]);
    }

    #[test]
    fn source_range_page_index_validation_rejects_negative_indices() {
        let error = source_page_range_validate_page_index(-1, 3).unwrap_err();

        assert!(error.contains("negative page index"));
    }

    #[test]
    fn source_range_page_index_validation_rejects_out_of_range_indices() {
        let error = source_page_range_validate_page_index(3, 3).unwrap_err();

        assert!(error.contains("out-of-range page index"));
    }
}
