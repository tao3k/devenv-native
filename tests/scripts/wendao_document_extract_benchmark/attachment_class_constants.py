"""Attachment class labels and suffix maps."""

from __future__ import annotations

PDF_CLASS = "pdf"

OFFICE_CLASS = "office"

IMAGE_CLASS = "image"

STRUCTURED_TEXT_CLASS = "structured_text"

WEB_CLASS = "web"

TABLE_DATA_CLASS = "table_data"

XML_CLASS = "xml"

SUBTITLE_CLASS = "subtitle"

AUDIO_CLASS = "audio"

DOCLING_JSON_CLASS = "docling_json"

ARCHIVE_DOCUMENT_CLASS = "archive_document"

UNKNOWN_CLASS = "unknown"

FIXTURE_CLASS_OVERRIDES = {
    "pdf": PDF_CLASS,
    "docx": OFFICE_CLASS,
    "xlsx": OFFICE_CLASS,
    "pptx": OFFICE_CLASS,
    "markdown": STRUCTURED_TEXT_CLASS,
    "asciidoc": STRUCTURED_TEXT_CLASS,
    "latex": STRUCTURED_TEXT_CLASS,
    "html": WEB_CLASS,
    "csv": TABLE_DATA_CLASS,
    "image-png": IMAGE_CLASS,
    "image-tiff": IMAGE_CLASS,
    "image-webp": IMAGE_CLASS,
    "uspto-xml": XML_CLASS,
    "jats-xml": XML_CLASS,
    "xbrl-xml": XML_CLASS,
    "mets-gbs": ARCHIVE_DOCUMENT_CLASS,
    "docling-json": DOCLING_JSON_CLASS,
    "webvtt": SUBTITLE_CLASS,
    "audio": AUDIO_CLASS,
}

SUFFIX_CLASS_OVERRIDES = {
    ".pdf": PDF_CLASS,
    ".docx": OFFICE_CLASS,
    ".xlsx": OFFICE_CLASS,
    ".pptx": OFFICE_CLASS,
    ".md": STRUCTURED_TEXT_CLASS,
    ".markdown": STRUCTURED_TEXT_CLASS,
    ".adoc": STRUCTURED_TEXT_CLASS,
    ".asciidoc": STRUCTURED_TEXT_CLASS,
    ".tex": STRUCTURED_TEXT_CLASS,
    ".latex": STRUCTURED_TEXT_CLASS,
    ".txt": STRUCTURED_TEXT_CLASS,
    ".text": STRUCTURED_TEXT_CLASS,
    ".qmd": STRUCTURED_TEXT_CLASS,
    ".rmd": STRUCTURED_TEXT_CLASS,
    ".html": WEB_CLASS,
    ".htm": WEB_CLASS,
    ".xhtml": WEB_CLASS,
    ".csv": TABLE_DATA_CLASS,
    ".tsv": TABLE_DATA_CLASS,
    ".png": IMAGE_CLASS,
    ".jpg": IMAGE_CLASS,
    ".jpeg": IMAGE_CLASS,
    ".tif": IMAGE_CLASS,
    ".tiff": IMAGE_CLASS,
    ".bmp": IMAGE_CLASS,
    ".webp": IMAGE_CLASS,
    ".xml": XML_CLASS,
    ".xbrl": XML_CLASS,
    ".vtt": SUBTITLE_CLASS,
    ".webvtt": SUBTITLE_CLASS,
    ".mp3": AUDIO_CLASS,
    ".wav": AUDIO_CLASS,
    ".m4a": AUDIO_CLASS,
    ".json": DOCLING_JSON_CLASS,
}

COMPOUND_SUFFIX_CLASS_OVERRIDES = {
    ".tar.gz": ARCHIVE_DOCUMENT_CLASS,
    ".docling.json": DOCLING_JSON_CLASS,
}
