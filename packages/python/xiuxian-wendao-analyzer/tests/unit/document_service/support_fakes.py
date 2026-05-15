"""Fake Docling and shard worker fixtures for document service tests."""

from __future__ import annotations

from pathlib import Path

from xiuxian_wendao_analyzer import (
    succeeded_audio_shard_result,
    succeeded_pdf_ocr_shard_result,
)


class FakeDoclingDocument:
    def __init__(
        self,
        markdown: str,
        *,
        markdown_by_page: dict[int, str] | None = None,
    ) -> None:
        self.markdown = markdown
        self.markdown_by_page = markdown_by_page or {}

    def export_to_markdown(self, **kwargs: object) -> str:
        page_no = kwargs.get("page_no")
        if isinstance(page_no, int) and page_no in self.markdown_by_page:
            return self.markdown_by_page[page_no]
        return self.markdown


class FakeDoclingResult:
    def __init__(
        self,
        markdown: str,
        *,
        markdown_by_page: dict[int, str] | None = None,
    ) -> None:
        self.document = FakeDoclingDocument(
            markdown,
            markdown_by_page=markdown_by_page,
        )


class FakeDoclingConverter:
    def __init__(self, markdown: str = "# Service\n") -> None:
        self.markdown = markdown
        self.calls: list[Path] = []
        self.kwargs_calls: list[dict[str, object]] = []

    def convert(self, source: str | Path, **kwargs: object) -> FakeDoclingResult:
        self.calls.append(Path(source))
        self.kwargs_calls.append(dict(kwargs))
        return FakeDoclingResult(self.markdown)


class FailingDoclingConverter:
    def convert(self, source: str | Path, **kwargs: object) -> FakeDoclingResult:
        _ = kwargs
        raise RuntimeError(f"cannot OCR {source}")


class FakePdfOcrShardWorker:
    def __init__(self) -> None:
        self.inputs: list[dict[str, object]] = []
        self.max_workers: int | str | None = None

    def recognize(
        self,
        inputs: list[dict[str, object]],
        *,
        max_workers: int | str | None = None,
    ) -> list[dict[str, object]]:
        self.inputs = list(inputs)
        self.max_workers = max_workers
        return [succeeded_pdf_ocr_shard_result(inputs[0], "page text", 0.91)]


class FakeAudioShardWorker:
    def __init__(self) -> None:
        self.inputs: list[dict[str, object]] = []
        self.max_workers: int | str | None = None

    def process(
        self,
        inputs: list[dict[str, object]],
        *,
        max_workers: int | str | None = None,
    ) -> list[dict[str, object]]:
        self.inputs = list(inputs)
        self.max_workers = max_workers
        return [succeeded_audio_shard_result(inputs[0], "audio text", 0.92)]
