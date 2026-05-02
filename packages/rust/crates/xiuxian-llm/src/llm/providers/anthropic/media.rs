//! Anthropic image media-type normalization helpers.

use base64::Engine as _;

/// Normalize image media type for Anthropic image blocks.
///
/// Anthropic `messages` image content only accepts a strict image MIME subset.
/// Unknown/opaque values (for example `application/octet-stream`) are normalized
/// by probing the base64 payload header.
#[must_use]
pub fn normalize_anthropic_image_media_type(media_type: &str, base64_data: &str) -> String {
    if let Some(normalized) = normalize_explicit_image_media_type(media_type) {
        return normalized.to_string();
    }
    if let Some(detected) = detect_image_media_type_from_base64(base64_data) {
        return detected.to_string();
    }
    "image/jpeg".to_string()
}

fn normalize_explicit_image_media_type(media_type: &str) -> Option<&'static str> {
    match media_type.trim().to_ascii_lowercase().as_str() {
        "image/png" => Some("image/png"),
        "image/jpeg" | "image/jpg" => Some("image/jpeg"),
        "image/webp" => Some("image/webp"),
        "image/gif" => Some("image/gif"),
        _ => None,
    }
}

fn detect_image_media_type_from_base64(base64_data: &str) -> Option<&'static str> {
    let payload = extract_base64_payload(base64_data);
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .ok()?;
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some("image/png");
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    None
}

fn extract_base64_payload(raw: &str) -> &str {
    let trimmed = raw.trim();
    if let Some((prefix, payload)) = trimmed.split_once(',')
        && prefix.to_ascii_lowercase().contains("base64")
    {
        return payload;
    }
    trimmed
}
