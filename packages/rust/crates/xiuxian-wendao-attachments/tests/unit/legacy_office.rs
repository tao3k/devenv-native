use std::path::Path;

use xiuxian_wendao_attachments::legacy_office::{
    LegacyOfficeFormat, extract_legacy_office, is_supported_legacy_office_path,
    legacy_office_format,
};

#[test]
fn legacy_office_suffix_detection_is_case_insensitive() {
    assert_eq!(
        legacy_office_format(Path::new("contract.DOC")),
        Some(LegacyOfficeFormat::Doc)
    );
    assert_eq!(
        legacy_office_format(Path::new("sheet.XLS")),
        Some(LegacyOfficeFormat::Xls)
    );
    assert_eq!(
        legacy_office_format(Path::new("deck.PPT")),
        Some(LegacyOfficeFormat::Ppt)
    );
    assert!(!is_supported_legacy_office_path(Path::new("modern.docx")));
}

#[test]
fn legacy_office_real_fixture_diagnostic_extracts_text_when_env_is_set() -> Result<(), String> {
    let Some(path) = std::env::var_os("WENDAO_LEGACY_OFFICE_DIAGNOSTIC_PATH") else {
        return Ok(());
    };
    let path = std::path::PathBuf::from(path);
    let extraction = extract_legacy_office(path.as_path())?;
    assert!(!extraction.text.trim().is_empty());
    assert!(extraction.markdown.starts_with("# "));
    eprintln!(
        "legacy_office_diagnostic format={} text_chars={} markdown_chars={}",
        extraction.format.extension(),
        extraction.text.chars().count(),
        extraction.markdown.chars().count()
    );
    Ok(())
}
