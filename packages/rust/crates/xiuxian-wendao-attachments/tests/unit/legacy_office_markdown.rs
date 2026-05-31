use std::path::Path;

#[path = "../../src/legacy_office/format.rs"]
mod format;
use format::LegacyOfficeFormat;

#[path = "../../src/legacy_office/markdown.rs"]
mod markdown;
use markdown::legacy_office_markdown;

#[test]
fn xls_markdown_preserves_tabular_boundaries() {
    let markdown = match legacy_office_markdown(
        Path::new("rates.xls"),
        LegacyOfficeFormat::Xls,
        "name\tvalue\nalpha\t42",
    ) {
        Ok(markdown) => markdown,
        Err(error) => panic!("markdown failed: {error}"),
    };

    assert!(markdown.contains("```tsv\nname\tvalue\nalpha\t42\n```"));
}
