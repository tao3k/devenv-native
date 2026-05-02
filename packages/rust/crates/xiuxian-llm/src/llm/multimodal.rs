//! Multimodal marker parsing and image-source normalization.

use super::error::LlmError;
use super::error::LlmResult;
use base64::Engine;

/// Platform-neutral multimodal content part extracted from text markers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultimodalContentPart {
    /// Plain text segment.
    Text(String),
    /// Image URL or data URI segment.
    ImageUrl {
        /// Remote URL or `data:` URI passed to multimodal providers.
        url: String,
    },
}

/// Base64-encoded image payload ready for providers that require inline binary content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base64ImageSource {
    /// MIME type (for example `image/jpeg`).
    pub media_type: String,
    /// Base64-encoded image bytes.
    pub data: String,
}

fn parse_image_marker_url(marker: &str) -> Option<&str> {
    let (kind, target) = marker.split_once(':')?;
    if !matches!(kind.trim().to_ascii_uppercase().as_str(), "IMAGE" | "PHOTO") {
        return None;
    }

    let trimmed_target = target.trim();
    if trimmed_target.is_empty() || trimmed_target.contains('\n') {
        return None;
    }
    if trimmed_target.starts_with("http://")
        || trimmed_target.starts_with("https://")
        || trimmed_target.starts_with("data:")
    {
        return Some(trimmed_target);
    }

    None
}

/// Parse text that may include media markers (for example `[IMAGE:https://...]`)
/// into multimodal parts consumable by LLM adapters.
///
/// Returns `None` when no valid multimodal marker is present.
#[must_use]
pub fn parse_multimodal_text_content(content: &str) -> Option<Vec<MultimodalContentPart>> {
    let mut parts = Vec::new();
    let mut text_start = 0usize;
    let mut scan = 0usize;
    let mut has_image_part = false;

    while scan < content.len() {
        let Some(open_rel) = content[scan..].find('[') else {
            break;
        };
        let open = scan + open_rel;
        let Some(close_rel) = content[open..].find(']') else {
            break;
        };
        let close = open + close_rel;
        let marker = &content[open + 1..close];

        if let Some(image_url) = parse_image_marker_url(marker) {
            if text_start < open {
                parts.push(MultimodalContentPart::Text(
                    content[text_start..open].to_string(),
                ));
            }
            parts.push(MultimodalContentPart::ImageUrl {
                url: image_url.to_string(),
            });
            has_image_part = true;
            text_start = close + 1;
        }

        scan = close + 1;
    }

    if !has_image_part {
        return None;
    }

    if text_start < content.len() {
        parts.push(MultimodalContentPart::Text(
            content[text_start..].to_string(),
        ));
    }

    Some(parts)
}

/// Parse a `data:` URI to a base64 image source.
#[must_use]
pub fn parse_data_uri_image_source(uri: &str) -> Option<Base64ImageSource> {
    let stripped = uri.strip_prefix("data:")?;
    let (meta, data) = stripped.split_once(',')?;
    let media_type = meta.split(';').next().unwrap_or("image/jpeg");
    Some(Base64ImageSource {
        media_type: media_type.to_string(),
        data: data.to_string(),
    })
}

/// Resolve an image reference (`data:` URI or HTTP(S) URL) into base64 payload.
///
/// # Errors
///
/// Returns an error when the input is neither `data:` nor HTTP(S), the fetch fails,
/// the response body is empty, or bytes cannot be read.
pub async fn resolve_image_source_to_base64(
    client: &reqwest::Client,
    image_ref: &str,
) -> LlmResult<Base64ImageSource> {
    if let Some(source) = parse_data_uri_image_source(image_ref) {
        return Ok(source);
    }
    if !image_ref.starts_with("http://") && !image_ref.starts_with("https://") {
        return Err(LlmError::InvalidImageReference);
    }

    let response = client
        .get(image_ref)
        .send()
        .await
        .map_err(|error| LlmError::ImageDownloadRequestFailed { source: error })?;
    if !response.status().is_success() {
        return Err(LlmError::ImageDownloadFailed {
            status: response.status(),
        });
    }

    let media_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(';')
                .next()
                .unwrap_or("image/jpeg")
                .trim()
                .to_string()
        })
        .filter(|value| !value.is_empty())
        .or_else(|| infer_media_type_from_url(image_ref).map(str::to_string))
        .unwrap_or_else(|| "image/jpeg".to_string());

    let bytes = response
        .bytes()
        .await
        .map_err(|error| LlmError::ImageBytesReadFailed { source: error })?;
    if bytes.is_empty() {
        return Err(LlmError::ImageEmptyBody);
    }

    let data = base64::engine::general_purpose::STANDARD.encode(bytes.as_ref());
    Ok(Base64ImageSource { media_type, data })
}

fn infer_media_type_from_url(image_url: &str) -> Option<&'static str> {
    let path = reqwest::Url::parse(image_url).ok()?.path().to_string();
    let extension = std::path::Path::new(path.as_str()).extension()?;
    if extension.eq_ignore_ascii_case("png") {
        return Some("image/png");
    }
    if extension.eq_ignore_ascii_case("gif") {
        return Some("image/gif");
    }
    if extension.eq_ignore_ascii_case("webp") {
        return Some("image/webp");
    }
    if extension.eq_ignore_ascii_case("bmp") {
        return Some("image/bmp");
    }
    if extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg") {
        return Some("image/jpeg");
    }
    None
}
