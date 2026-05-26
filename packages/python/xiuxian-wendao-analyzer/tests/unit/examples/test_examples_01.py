"""examples test slice 1."""

from __future__ import annotations

from .support import (
    _package_root,
    _run_example_via_uv,
)


def test_shipped_example_set_matches_current_beta_freeze() -> None:
    example_names = {
        path.name for path in (_package_root() / "examples").glob("*.py") if path.is_file()
    }

    assert example_names == {
        "attachment_pdf_analyzer_workflow.py",
        "custom_repo_analyzer_workflow.py",
        "document_extraction_workflow.py",
        "host_backed_repo_search_beta_smoke.py",
        "repo_search_workflow.py",
        "scripted_repo_search_workflow.py",
    }


def test_scripted_repo_search_example_runs() -> None:
    result = _run_example_via_uv("examples/scripted_repo_search_workflow.py")

    assert "query_text= alpha" in result.stdout
    assert "rows= 3" in result.stdout
    assert "top_path= src/alpha.py" in result.stdout
    assert "top_rank= 1" in result.stdout
    assert "recorded_calls= 1" in result.stdout
    assert "recorded_route= /search/repos/main" in result.stdout


def test_attachment_pdf_analyzer_example_runs_scripted() -> None:
    result = _run_example_via_uv("examples/attachment_pdf_analyzer_workflow.py")

    assert "mode= scripted" in result.stdout
    assert "query_text= architecture" in result.stdout
    assert "rows= 2" in result.stdout
    assert "top_path= assets/design-review.pdf" in result.stdout
    assert "top_rank= 1" in result.stdout
    assert "top_attachment_name= design-review.pdf" in result.stdout
    assert "top_source_title= Architecture Notes" in result.stdout
    assert "recorded_calls= 1" in result.stdout
    assert "recorded_route= /search/attachments" in result.stdout


def test_document_extraction_example_runs_fixture_mode() -> None:
    result = _run_example_via_uv("examples/document_extraction_workflow.py")

    assert "mode= fixture" in result.stdout
    assert "known_docling_source= True" in result.stdout
    assert (
        "supported_formats= PDF,DOCX,DOC (via legacy Office pre-conversion),XLSX,PPTX"
        in result.stdout
    )
    assert "common_suffixes= .pdf,.docx,.doc,.xlsx,.pptx" in result.stdout
    assert "rows= 1" in result.stdout
    assert (
        "sourcePath,resourceType,resourcePath,pageIndex,caption,content,mimeType,status,elementId"
        in result.stdout
    )
    assert "top_status= ok" in result.stdout
    assert "top_resource_type= document" in result.stdout
    assert "top_mime_type= text/markdown" in result.stdout
    assert "top_content= # Parsed fixture" in result.stdout


def test_repo_search_example_exposes_help() -> None:
    result = _run_example_via_uv("examples/repo_search_workflow.py", "--help")

    assert "Run a host-backed repo-search analyzer workflow." in result.stdout
    assert "--query-text" in result.stdout
    assert "--path-prefix" in result.stdout


def test_custom_repo_search_example_exposes_help() -> None:
    result = _run_example_via_uv("examples/custom_repo_analyzer_workflow.py", "--help")

    assert "Run a host-backed repo-search workflow with a custom Python analyzer." in result.stdout
    assert "--query-text" in result.stdout
    assert "--path-prefix" in result.stdout
