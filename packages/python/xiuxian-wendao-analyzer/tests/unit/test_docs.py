from __future__ import annotations

from pathlib import Path


def _package_root() -> Path:
    return Path(__file__).resolve().parents[2]


def test_first_analyzer_author_tutorial_mentions_supported_workflows() -> None:
    tutorial = (
        _package_root() / "docs" / "first_analyzer_author_tutorial.md"
    ).read_text(encoding="utf-8")

    assert "Workflow 1: Offline Repo Search Authoring With Scripted Results" in tutorial
    assert "Workflow 2: Host-Backed Repo Search With Built-In Ranking" in tutorial
    assert (
        "Workflow 3: Host-Backed Repo Search With A Custom Python Analyzer" in tutorial
    )
    assert (
        "Workflow 4: PDF Attachment Search Then Analyze The Returned Table" in tutorial
    )
    assert "Workflow 5: Docling Document Extraction Into Arrow Rows" in tutorial
    assert "Workflow 6: Analyze An Already Materialized Rust Query Result" in tutorial
    assert "examples/document_extraction_workflow.py" in tutorial
    assert "examples/attachment_pdf_analyzer_workflow.py" in tutorial
    assert "examples/scripted_repo_search_workflow.py" in tutorial
    assert "examples/repo_search_workflow.py" in tutorial
    assert "examples/custom_repo_analyzer_workflow.py" in tutorial
    assert "WendaoArrowSession.for_repo_search_testing(...)" in tutorial
    assert "WendaoArrowSession.attachment_search(...)" in tutorial
    assert "run_repo_analysis(...)" in tutorial
    assert "run_table_analysis(...)" in tutorial
    assert "extract_document_table(...)" in tutorial
    assert "uv run wendao-document-extract --host 0.0.0.0 --port 50051" in tutorial
    assert "/analysis/document-extract" in tutorial
    assert "analyze_table(...)" in tutorial
    assert "does not own rerank workflows" in tutorial


def test_custom_analyzer_tutorial_mentions_contract_and_example() -> None:
    tutorial = (
        _package_root() / "docs" / "write_your_first_custom_analyzer.md"
    ).read_text(encoding="utf-8")

    assert "The Smallest Honest Contract" in tutorial
    assert "def analyze_rows(self, rows: list[dict[str, object]])" in tutorial
    assert "stable `rank` field" in tutorial
    assert "custom_repo_analyzer_workflow.py" in tutorial
    assert "scripted_repo_search_workflow.py" in tutorial
    assert "attachment_pdf_analyzer_workflow.py" in tutorial
    assert "document_extraction_workflow.py" in tutorial
    assert "run_repo_analysis(...)" in tutorial
    assert "summarize_repo_analysis(...)" in tutorial
    assert "analyze_table(...)" in tutorial
    assert "WendaoArrowSession.attachment_search(...)" in tutorial
    assert "extract_document_table(...)" in tutorial
    assert "does not own rerank transport" in tutorial


def test_release_policy_mentions_beta_contract_and_workflow_stability() -> None:
    policy = (
        _package_root() / "docs" / "release_and_compatibility_policy.md"
    ).read_text(encoding="utf-8")

    assert "This package is currently in beta." in policy
    assert "Compatibility Rule For This Beta" in policy
    assert "The current lockable beta baseline is `0.2.1`." in policy
    assert "workflow-frozen, not helper-frozen" in policy
    assert "run_repo_analysis(...)" in policy
    assert "WendaoArrowSession.attachment_search(...)" in policy
    assert "generic rows, table, and query analysis over Rust-returned data" in policy
    assert (
        "Docling-backed multi-format document extraction into Arrow resource rows"
        in policy
    )
    assert "analyzer format metadata is a UX hint surface" in policy
    assert "Wendao-facing `/analysis/document-extract` service route" in policy
    assert "analyzer-owned rerank helpers are out of scope" in policy
    assert "Current Beta Exit Reading" in policy
    assert "Current Beta Freeze Reading" in policy


def test_external_consumer_checklist_mentions_environment_and_boundary() -> None:
    checklist = (_package_root() / "docs" / "external_consumer_checklist.md").read_text(
        encoding="utf-8"
    )

    assert "Python `>=3.12`" in checklist
    assert "pyarrow>=14.0.0" in checklist
    assert "uv run python examples/scripted_repo_search_workflow.py" in checklist
    assert "uv run python examples/attachment_pdf_analyzer_workflow.py" in checklist
    assert "uv run python examples/document_extraction_workflow.py" in checklist
    assert "plain `python examples/...`" in checklist.lower()
    assert "WendaoArrowSession.for_repo_search_testing(...)" in checklist
    assert "WendaoArrowSession.attachment_search(...)" in checklist
    assert "wendao_search_flight_server" in checklist
    assert (
        "cargo build -p xiuxian-wendao --features julia --bin wendao_search_flight_server --bin wendao_search_seed_sample"
        in checklist
    )
    assert 'tmp_root="$(mktemp -d)"' in checklist
    assert 'wendao_search_seed_sample alpha/repo "$tmp_root"' in checklist
    assert "examples/repo_search_workflow.py --host 127.0.0.1 --port 8815" in checklist
    assert "examples/host_backed_repo_search_beta_smoke.py --port 0" in checklist
    assert (
        "examples/host_backed_repo_search_beta_smoke.py --mode custom --port 0"
        in checklist
    )
    assert (
        "examples/host_backed_repo_search_beta_smoke.py --port 0 --keep-workspace"
        in checklist
    )
    assert "Use `--keep-workspace`" in checklist
    assert "analyze_table(...)" in checklist
    assert "extract_document_table(...)" in checklist
    assert "uv run wendao-document-extract --host 0.0.0.0 --port 50051" in checklist
    assert "/analysis/document-extract" in checklist
    assert "There is no analyzer-owned rerank workflow" in checklist


def test_readme_freezes_v1_documentation_set() -> None:
    readme = (_package_root() / "README.md").read_text(encoding="utf-8")

    assert "## Documentation Set" in readme
    assert "docs/first_analyzer_author_tutorial.md" in readme
    assert "docs/write_your_first_custom_analyzer.md" in readme
    assert "docs/release_and_compatibility_policy.md" in readme
    assert "docs/external_consumer_checklist.md" in readme
    assert "intended v1 public docs surface index" in readme


def test_readme_marks_package_as_beta_with_rust_query_boundary() -> None:
    readme = (_package_root() / "README.md").read_text(encoding="utf-8")

    assert "is a beta Python analyzer and document parsing" in readme
    assert "package built on top of `wendao-core-lib`" in readme
    assert "Current Beta Surface" in readme
    assert "It does not own:" in readme
    assert "rerank transport contracts" in readme
    assert "fetch it through `wendao-core-lib` or `wendao-arrow-interface`" in readme
    assert "Docling-backed document extraction helpers" in readme
    assert "PDF, DOCX, XLSX, PPTX, Markdown" in readme
    assert "is_known_docling_source(...)" in readme
    assert "Wendao Document Service" in readme
    assert "wendao-document-extract" in readme
    assert "/analysis/document-extract" in readme


def test_readme_records_beta_readiness_and_known_gaps() -> None:
    readme = (_package_root() / "README.md").read_text(encoding="utf-8")

    assert "## Beta Readiness" in readme
    assert "ready now:" in readme
    assert "known gaps before broader adoption:" in readme
    assert "uv run python ...` from the package directory" in readme
    assert "no analyzer-owned rerank helper surface" in readme
    assert "no GA-level release promise yet" in readme
    assert "Docling is optional through the `documents` extra" in readme
    assert "tests/scripts/audio_asr_diagnostic.py" in readme
    assert "audio_shards.json" in readme
    assert "xiuxian_wendao.audio_shards.v1" in readme
    assert "materialize normalized shard media in parallel with local" in readme
    assert "tests/scripts/fireredasr2s_local_setup.py" in readme
    assert "OPENROUTER_API_KEY" in readme
    assert "environment-provided diagnostic backend" in readme
    assert "Wendao-facing Arrow Flight service entrypoint" in readme


def test_readme_records_beta_exit_audit() -> None:
    readme = (_package_root() / "README.md").read_text(encoding="utf-8")

    assert "## Beta Exit Audit" in readme
    assert "exit-ready now:" in readme
    assert "not exit-ready yet:" in readme
    assert "real-host repo-search coverage exists" in readme
    assert "Rust-query-first analyzer boundary" in readme


def test_readme_records_beta_freeze_audit() -> None:
    readme = (_package_root() / "README.md").read_text(encoding="utf-8")

    assert "## Beta Freeze Audit" in readme
    assert (
        "The current package boundary is now intentionally lockable as `0.2.1`."
        in readme
    )
    assert "frozen for this beta trial:" in readme
    assert "examples/scripted_repo_search_workflow.py" in readme
    assert "examples/attachment_pdf_analyzer_workflow.py" in readme
    assert "examples/document_extraction_workflow.py" in readme
    assert "examples/host_backed_repo_search_beta_smoke.py" in readme
    assert "not frozen for this beta trial:" in readme
    assert "workflow-frozen, not helper-frozen" in readme


def test_readme_mentions_host_backed_beta_smoke_example() -> None:
    readme = (_package_root() / "README.md").read_text(encoding="utf-8")

    assert "examples/repo_search_workflow.py" in readme
    assert "examples/custom_repo_analyzer_workflow.py" in readme
    assert "examples/attachment_pdf_analyzer_workflow.py" in readme
    assert "examples/document_extraction_workflow.py" in readme
    assert "examples/host_backed_repo_search_beta_smoke.py" in readme
    assert "one-shot beta smoke for the full host-backed repo-search path" in readme
    assert "--mode custom --port 0" in readme
