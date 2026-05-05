"""document_extract_perf_script test slice 1."""

from __future__ import annotations

import argparse

from .support import (
    Path,
    _load_benchmark_module,
)


def test_docling_real_fixtures_select_all_supported_real_attachment_paths(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    for relative_path in benchmark.DOCLING_REAL_FIXTURE_PATHS.values():
        fixture_path = tmp_path / relative_path
        fixture_path.parent.mkdir(parents=True, exist_ok=True)
        fixture_path.write_bytes(b"fixture")

    fixtures = benchmark.docling_real_fixtures(tmp_path, include_audio=True)
    assert set(fixtures) == set(benchmark.DOCLING_REAL_FIXTURE_PATHS)
    assert fixtures["mets-gbs"].name.endswith(".tar.gz")
    assert fixtures["xbrl-xml"].name == "mlac-20251231.xml"
    assert fixtures["audio"].name == "sample_10s.mp3"


def test_attachment_classification_covers_docling_real_lanes(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()

    assert benchmark.classify_attachment("pdf", tmp_path / "paper.pdf") == "pdf"
    assert benchmark.classify_attachment("pdf-rtl-01", tmp_path / "rtl.pdf") == "pdf"
    assert benchmark.classify_attachment("docx", tmp_path / "word.docx") == "office"
    assert benchmark.classify_attachment("pptx", tmp_path / "deck.pptx") == "office"
    assert benchmark.classify_attachment("xlsx", tmp_path / "book.xlsx") == "office"
    assert (
        benchmark.classify_attachment("markdown", tmp_path / "wiki.md")
        == "structured_text"
    )
    assert benchmark.classify_attachment("latex", tmp_path / "paper.tex") == (
        "structured_text"
    )
    assert benchmark.classify_attachment("html", tmp_path / "wiki.html") == "web"
    assert benchmark.classify_attachment("csv", tmp_path / "rows.csv") == "table_data"
    assert benchmark.classify_attachment("image-png", tmp_path / "page.png") == "image"
    assert benchmark.classify_attachment("jats-xml", tmp_path / "article.xml") == "xml"
    assert (
        benchmark.classify_attachment("mets-gbs", tmp_path / "book.tar.gz")
        == "archive_document"
    )
    assert (
        benchmark.classify_attachment("docling-json", tmp_path / "docling.json")
        == "docling_json"
    )
    assert benchmark.classify_attachment("webvtt", tmp_path / "captions.vtt") == (
        "subtitle"
    )
    assert benchmark.classify_attachment("audio", tmp_path / "sample.mp3") == "audio"
    assert benchmark.classify_attachment("custom", tmp_path / "unknown.bin") == (
        "unknown"
    )


def test_docling_real_fixtures_can_skip_audio(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    for name, relative_path in benchmark.DOCLING_REAL_FIXTURE_PATHS.items():
        if name == "audio":
            continue
        fixture_path = tmp_path / relative_path
        fixture_path.parent.mkdir(parents=True, exist_ok=True)
        fixture_path.write_bytes(b"fixture")

    fixtures = benchmark.docling_real_fixtures(tmp_path, include_audio=False)
    assert "audio" not in fixtures
    assert "webvtt" in fixtures


def test_docling_real_fixtures_keep_pdf_corpus_opt_in(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    fixture_paths = {
        **benchmark.DOCLING_REAL_FIXTURE_PATHS,
        **benchmark.DOCLING_REAL_PDF_CORPUS_FIXTURE_PATHS,
    }
    for relative_path in fixture_paths.values():
        fixture_path = tmp_path / relative_path
        fixture_path.parent.mkdir(parents=True, exist_ok=True)
        fixture_path.write_bytes(b"fixture")

    default_fixtures = benchmark.docling_real_fixtures(
        tmp_path,
        include_audio=False,
    )
    corpus_fixtures = benchmark.docling_real_fixtures(
        tmp_path,
        include_audio=False,
        include_pdf_corpus=True,
    )

    assert "pdf-redp5110-sampled" not in default_fixtures
    assert "pdf-redp5110-sampled" in corpus_fixtures
    assert "pdf" in corpus_fixtures
    assert corpus_fixtures["pdf-redp5110-sampled"].name == "redp5110_sampled.pdf"
    assert "audio" not in corpus_fixtures


def test_select_fixtures_filters_named_fixture(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    fixtures = {
        "pdf": tmp_path / "sample.pdf",
        "audio": tmp_path / "sample.mp3",
    }

    selected = benchmark.select_fixtures(fixtures, ["audio"])

    assert selected == {"audio": tmp_path / "sample.mp3"}


def test_parse_extra_fixtures_resolves_existing_files(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    pdf_fixture = tmp_path / "2604.17337.pdf"
    pdf_fixture.write_bytes(b"%PDF")

    fixtures = benchmark.parse_extra_fixtures([f"arxiv-2604-17337={pdf_fixture}"])

    assert fixtures == {"arxiv-2604-17337": pdf_fixture.resolve()}


def test_explicit_fixture_suite_uses_only_extra_real_inputs(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    pdf_fixture = tmp_path / "2604.17337.pdf"
    pdf_fixture.write_bytes(b"%PDF")
    args = argparse.Namespace(
        fixture_suite="explicit",
        extra_fixture=[f"arxiv-2604-17337={pdf_fixture}"],
    )

    fixtures, real_fixture_root = benchmark.resolve_fixtures(
        args, tmp_path / "fixtures"
    )

    assert fixtures == {"arxiv-2604-17337": pdf_fixture.resolve()}
    assert real_fixture_root is None


def test_explicit_fixture_suite_requires_extra_fixture(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    args = argparse.Namespace(fixture_suite="explicit", extra_fixture=[])

    try:
        benchmark.resolve_fixtures(args, tmp_path / "fixtures")
    except SystemExit as error:
        assert "--fixture-suite explicit requires --extra-fixture" in str(error)
    else:
        raise AssertionError("explicit suite without fixtures should fail")
