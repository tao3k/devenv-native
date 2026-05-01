"""Shared constants for the document extraction benchmark harness."""

from __future__ import annotations

from .common import Path

REPORT_SCHEMA = "xiuxian_wendao.document_extract_perf.v2"
DOCLING_REPO_URL = "https://github.com/docling-project/docling.git"
DOCLING_DEFAULT_GIT_REF = "main"
DOCLING_DATA_RELATIVE_ROOT = Path("tests/data")
PDFIUM_BINARIES_RELEASE = "chromium/7543"
PDFIUM_BINARIES_BASE_URL = (
    "https://github.com/bblanchon/pdfium-binaries/releases/download"
)
DEFAULT_OCR_SHARD_CACHE_MAX_BYTES = 10 * 1024 * 1024 * 1024
OCR_SHARD_CACHE_ROOT_ENV = "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT"

DOCLING_REAL_FIXTURE_PATHS = {
    "pdf": "tests/data/pdf/2206.01062.pdf",
    "docx": "tests/data/docx/word_sample.docx",
    "xlsx": "tests/data/xlsx/xlsx_01.xlsx",
    "pptx": "tests/data/pptx/powerpoint_sample.pptx",
    "markdown": "tests/data/md/wiki.md",
    "asciidoc": "tests/data/asciidoc/test_01.asciidoc",
    "html": "tests/data/html/wiki_duck.html",
    "csv": "tests/data/csv/csv-comma.csv",
    "image-png": "tests/data/2305.03393v1-pg9-img.png",
    "image-tiff": "tests/data/tiff/2206.01062.tif",
    "image-webp": "tests/data/webp/webp-test.webp",
    "uspto-xml": "tests/data/uspto/ipa20110039701.xml",
    "jats-xml": "tests/data/jats/elife-56337.xml",
    "xbrl-xml": "tests/data/xbrl/mlac-20251231.xml",
    "mets-gbs": "tests/data/mets_gbs/32044009881525_select.tar.gz",
    "docling-json": "tests/data/groundtruth/docling_v2/2206.01062.json",
    "webvtt": "tests/data/webvtt/webvtt_example_01.vtt",
    "latex": "tests/data/latex/example_01.tex",
    "audio": "tests/data/audio/sample_10s.mp3",
}

DOCLING_REAL_PDF_CORPUS_FIXTURE_PATHS = {
    "pdf-2203-paper": "tests/data/pdf/2203.01017v2.pdf",
    "pdf-2305-paper-page9": "tests/data/pdf/2305.03393v1-pg9.pdf",
    "pdf-2305-paper": "tests/data/pdf/2305.03393v1.pdf",
    "pdf-amt-handbook": "tests/data/pdf/amt_handbook_sample.pdf",
    "pdf-code-formula": "tests/data/pdf/code_and_formula.pdf",
    "pdf-multi-page": "tests/data/pdf/multi_page.pdf",
    "pdf-normal-4pages": "tests/data/pdf/normal_4pages.pdf",
    "pdf-picture-classification": "tests/data/pdf/picture_classification.pdf",
    "pdf-redp5110-sampled": "tests/data/pdf/redp5110_sampled.pdf",
    "pdf-rtl-01": "tests/data/pdf/right_to_left_01.pdf",
    "pdf-rtl-02": "tests/data/pdf/right_to_left_02.pdf",
    "pdf-rtl-03": "tests/data/pdf/right_to_left_03.pdf",
    "pdf-skipped-1page": "tests/data/pdf/skipped_1page.pdf",
    "pdf-skipped-2pages": "tests/data/pdf/skipped_2pages.pdf",
    "pdf-latex-llncsdoc": "tests/data/latex/2305.03393/llncsdoc.pdf",
}
