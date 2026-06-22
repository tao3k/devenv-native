"""documents test slice 3."""

from __future__ import annotations

from .support import (
    DOCLING_COMMON_SOURCE_SUFFIXES,
    DOCLING_SUPPORTED_DOCUMENT_FORMATS,
    DocumentsFakeDoclingConverter,
    Path,
    default_document_output_dir,
    extract_pdf_resources,
    is_known_docling_source,
)


def test_pdf_compatibility_wrapper_delegates_to_document_extraction(
    tmp_path: Path,
) -> None:
    source = tmp_path / "legacy.pdf"
    source.write_bytes(b"%PDF fixture")

    rows = extract_pdf_resources(source, converter=DocumentsFakeDoclingConverter("# Legacy\n"))

    assert rows[0].sourcePath == str(source)
    assert rows[0].content == "# Legacy\n"


def test_default_document_output_dir_preserves_source_suffix() -> None:
    assert default_document_output_dir("manual.docx") == Path("manual.docx.extracted")


def test_known_docling_source_suffixes_cover_common_document_formats() -> None:
    expected_suffixes = {
        ".pdf",
        ".docx",
        ".doc",
        ".xlsx",
        ".pptx",
        ".md",
        ".html",
        ".csv",
        ".png",
        ".jpg",
        ".tiff",
        ".webp",
        ".xml",
        ".json",
        ".xbrl",
        ".vtt",
        ".tex",
        ".txt",
        ".qmd",
        ".mp3",
        ".wav",
    }

    assert expected_suffixes.issubset(set(DOCLING_COMMON_SOURCE_SUFFIXES))
    assert {
        "PDF",
        "DOCX",
        "DOC (via Rust gateway legacy Office parser)",
        "XLSX",
        "PPTX",
        "HTML",
        "CSV",
        "PNG",
        "Docling JSON",
        "XBRL XML",
        "METS GBS",
        "WebVTT",
        "LaTeX",
        "Plain Text",
        "Audio",
    }.issubset(set(DOCLING_SUPPORTED_DOCUMENT_FORMATS))


def test_is_known_docling_source_is_a_case_insensitive_suffix_helper() -> None:
    assert is_known_docling_source("deck.PPTX")
    assert is_known_docling_source("scan.TIFF")
    assert is_known_docling_source("captions.VTT")
    assert is_known_docling_source("lecture.MP3")
    assert is_known_docling_source("paper.TEX")
    assert not is_known_docling_source("archive.zip")
