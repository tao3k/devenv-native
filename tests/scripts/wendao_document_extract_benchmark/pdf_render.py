"""PDF render shard audit command helpers."""

from __future__ import annotations

from .common import (
    Any,
    Path,
    argparse,
    json,
    subprocess,
    sys,
    tempfile,
)
from .features import cargo_features_with_pdf_render, normalize_render_selection
from .fixtures import resolve_fixtures, select_fixtures
from .pdfium import prepare_pdfium_runtime, validate_pdfium_library_path
from .runtime import rust_process_env


def run_pdf_render_shard_audit(args: argparse.Namespace, report_dir: Path) -> int:
    with tempfile.TemporaryDirectory(
        prefix="wendao-pdf-render-shard-audit-"
    ) as temp_root_text:
        fixture_dir = Path(temp_root_text) / "fixtures"
        fixture_dir.mkdir()
        fixtures, _real_fixture_root = resolve_fixtures(args, fixture_dir)
        fixtures = select_fixtures(fixtures, args.only_fixture)
        if not args.only_fixture:
            fixtures = {
                name: path
                for name, path in fixtures.items()
                if path.suffix.lower() == ".pdf"
            }
        if not fixtures:
            raise SystemExit(
                "PDF render shard audit requires at least one selected PDF fixture"
            )
        command, env_update = build_pdf_render_shard_audit_command(
            args,
            fixtures,
            report_dir.resolve(),
        )
        env = rust_process_env()
        env.update(env_update)
        subprocess.run(command, check=True, env=env)
    sys.stdout.write(
        "PDF render shard reports: "
        f"{report_dir / 'pdf_page_render_shard_manifest.json'}, "
        f"{report_dir / 'pdf_page_render_shard_manifest.md'}\n"
    )
    return 0


def build_pdf_render_shard_audit_command(
    args: argparse.Namespace,
    fixtures: dict[str, Path],
    report_dir: Path,
) -> tuple[list[str], dict[str, str]]:
    inputs = [
        {
            "name": name,
            "source": str(path),
        }
        for name, path in fixtures.items()
    ]
    command = [
        args.cargo,
        "test",
        "-p",
        "xiuxian-wendao",
        "--test",
        "wendao-validation-gate",
        "--features",
        cargo_features_with_pdf_render(args.cargo_features),
        "pdf_render_page_render_shard_manifest",
        "--",
        "--ignored",
        "--nocapture",
    ]
    env = {
        "WENDAO_PDF_RENDER_SHARD_INPUTS_JSON": json.dumps(inputs),
        "WENDAO_PDF_RENDER_SHARD_REPORT_DIR": str(report_dir),
        "WENDAO_PDF_RENDER_SELECTION": normalize_render_selection(
            args.pdf_render_selection
        ),
    }
    env.update(build_pdf_render_region_env(args, fixtures))
    pdfium_library_path = resolve_pdfium_library_path(args)
    if pdfium_library_path is not None:
        env["WENDAO_PDFIUM_LIBRARY_PATH"] = str(pdfium_library_path)
    if getattr(args, "require_pdfium", False):
        env["WENDAO_PDF_RENDER_REQUIRE_PDFIUM"] = "1"
    return command, env


def build_pdf_render_region_env(
    args: argparse.Namespace,
    fixtures: dict[str, Path],
) -> dict[str, str]:
    region_specs = getattr(args, "pdf_render_region", [])
    selection = normalize_render_selection(args.pdf_render_selection)
    if selection != "region_shards":
        if region_specs:
            raise SystemExit(
                "--pdf-render-region requires --pdf-render-selection region-shards"
            )
        return {}
    return {
        "WENDAO_PDF_RENDER_REGIONS_JSON": json.dumps(
            parse_pdf_render_regions(region_specs, fixtures)
        )
    }


def build_hybrid_pdf_render_region_env(args: argparse.Namespace) -> dict[str, str]:
    selection = normalize_render_selection(
        getattr(args, "hybrid_pdf_render_selection", "shard-fallback-pages")
    )
    region_specs = getattr(args, "pdf_render_region", [])
    if selection != "region_shards":
        return {}
    fixtures = getattr(args, "benchmark_fixtures", {})
    if not fixtures:
        raise SystemExit(
            "--hybrid-pdf-render-selection region-shards requires selected fixtures"
        )
    return {
        "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_JSON": json.dumps(
            parse_pdf_render_regions(region_specs, fixtures)
        )
    }


def parse_pdf_render_regions(
    region_specs: list[str],
    fixtures: dict[str, Path],
) -> list[dict[str, Any]]:
    if not region_specs:
        raise SystemExit(
            "--pdf-render-selection region-shards requires at least one --pdf-render-region"
        )
    regions_by_fixture: dict[str, list[dict[str, Any]]] = {
        name: [] for name in fixtures
    }
    seen_regions: set[tuple[str, int, int]] = set()
    for region_spec in region_specs:
        fixture_name, region = parse_pdf_render_region(region_spec)
        if fixture_name not in fixtures:
            available = ", ".join(sorted(fixtures))
            raise SystemExit(
                f"Unknown --pdf-render-region fixture alias: {fixture_name}\n"
                f"Available fixtures: {available}"
            )
        region_key = (
            fixture_name,
            int(region["pageIndex"]),
            int(region["regionIndex"]),
        )
        if region_key in seen_regions:
            raise SystemExit(
                "Duplicate --pdf-render-region page/region for fixture: "
                f"{fixture_name} page={region_key[1]} region={region_key[2]}"
            )
        seen_regions.add(region_key)
        regions_by_fixture[fixture_name].append(region)

    missing = sorted(
        fixture_name
        for fixture_name, regions in regions_by_fixture.items()
        if not regions
    )
    if missing:
        raise SystemExit(
            "Missing --pdf-render-region for selected fixture(s): " + ", ".join(missing)
        )
    return [
        {
            "source": str(fixtures[fixture_name]),
            "regions": regions_by_fixture[fixture_name],
        }
        for fixture_name in fixtures
    ]


def parse_pdf_render_region(region_spec: str) -> tuple[str, dict[str, Any]]:
    if "=" not in region_spec:
        raise SystemExit(
            "--pdf-render-region must use "
            "NAME=PAGE,REGION,LEFT,BOTTOM,RIGHT,TOP[,ORDER] syntax: " + region_spec
        )
    fixture_name, raw_region = region_spec.split("=", maxsplit=1)
    fixture_name = fixture_name.strip()
    if not fixture_name:
        raise SystemExit("--pdf-render-region fixture alias must not be empty")
    parts = [part.strip() for part in raw_region.split(",")]
    if len(parts) not in {6, 7}:
        raise SystemExit(
            f"--pdf-render-region requires 6 or 7 comma-separated values after NAME=: {region_spec}"
        )
    try:
        page_index = int(parts[0])
        region_index = int(parts[1])
        left = float(parts[2])
        bottom = float(parts[3])
        right = float(parts[4])
        top = float(parts[5])
    except ValueError as error:
        raise SystemExit(
            f"Invalid --pdf-render-region numeric value: {region_spec}"
        ) from error
    if page_index < 0 or region_index < 0:
        raise SystemExit(
            "--pdf-render-region page and region indexes must be non-negative: "
            + region_spec
        )
    if right <= left or top <= bottom:
        raise SystemExit(
            "--pdf-render-region bbox must satisfy right > left and top > bottom: "
            + region_spec
        )
    region: dict[str, Any] = {
        "pageIndex": page_index,
        "regionIndex": region_index,
        "regionBox": {
            "left": left,
            "bottom": bottom,
            "right": right,
            "top": top,
        },
    }
    if len(parts) == 7 and parts[6]:
        region["readingOrderKey"] = parts[6]
    return fixture_name, region


def resolve_pdfium_library_path(args: argparse.Namespace) -> Path | None:
    explicit_path = getattr(args, "pdfium_library_path", None)
    if explicit_path is not None:
        return validate_pdfium_library_path(explicit_path)
    if getattr(args, "prepare_pdfium_runtime", False) or hybrid_pdf_ocr_requires_pdfium(
        args
    ):
        return prepare_pdfium_runtime()
    return None


def hybrid_pdf_ocr_requires_pdfium(args: argparse.Namespace) -> bool:
    profile_planner = (
        str(getattr(args, "rust_pdf_ocr_profile_planner", "") or "")
        .strip()
        .replace("_", "-")
        .lower()
    )
    if profile_planner in {"ocr2-all", "ocr2-risk-window"}:
        return True
    region_planner = (
        str(getattr(args, "rust_pdf_ocr2_region_planner", "") or "")
        .strip()
        .replace("_", "-")
        .lower()
    )
    if region_planner and region_planner != "disabled":
        return True
    selection = normalize_render_selection(
        getattr(args, "hybrid_pdf_render_selection", "shard-fallback-pages")
    )
    return selection == "region_shards"
