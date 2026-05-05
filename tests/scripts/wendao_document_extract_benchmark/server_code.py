"""Inline Python worker source builders for local benchmark services."""

from __future__ import annotations

from .common import (
    Path,
    textwrap,
)


def real_docling_server_code(
    host: str,
    port: int,
    fixture_root: Path | None,
    include_audio: bool,
    converter_count_path: Path | None,
    pdf_ocr_worker: str = "skip",
    pdf_ocr_workers: str = "auto",
) -> str:
    fixture_root_text = str(fixture_root) if fixture_root is not None else ""
    count_path_text = (
        str(converter_count_path) if converter_count_path is not None else ""
    )
    return textwrap.dedent(
        f"""
        from pathlib import Path
        from threading import Lock

        from docling.datamodel.backend_options import XBRLBackendOptions
        from docling.datamodel.base_models import InputFormat
        from docling.datamodel.pipeline_options import PdfPipelineOptions, TableFormerMode
        from docling.document_converter import DocumentConverter, PdfFormatOption, XBRLFormatOption
        from xiuxian_wendao_analyzer.document_service import DocumentExtractFlightServer
        from xiuxian_wendao_analyzer.pdf_ocr import (
            DoclingPdfOcrShardWorker,
            PDF_OCR_FAST_TEXT_PROFILE,
        )

        fixture_root = Path({fixture_root_text!r}) if {bool(fixture_root_text)!r} else None
        CONVERTER_COUNT_PATH = Path({count_path_text!r}) if {bool(count_path_text)!r} else None
        if CONVERTER_COUNT_PATH is not None:
            CONVERTER_COUNT_PATH.parent.mkdir(parents=True, exist_ok=True)
            CONVERTER_COUNT_PATH.write_text("0", encoding="utf-8")

        class CountingConverter:
            def __init__(self, inner):
                self.inner = inner
                self.calls = 0
                self.lock = Lock()

            def convert(self, source, **kwargs):
                with self.lock:
                    self.calls += 1
                    if CONVERTER_COUNT_PATH is not None:
                        CONVERTER_COUNT_PATH.write_text(str(self.calls), encoding="utf-8")
                return self.inner.convert(source, **kwargs)

        format_options = {{}}
        if fixture_root is not None:
            taxonomy = fixture_root / "tests" / "data" / "xbrl" / "mlac-taxonomy"
            if taxonomy.exists():
                format_options[InputFormat.XML_XBRL] = XBRLFormatOption(
                    backend_options=XBRLBackendOptions(
                        enable_local_fetch=True,
                        taxonomy=taxonomy,
                    )
                )

        if {include_audio!r}:
            import os
            import shutil
            import tempfile

            try:
                import imageio_ffmpeg

                ffmpeg_path = Path(imageio_ffmpeg.get_ffmpeg_exe())
                ffmpeg_bin_dir = Path(tempfile.mkdtemp(prefix="wendao-docling-ffmpeg-"))
                ffmpeg_link = ffmpeg_bin_dir / "ffmpeg"
                try:
                    ffmpeg_link.symlink_to(ffmpeg_path)
                except OSError:
                    shutil.copy2(ffmpeg_path, ffmpeg_link)
                    ffmpeg_link.chmod(0o755)
                os.environ["PATH"] = (
                    str(ffmpeg_bin_dir)
                    + os.pathsep
                    + os.environ.get("PATH", "")
                )
            except ImportError:
                pass

            from docling.datamodel import asr_model_specs
            from docling.datamodel.pipeline_options import AsrPipelineOptions
            from docling.document_converter import AudioFormatOption
            from docling.pipeline.asr_pipeline import AsrPipeline

            audio_options = AsrPipelineOptions()
            audio_options.asr_options = asr_model_specs.WHISPER_TINY
            format_options[InputFormat.AUDIO] = AudioFormatOption(
                pipeline_cls=AsrPipeline,
                pipeline_options=audio_options,
            )

        def make_converter(ocr_profile=None):
            effective_format_options = dict(format_options)
            if ocr_profile == PDF_OCR_FAST_TEXT_PROFILE:
                pdf_options = PdfPipelineOptions()
                pdf_options.table_structure_options.mode = TableFormerMode.FAST
                effective_format_options[InputFormat.PDF] = PdfFormatOption(
                    pipeline_options=pdf_options
                )
            converter = DocumentConverter(format_options=effective_format_options)
            if CONVERTER_COUNT_PATH is not None:
                return CountingConverter(converter)
            return converter

        converter = make_converter()
        ocr_worker = None
        if {pdf_ocr_worker!r} == "docling":
            ocr_worker = DoclingPdfOcrShardWorker(
                converter_factory=make_converter,
                max_workers={pdf_ocr_workers!r},
            )
        server = DocumentExtractFlightServer(
            "grpc://{host}:{port}",
            converter=converter,
            ocr_worker=ocr_worker,
        )
        server.serve()
        """
    )


def fixture_server_code(
    host: str,
    port: int,
    converter_count_path: Path | None,
    pdf_ocr_worker: str = "skip",
    pdf_ocr_workers: str = "auto",
) -> str:
    count_path_text = (
        str(converter_count_path) if converter_count_path is not None else ""
    )
    return textwrap.dedent(
        f"""
        from pathlib import Path
        from threading import Lock
        import time
        from xiuxian_wendao_analyzer.document_service import DocumentExtractFlightServer
        from xiuxian_wendao_analyzer.pdf_ocr import succeeded_pdf_ocr_shard_result

        CONVERTER_COUNT_PATH = Path({count_path_text!r}) if {bool(count_path_text)!r} else None
        if CONVERTER_COUNT_PATH is not None:
            CONVERTER_COUNT_PATH.parent.mkdir(parents=True, exist_ok=True)
            CONVERTER_COUNT_PATH.write_text("0", encoding="utf-8")

        class Element:
            def __init__(self, text, self_ref, page_no=1):
                self.text = text
                self.self_ref = self_ref
                self.page_no = page_no

        class Document:
            def __init__(self, source):
                name = Path(source).name
                self.tables = [Element("| k | v |\\n| - | - |\\n| file | " + name + " |", "#/tables/0", 1)]
                self.pictures = [Element("fixture image " + name, "#/pictures/0", 1)]
                self.audio_segments = [Element("fixture transcript " + name, "#/audio/0", 1)]
                self.subtitles = [Element("00:00.000 --> 00:01.000\\n" + name, "#/cues/0", 1)]
            def export_to_markdown(self):
                return "# Fixture\\n\\nParsed by fake Docling converter.\\n"
            def export_to_dict(self):
                return {{"schema_name": "DoclingDocument", "fixture": True}}

        class Result:
            def __init__(self, source):
                self.document = Document(source)

        class Converter:
            def __init__(self):
                self.calls = 0
                self.lock = Lock()
            def convert(self, source, **kwargs):
                _ = kwargs
                with self.lock:
                    self.calls += 1
                    if CONVERTER_COUNT_PATH is not None:
                        CONVERTER_COUNT_PATH.write_text(str(self.calls), encoding="utf-8")
                time.sleep(0.025)
                return Result(source)

        class FixtureOcrWorker:
            def recognize(self, inputs, *, max_workers=None):
                _ = max_workers
                return [
                    succeeded_pdf_ocr_shard_result(
                        input_row,
                        "fixture OCR page " + str(input_row["pageIndex"]),
                        0.99,
                    )
                    for input_row in inputs
                ]

        ocr_worker = FixtureOcrWorker() if {pdf_ocr_worker!r} == "fixture" else None
        server = DocumentExtractFlightServer(
            "grpc://{host}:{port}",
            converter=Converter(),
            ocr_worker=ocr_worker,
        )
        server.serve()
        """
    )
