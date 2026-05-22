use super::{
    ocr2_region_render_cache_key, ocr2_region_render_cache_key_with_source_hash, sha256_hex,
};
use xiuxian_wendao_attachments::pdf::render::PdfPageBox;
use xiuxian_wendao_attachments::pdf::render::{PdfPageRegionRenderRequest, PdfPageRenderProfile};

#[test]
fn ocr2_region_render_cache_key_accepts_precomputed_source_hash() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    std::fs::write(source.as_path(), b"source-a").map_err(|error| error.to_string())?;
    let profile = PdfPageRenderProfile::ocr_default();
    let region = PdfPageRegionRenderRequest::new(
        1,
        2,
        PdfPageBox::new(10.0, 20.0, 110.0, 220.0),
        Some("000001.000002".to_string()),
    );
    let source_hash = sha256_hex(b"source-a");

    let baseline =
        ocr2_region_render_cache_key(source.as_path(), &profile, std::slice::from_ref(&region))?;

    assert_eq!(
        baseline,
        ocr2_region_render_cache_key_with_source_hash(
            source_hash.as_str(),
            &profile,
            std::slice::from_ref(&region),
        )?
    );
    assert_ne!(
        baseline,
        ocr2_region_render_cache_key_with_source_hash(
            "different-source-hash",
            &profile,
            std::slice::from_ref(&region),
        )?
    );
    Ok(())
}
