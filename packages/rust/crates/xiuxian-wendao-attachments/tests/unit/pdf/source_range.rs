use super::*;

#[test]
fn source_range_all_indices_cover_every_page() {
    assert_eq!(source_page_range_all_page_indices(4), vec![0, 1, 2, 3]);
}

#[test]
fn source_range_page_index_validation_rejects_negative_indices() {
    let error = match source_page_range_validate_page_index(-1, 3) {
        Ok(page_index) => panic!("expected negative index to fail, got {page_index}"),
        Err(error) => error,
    };

    assert!(error.contains("negative page index"));
}

#[test]
fn source_range_page_index_validation_rejects_out_of_range_indices() {
    let error = match source_page_range_validate_page_index(3, 3) {
        Ok(page_index) => panic!("expected out-of-range index to fail, got {page_index}"),
        Err(error) => error,
    };

    assert!(error.contains("out-of-range page index"));
}
