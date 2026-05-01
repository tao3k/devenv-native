use super::*;

fn png_bytes(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::from(&b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"[..]);
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 2, 0, 0, 0, 0, 0, 0, 0]);
    bytes
}

fn jpeg_bytes(width: u16, height: u16) -> Vec<u8> {
    let [width_high, width_low] = width.to_be_bytes();
    let [height_high, height_low] = height.to_be_bytes();
    vec![
        0xff,
        0xd8,
        0xff,
        0xe0,
        0x00,
        0x04,
        0x00,
        0x00,
        0xff,
        0xc0,
        0x00,
        0x11,
        0x08,
        height_high,
        height_low,
        width_high,
        width_low,
        0x03,
        0x01,
        0x11,
        0x00,
        0x02,
        0x11,
        0x00,
        0x03,
        0x11,
        0x00,
    ]
}

fn gif_bytes(width: u16, height: u16) -> Vec<u8> {
    let mut bytes = Vec::from(&b"GIF89a"[..]);
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&height.to_le_bytes());
    bytes
}

fn webp_vp8x_bytes(width: u32, height: u32) -> Vec<u8> {
    let width_minus_one = width - 1;
    let height_minus_one = height - 1;
    vec![
        b'R',
        b'I',
        b'F',
        b'F',
        18,
        0,
        0,
        0,
        b'W',
        b'E',
        b'B',
        b'P',
        b'V',
        b'P',
        b'8',
        b'X',
        10,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        (width_minus_one & 0xff) as u8,
        ((width_minus_one >> 8) & 0xff) as u8,
        ((width_minus_one >> 16) & 0xff) as u8,
        (height_minus_one & 0xff) as u8,
        ((height_minus_one >> 8) & 0xff) as u8,
        ((height_minus_one >> 16) & 0xff) as u8,
    ]
}

fn tiff_little_endian_bytes(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::from(&b"II"[..]);
    bytes.extend_from_slice(&42u16.to_le_bytes());
    bytes.extend_from_slice(&8u32.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&256u16.to_le_bytes());
    bytes.extend_from_slice(&4u16.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&257u16.to_le_bytes());
    bytes.extend_from_slice(&4u16.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&height.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes
}

#[test]
fn image_audit_png_reads_dimensions_and_candidate() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let image_path = temp_dir.path().join("fixture.png");
    std::fs::write(image_path.as_path(), png_bytes(640, 480)).map_err(|error| error.to_string())?;

    let audit = audit_image_attachment(image_path.as_path())?;

    assert_eq!(audit.format, "png");
    assert_eq!(audit.mime_type, "image/png");
    assert_eq!(audit.width_px, Some(640));
    assert_eq!(audit.height_px, Some(480));
    assert_eq!(audit.pixel_count, Some(307_200));
    assert_eq!(audit.dimension_source, "png_ihdr");
    assert_eq!(
        audit.rust_acceleration_candidate,
        "image_ocr_cache_candidate"
    );
    assert!(is_supported_image_path(image_path.as_path()));
    Ok(())
}

#[test]
fn image_audit_jpeg_reads_start_of_frame_dimensions() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let image_path = temp_dir.path().join("fixture.jpeg");
    std::fs::write(image_path.as_path(), jpeg_bytes(800, 600))
        .map_err(|error| error.to_string())?;

    let audit = audit_image_attachment(image_path.as_path())?;

    assert_eq!(audit.format, "jpeg");
    assert_eq!(audit.mime_type, "image/jpeg");
    assert_eq!(audit.width_px, Some(800));
    assert_eq!(audit.height_px, Some(600));
    assert_eq!(audit.dimension_source, "jpeg_sof");
    Ok(())
}

#[test]
fn image_audit_gif_reads_logical_screen_dimensions() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let image_path = temp_dir.path().join("fixture.gif");
    std::fs::write(image_path.as_path(), gif_bytes(320, 240)).map_err(|error| error.to_string())?;

    let audit = audit_image_attachment(image_path.as_path())?;

    assert_eq!(audit.format, "gif");
    assert_eq!(audit.mime_type, "image/gif");
    assert_eq!(audit.width_px, Some(320));
    assert_eq!(audit.height_px, Some(240));
    assert_eq!(audit.dimension_source, "gif_lsd");
    Ok(())
}

#[test]
fn image_audit_webp_reads_vp8x_dimensions() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let image_path = temp_dir.path().join("fixture.webp");
    std::fs::write(image_path.as_path(), webp_vp8x_bytes(1024, 768))
        .map_err(|error| error.to_string())?;

    let audit = audit_image_attachment(image_path.as_path())?;

    assert_eq!(audit.format, "webp");
    assert_eq!(audit.mime_type, "image/webp");
    assert_eq!(audit.width_px, Some(1024));
    assert_eq!(audit.height_px, Some(768));
    assert_eq!(audit.dimension_source, "webp_header");
    Ok(())
}

#[test]
fn image_audit_tiff_reads_ifd_dimensions() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let image_path = temp_dir.path().join("fixture.tiff");
    std::fs::write(image_path.as_path(), tiff_little_endian_bytes(2048, 1536))
        .map_err(|error| error.to_string())?;

    let audit = audit_image_attachment(image_path.as_path())?;

    assert_eq!(audit.format, "tiff");
    assert_eq!(audit.mime_type, "image/tiff");
    assert_eq!(audit.width_px, Some(2048));
    assert_eq!(audit.height_px, Some(1536));
    assert_eq!(audit.dimension_source, "tiff_ifd");
    Ok(())
}

#[test]
fn image_audit_marks_oversized_images_as_preflight_candidates() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let image_path = temp_dir.path().join("large.png");
    std::fs::write(image_path.as_path(), png_bytes(6000, 5000))
        .map_err(|error| error.to_string())?;

    let audit = audit_image_attachment(image_path.as_path())?;

    assert_eq!(
        audit.rust_acceleration_candidate,
        "oversized_image_preflight_candidate"
    );
    assert_eq!(audit.pixel_count, Some(30_000_000));
    Ok(())
}

#[test]
fn image_audit_keeps_unknown_extension_unsupported() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source_path = temp_dir.path().join("fixture.bin");
    std::fs::write(source_path.as_path(), b"not an image").map_err(|error| error.to_string())?;

    let audit = audit_image_attachment(source_path.as_path())?;

    assert_eq!(audit.format, "unknown");
    assert_eq!(audit.rust_acceleration_candidate, "unsupported_non_image");
    assert!(!is_supported_image_path(source_path.as_path()));
    Ok(())
}
