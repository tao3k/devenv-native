"""documents test slice 2."""

from __future__ import annotations

import subprocess

from xiuxian_wendao_analyzer.document_cache import _write_cached_resources
from xiuxian_wendao_analyzer.document_isolation import run_isolated_document_extract

from .support import (
    DOCUMENT_RESOURCE_SCHEMA,
    DOCUMENT_STRUCTURE_SCHEMA,
    DOCUMENT_STRUCTURE_SCHEMA_VERSION,
    DOCUMENT_TIMING_ARROW_CACHE_NAME,
    DOCUMENT_TIMING_SCHEMA,
    DOCUMENT_TIMING_SCHEMA_VERSION,
    DocumentResourceRow,
    DocumentsFakeDoclingConverter,
    DocumentStructureBlock,
    FailingConverter,
    Path,
    document_resources_to_table,
    document_structure_to_table,
    document_timing_to_table,
    extract_document_resources,
    pytest,
    warm_document_arrow_runtime,
)


def test_document_resources_to_table_accepts_mappings() -> None:
    table = document_resources_to_table(
        [
            {
                "sourcePath": "source.pdf",
                "resourceType": "document",
                "resourcePath": "source.md",
                "pageIndex": 0,
                "caption": "",
                "content": "# Source\n",
                "mimeType": "text/markdown",
                "status": "ok",
                "elementId": "_main",
            }
        ]
    )

    assert table.schema == DOCUMENT_RESOURCE_SCHEMA
    assert table.to_pylist()[0]["sourcePath"] == "source.pdf"


def test_document_structure_to_table_sorts_reading_order() -> None:
    table = document_structure_to_table(
        [
            DocumentStructureBlock(
                contractVersion=DOCUMENT_STRUCTURE_SCHEMA_VERSION,
                sourcePath="source.pdf",
                sourceContentHash="hash",
                blockId="b",
                parentBlockId="",
                pageIndex=1,
                blockIndex=2,
                readingOrderKey="000001.000002",
                blockType="ocr_text",
                resourceElementId="b",
                content="second",
                mimeType="text/plain",
                status="succeeded",
                engine="docling",
                confidence=None,
                bboxLeft=None,
                bboxTop=None,
                bboxRight=None,
                bboxBottom=None,
                provenance="{}",
            ),
            DocumentStructureBlock(
                contractVersion=DOCUMENT_STRUCTURE_SCHEMA_VERSION,
                sourcePath="source.pdf",
                sourceContentHash="hash",
                blockId="a",
                parentBlockId="",
                pageIndex=0,
                blockIndex=1,
                readingOrderKey="000000.000001",
                blockType="text_page",
                resourceElementId="a",
                content="first",
                mimeType="text/markdown",
                status="ok",
                engine="wendao-hybrid",
                confidence=None,
                bboxLeft=None,
                bboxTop=None,
                bboxRight=None,
                bboxBottom=None,
                provenance="{}",
            ),
        ]
    )

    assert table.schema == DOCUMENT_STRUCTURE_SCHEMA
    assert [row["blockId"] for row in table.to_pylist()] == ["a", "b"]


def test_document_timing_to_table_uses_stable_schema() -> None:
    table = document_timing_to_table(
        [
            {
                "contractVersion": DOCUMENT_TIMING_SCHEMA_VERSION,
                "sourcePath": "source.png",
                "sourceSuffix": ".png",
                "phase": "doclingConvert",
                "elapsedMs": 12.5,
                "status": "ok",
                "detail": "",
                "resourceRows": 1,
                "structureRows": 1,
            }
        ]
    )

    assert table.schema == DOCUMENT_TIMING_SCHEMA
    assert table.to_pylist()[0]["phase"] == "doclingConvert"


def test_warm_document_arrow_runtime_is_idempotent() -> None:
    warm_document_arrow_runtime()
    warm_document_arrow_runtime()


def test_extract_document_resources_can_return_error_row(tmp_path: Path) -> None:
    source = tmp_path / "broken.pdf"
    source.write_bytes(b"bad fixture")

    rows = extract_document_resources(
        source, converter=FailingConverter(), error_row=True
    )

    assert rows[0].resourceType == "error"
    assert rows[0].status == "error"
    assert "cannot parse" in rows[0].content


def test_extract_document_resources_raises_for_missing_source(tmp_path: Path) -> None:
    with pytest.raises(FileNotFoundError):
        extract_document_resources(
            tmp_path / "missing.pdf", converter=DocumentsFakeDoclingConverter()
        )


def test_full_profile_extraction_reads_cache_from_isolated_child(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    source = tmp_path / "heavy.pdf"
    source.write_bytes(b"pdf fixture")
    output_dir = tmp_path / "heavy-output"
    calls: list[tuple[Path, Path, str, bool]] = []

    def fake_isolated_extract(
        source_path: str | Path,
        output_path: str | Path,
        *,
        profile: str,
        force: bool,
    ) -> None:
        output = Path(output_path)
        calls.append((Path(source_path), output, profile, force))
        _write_cached_resources(
            output,
            [
                DocumentResourceRow(
                    sourcePath=str(source_path),
                    resourceType="document",
                    resourcePath=str(output / "heavy.md"),
                    pageIndex=0,
                    caption="",
                    content="# Heavy\n",
                    mimeType="text/markdown",
                    status="ok",
                    elementId="_main",
                )
            ],
        )

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.document_isolation.run_isolated_document_extract",
        fake_isolated_extract,
    )

    rows = extract_document_resources(source, output_dir, profile="full")

    assert calls == [(source, output_dir, "full", False)]
    assert rows[0].content == "# Heavy\n"


def test_full_profile_child_failure_returns_error_row(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    source = tmp_path / "crashy.pdf"
    source.write_bytes(b"pdf fixture")
    output_dir = tmp_path / "crashy-output"

    def fail_isolated_extract(
        source_path: str | Path,
        output_path: str | Path,
        *,
        profile: str,
        force: bool,
    ) -> None:
        _ = source_path, output_path, profile, force
        raise RuntimeError("child exited with signal 6")

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.document_isolation.run_isolated_document_extract",
        fail_isolated_extract,
    )

    rows = extract_document_resources(
        source,
        output_dir,
        profile="full",
        error_row=True,
    )

    assert rows[0].resourceType == "error"
    assert rows[0].status == "error"
    assert "child exited with signal 6" in rows[0].content
    assert (output_dir / DOCUMENT_TIMING_ARROW_CACHE_NAME).exists()


def test_run_isolated_document_extract_reports_child_exit(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    source = tmp_path / "fatal.pdf"
    output_dir = tmp_path / "fatal-output"

    def fake_run(
        command: list[str],
        **kwargs: object,
    ) -> subprocess.CompletedProcess[str]:
        assert command[-6:] == [
            "--source-path",
            str(source),
            "--output-dir",
            str(output_dir),
            "--profile",
            "full",
        ]
        assert kwargs["capture_output"] is True
        return subprocess.CompletedProcess(
            command,
            134,
            stdout="",
            stderr="fatal python error",
        )

    monkeypatch.setattr("subprocess.run", fake_run)

    with pytest.raises(RuntimeError, match="exit code 134: fatal python error"):
        run_isolated_document_extract(source, output_dir, profile="full", force=False)
