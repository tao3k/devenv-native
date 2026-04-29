from __future__ import annotations

import tomllib
from inspect import isclass
from pathlib import Path

import xiuxian_wendao_analyzer as analyzer


def _pyproject_version() -> str:
    pyproject = Path(__file__).resolve().parents[1] / "pyproject.toml"
    return tomllib.loads(pyproject.read_text(encoding="utf-8"))["project"]["version"]


def test_public_exports_resolve_from_package_root() -> None:
    exported_names = analyzer.__all__

    assert exported_names
    assert len(exported_names) == len(set(exported_names))

    for name in exported_names:
        assert hasattr(analyzer, name), name


def test_public_exports_include_core_analyzer_surface() -> None:
    assert "__version__" in analyzer.__all__
    assert "AnalyzerConfig" in analyzer.__all__
    assert "AnalyzerResultRow" in analyzer.__all__
    assert "AnalysisSummary" in analyzer.__all__
    assert "RowsAnalysisRun" in analyzer.__all__
    assert "TableAnalysisRun" in analyzer.__all__
    assert "QueryAnalysisRun" in analyzer.__all__
    assert "RepoAnalysisRun" in analyzer.__all__
    assert "DocumentResourceRow" in analyzer.__all__
    assert "DOCUMENT_RESOURCE_ARROW_CACHE_NAME" in analyzer.__all__
    assert "DOCUMENT_RESOURCE_SCHEMA" in analyzer.__all__
    assert "DOCLING_COMMON_SOURCE_SUFFIXES" in analyzer.__all__
    assert "DOCLING_SUPPORTED_DOCUMENT_FORMATS" in analyzer.__all__
    assert "DocumentExtractFlightServer" in analyzer.__all__
    assert "ANALYSIS_DOCUMENT_EXTRACT_ROUTE" in analyzer.__all__
    assert "ANALYSIS_PDF_OCR_SHARDS_ROUTE" in analyzer.__all__
    assert "PDF_OCR_SHARD_INPUT_SCHEMA" in analyzer.__all__
    assert "PDF_OCR_SHARD_RESULT_SCHEMA" in analyzer.__all__
    assert "PdfOcrShardWorkerProtocol" in analyzer.__all__
    assert "ScoreRankAnalyzer" in analyzer.__all__
    assert "analyze_query" in analyzer.__all__
    assert "analyze_repo_search" in analyzer.__all__
    assert "run_query_analysis" in analyzer.__all__
    assert "run_repo_search_analysis" in analyzer.__all__
    assert "summarize_query_route" in analyzer.__all__
    assert "summarize_repo_query_text_results" in analyzer.__all__
    assert "run_rows_analysis" in analyzer.__all__
    assert "extract_document_resources" in analyzer.__all__
    assert "extract_document_table" in analyzer.__all__
    assert "extract_pdf_resources" in analyzer.__all__
    assert "is_known_docling_source" in analyzer.__all__
    assert "build_document_extract_table" in analyzer.__all__
    assert "build_pdf_ocr_shard_result_table" in analyzer.__all__
    assert "summarize_rows_analysis" in analyzer.__all__


def test_public_exports_preserve_expected_symbol_kinds() -> None:
    assert isclass(analyzer.AnalyzerConfig)
    assert isclass(analyzer.AnalyzerResultRow)
    assert isclass(analyzer.AnalysisSummary)
    assert isclass(analyzer.QueryAnalysisRun)
    assert isclass(analyzer.RepoAnalysisRun)
    assert isclass(analyzer.RowsAnalysisRun)
    assert isclass(analyzer.TableAnalysisRun)
    assert isclass(analyzer.DocumentResourceRow)
    assert isclass(analyzer.PdfOcrShardWorkerProtocol)
    assert analyzer.DOCUMENT_RESOURCE_ARROW_CACHE_NAME == "_resources.arrow"
    assert (
        analyzer.PDF_OCR_SHARD_INPUT_SCHEMA_VERSION
        == "xiuxian_wendao.pdf_ocr_shard_input.v1"
    )
    assert (
        analyzer.PDF_OCR_SHARD_RESULT_SCHEMA_VERSION
        == "xiuxian_wendao.pdf_ocr_shard_result.v1"
    )
    assert isclass(analyzer.DocumentExtractFlightServer)
    assert isclass(analyzer.ScoreRankAnalyzer)

    assert callable(analyzer.build_analyzer)
    assert callable(analyzer.parse_analyzer_result_rows)
    assert callable(analyzer.analyze_query)
    assert callable(analyzer.analyze_repo_search)
    assert callable(analyzer.run_query_analysis)
    assert callable(analyzer.run_repo_search_analysis)
    assert callable(analyzer.run_rows_analysis)
    assert callable(analyzer.run_table_analysis)
    assert callable(analyzer.extract_document_resources)
    assert callable(analyzer.extract_document_table)
    assert callable(analyzer.extract_pdf_resources)
    assert callable(analyzer.is_known_docling_source)
    assert callable(analyzer.build_document_extract_table)
    assert callable(analyzer.build_pdf_ocr_shard_result_table)
    assert callable(analyzer.summarize_query_route)
    assert callable(analyzer.summarize_repo_query_text_results)
    assert callable(analyzer.summarize_rows_analysis)
    assert callable(analyzer.summarize_table_analysis)


def test_package_root_exports_version_matching_pyproject() -> None:
    assert analyzer.__version__ == "0.2.1"
    assert analyzer.__version__ == _pyproject_version()
