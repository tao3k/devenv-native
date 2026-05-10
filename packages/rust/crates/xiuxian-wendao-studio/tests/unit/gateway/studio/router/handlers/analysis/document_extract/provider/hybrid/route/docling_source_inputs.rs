#[test]
fn direct_docling_structure_recovery_inputs_keep_text_shortcuts_schedulable() -> Result<(), String>
{
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    std::fs::write(&source, b"%PDF-1.7\n").map_err(|error| error.to_string())?;
    let profiles = vec![
        PdfSourcePageProfile {
            page_index: 0,
            content_bytes: 1024,
            operation_count: 100,
            text_show_ops: 20,
            path_ops: 64,
            rectangle_ops: 0,
            draw_object_ops: 0,
            estimated_weight: 100,
        },
        PdfSourcePageProfile {
            page_index: 1,
            content_bytes: 1024,
            operation_count: 100,
            text_show_ops: 20,
            path_ops: 0,
            rectangle_ops: 0,
            draw_object_ops: 0,
            estimated_weight: 100,
        },
    ];

    let inputs = direct_docling_structure_recovery_source_inputs_for_profiles(
        source.as_path(),
        2,
        profiles.as_slice(),
    )?;
    let fallback_pages = docling_structure_recovery_page_range_fallback_pages(&inputs, true);
    let scheduled =
        scheduled_inputs_without_docling_page_range_fallback_pages(inputs, &fallback_pages);

    assert_eq!(fallback_pages.into_iter().collect::<Vec<_>>(), vec![0]);
    assert_eq!(scheduled.len(), 1);
    assert_eq!(scheduled[0].page_index, 1);
    assert_eq!(scheduled[0].ocr_profile, PDF_OCR_BACKEND_TEXT_PROFILE);
    assert_eq!(scheduled[0].render_profile, "source-pdf-page-range-shards-v1");
    assert_eq!(
        scheduled[0].image_mime_type,
        "application/x-wendao-source-pdf-page"
    );

    Ok(())
}

#[test]
fn direct_docling_structure_recovery_promotes_fragmenting_text_shortcuts() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    std::fs::write(&source, b"%PDF-1.7\n").map_err(|error| error.to_string())?;
    let profiles = (0..9)
        .map(|page_index| PdfSourcePageProfile {
            page_index,
            content_bytes: 1024,
            operation_count: 100,
            text_show_ops: 20,
            path_ops: if page_index == 1 { 0 } else { 64 },
            rectangle_ops: 0,
            draw_object_ops: 0,
            estimated_weight: 100,
        })
        .collect::<Vec<_>>();

    let inputs = direct_docling_structure_recovery_source_inputs_for_profiles(
        source.as_path(),
        9,
        profiles.as_slice(),
    )?;
    let fallback_pages = docling_structure_recovery_page_range_fallback_pages(&inputs, true);
    let scheduled =
        scheduled_inputs_without_docling_page_range_fallback_pages(inputs, &fallback_pages);

    assert_eq!(
        fallback_pages.into_iter().collect::<Vec<_>>(),
        (0..9).collect::<Vec<_>>()
    );
    assert!(scheduled.is_empty());

    Ok(())
}

#[test]
fn direct_docling_structure_recovery_promotes_range_joining_text_shortcut() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    std::fs::write(&source, b"%PDF-1.7\n").map_err(|error| error.to_string())?;
    let profiles = (0..9)
        .map(|page_index| PdfSourcePageProfile {
            page_index,
            content_bytes: 1024,
            operation_count: 100,
            text_show_ops: 20,
            path_ops: if matches!(page_index, 0..=2 | 4..=5) {
                64
            } else {
                0
            },
            rectangle_ops: 0,
            draw_object_ops: 0,
            estimated_weight: 100,
        })
        .collect::<Vec<_>>();

    let inputs = direct_docling_structure_recovery_source_inputs_for_profiles(
        source.as_path(),
        9,
        profiles.as_slice(),
    )?;
    let fallback_pages = docling_structure_recovery_page_range_fallback_pages(&inputs, true);
    let scheduled =
        scheduled_inputs_without_docling_page_range_fallback_pages(inputs, &fallback_pages);

    assert_eq!(
        fallback_pages.into_iter().collect::<Vec<_>>(),
        (0..=5).collect::<Vec<_>>()
    );
    assert_eq!(scheduled.len(), 3);
    assert_eq!(
        scheduled
            .into_iter()
            .map(|input| input.page_index)
            .collect::<Vec<_>>(),
        vec![6, 7, 8]
    );

    Ok(())
}

#[test]
fn direct_docling_structure_recovery_can_leave_text_shortcut_holes() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    std::fs::write(&source, b"%PDF-1.7\n").map_err(|error| error.to_string())?;
    let profiles = (0..9)
        .map(|page_index| PdfSourcePageProfile {
            page_index,
            content_bytes: 1024,
            operation_count: 100,
            text_show_ops: 20,
            path_ops: if page_index == 1 { 0 } else { 64 },
            rectangle_ops: 0,
            draw_object_ops: 0,
            estimated_weight: 100,
        })
        .collect::<Vec<_>>();

    let inputs = direct_docling_structure_recovery_source_inputs_for_profiles_with_lookup(
        source.as_path(),
        9,
        profiles.as_slice(),
        &|key| {
            (key == "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_TEXT_SHORTCUT_PROMOTION")
                .then(|| "disabled".to_string())
        },
    )?;
    let fallback_pages = docling_structure_recovery_page_range_fallback_pages(&inputs, true);
    let scheduled =
        scheduled_inputs_without_docling_page_range_fallback_pages(inputs, &fallback_pages);

    assert_eq!(
        fallback_pages.into_iter().collect::<Vec<_>>(),
        vec![0, 2, 3, 4, 5, 6, 7, 8]
    );
    assert_eq!(scheduled.len(), 1);
    assert_eq!(scheduled[0].page_index, 1);
    assert_eq!(scheduled[0].ocr_profile, PDF_OCR_BACKEND_TEXT_PROFILE);

    Ok(())
}
