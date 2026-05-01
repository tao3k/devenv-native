use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub(super) struct ImageFormatHint {
    pub(super) format: &'static str,
    pub(super) mime_type: &'static str,
}

pub(super) fn image_format_hint(path: &Path) -> Option<ImageFormatHint> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some(ImageFormatHint {
            format: "png",
            mime_type: "image/png",
        }),
        "jpg" | "jpeg" => Some(ImageFormatHint {
            format: "jpeg",
            mime_type: "image/jpeg",
        }),
        "tif" | "tiff" => Some(ImageFormatHint {
            format: "tiff",
            mime_type: "image/tiff",
        }),
        "bmp" => Some(ImageFormatHint {
            format: "bmp",
            mime_type: "image/bmp",
        }),
        "webp" => Some(ImageFormatHint {
            format: "webp",
            mime_type: "image/webp",
        }),
        "gif" => Some(ImageFormatHint {
            format: "gif",
            mime_type: "image/gif",
        }),
        _ => None,
    }
}
