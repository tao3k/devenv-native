#[path = "../../src/legacy_office/format.rs"]
mod format;
use format::LegacyOfficeFormat;

#[path = "../../src/legacy_office/metrics.rs"]
mod metrics;
use metrics::legacy_office_quality_metrics;

#[test]
fn xls_metrics_preserve_tabular_boundary_signal() {
    let metrics = legacy_office_quality_metrics(
        LegacyOfficeFormat::Xls,
        "name\tvalue\nalpha\t42\nnotes",
        "# rates\n\n```tsv\nname\tvalue\nalpha\t42\nnotes\n```\n",
    );

    assert_eq!(metrics.line_count, 3);
    assert_eq!(metrics.non_empty_line_count, 3);
    assert_eq!(metrics.tab_delimited_row_count, 2);
    assert_eq!(metrics.max_column_count, 2);
    assert_eq!(metrics.markdown_fenced_block_count, 1);
    assert!(metrics.has_tabular_boundary_signal(LegacyOfficeFormat::Xls));
}
