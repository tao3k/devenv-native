"""document_extract_perf_script test slice 6."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
)


def test_pdf_render_shard_audit_can_pin_pdfium_runtime_path(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    pdfium_library = tmp_path / "libpdfium.dylib"
    pdfium_library.write_bytes(b"pdfium")
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        cargo_features="performance",
        pdfium_library_path=pdfium_library,
        prepare_pdfium_runtime=False,
        require_pdfium=True,
        pdf_render_selection="shard-fallback-pages",
        pdf_render_region=[],
    )

    _command, env = benchmark.build_pdf_render_shard_audit_command(
        args,
        {"pdf": tmp_path / "sample.pdf"},
        tmp_path / "reports",
    )

    assert env["WENDAO_PDFIUM_LIBRARY_PATH"] == str(pdfium_library.resolve())
    assert env["WENDAO_PDF_RENDER_REQUIRE_PDFIUM"] == "1"
    assert env["WENDAO_PDF_RENDER_SELECTION"] == "shard_fallback_pages"
