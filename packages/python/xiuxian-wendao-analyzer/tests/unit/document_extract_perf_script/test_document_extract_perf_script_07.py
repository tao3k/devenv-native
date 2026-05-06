"""document_extract_perf_script test slice 7."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
)


def test_pdf_render_region_shard_audit_emits_region_manifest(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        cargo_features="performance",
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        require_pdfium=False,
        pdf_render_selection="region-shards",
        pdf_render_region=[
            "pdf=0,2,72,80,540,700,000000.000002",
        ],
    )

    _command, env = benchmark.build_pdf_render_shard_audit_command(
        args,
        {"pdf": tmp_path / "sample.pdf"},
        tmp_path / "reports",
    )

    assert env["WENDAO_PDF_RENDER_SELECTION"] == "region_shards"
    regions = benchmark.json.loads(env["WENDAO_PDF_RENDER_REGIONS_JSON"])
    assert regions == [
        {
            "source": str(tmp_path / "sample.pdf"),
            "regions": [
                {
                    "pageIndex": 0,
                    "regionIndex": 2,
                    "regionBox": {
                        "left": 72.0,
                        "bottom": 80.0,
                        "right": 540.0,
                        "top": 700.0,
                    },
                    "readingOrderKey": "000000.000002",
                }
            ],
        }
    ]


def test_pdf_render_region_shard_audit_requires_all_selected_regions(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        cargo_features="performance",
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        require_pdfium=False,
        pdf_render_selection="region-shards",
        pdf_render_region=["pdf=0,0,72,72,540,700"],
    )

    try:
        benchmark.build_pdf_render_shard_audit_command(
            args,
            {
                "pdf": tmp_path / "sample.pdf",
                "other-pdf": tmp_path / "other.pdf",
            },
            tmp_path / "reports",
        )
    except SystemExit as error:
        assert "Missing --pdf-render-region" in str(error)
    else:
        raise AssertionError("missing selected fixture region should fail")


def test_pdf_render_region_rejects_non_region_selection(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        cargo_features="performance",
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        require_pdfium=False,
        pdf_render_selection="all-pages",
        pdf_render_region=["pdf=0,0,72,72,540,700"],
    )

    try:
        benchmark.build_pdf_render_shard_audit_command(
            args,
            {"pdf": tmp_path / "sample.pdf"},
            tmp_path / "reports",
        )
    except SystemExit as error:
        assert "--pdf-render-region requires" in str(error)
    else:
        raise AssertionError("region fixture on page selection should fail")


def test_hybrid_pdf_render_region_env_uses_selected_fixtures(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        hybrid_pdf_render_selection="region-shards",
        pdf_render_region=["pdf=0,4,10,20,110,220,000000.000004"],
        benchmark_fixtures={"pdf": tmp_path / "sample.pdf"},
    )

    env = benchmark.build_hybrid_pdf_render_region_env(args)

    regions = benchmark.json.loads(
        env["WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_JSON"]
    )
    assert regions == [
        {
            "source": str(tmp_path / "sample.pdf"),
            "regions": [
                {
                    "pageIndex": 0,
                    "regionIndex": 4,
                    "regionBox": {
                        "left": 10.0,
                        "bottom": 20.0,
                        "right": 110.0,
                        "top": 220.0,
                    },
                    "readingOrderKey": "000000.000004",
                }
            ],
        }
    ]


def test_hybrid_pdf_render_region_env_ignores_non_region_selection(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        hybrid_pdf_render_selection="shard-fallback-pages",
        pdf_render_region=["pdf=0,0,10,20,110,220"],
        benchmark_fixtures={"pdf": tmp_path / "sample.pdf"},
    )

    assert benchmark.build_hybrid_pdf_render_region_env(args) == {}


def test_ocr2_hybrid_profiles_auto_prepare_pdfium_runtime(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    pdfium_library = tmp_path / "libpdfium.dylib"
    pdfium_library.write_bytes(b"pdfium")
    monkeypatch.setattr(
        benchmark._pdf_render,
        "prepare_pdfium_runtime",
        lambda: pdfium_library,
    )
    args = benchmark.argparse.Namespace(
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_profile_planner="ocr2-risk-window",
        rust_pdf_ocr2_region_planner="profile-risk-window-slices",
        hybrid_pdf_render_selection="shard-fallback-pages",
    )

    assert benchmark.resolve_pdfium_library_path(args) == pdfium_library


def test_fast_source_range_profile_does_not_auto_prepare_pdfium_runtime() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_profile_planner="fast-risk-window",
        rust_pdf_ocr2_region_planner="disabled",
        hybrid_pdf_render_selection="shard-fallback-pages",
    )

    assert benchmark.resolve_pdfium_library_path(args) is None


def test_pdfium_asset_selection_covers_primary_platforms() -> None:
    benchmark = _load_benchmark_module()

    assert (
        benchmark.pdfium_asset_name(sys_platform="darwin", machine="arm64")
        == "pdfium-mac-arm64.tgz"
    )
    assert (
        benchmark.pdfium_asset_name(sys_platform="linux", machine="x86_64")
        == "pdfium-linux-x64.tgz"
    )
