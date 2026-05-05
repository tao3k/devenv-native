"""Real and synthetic fixture discovery helpers."""

from __future__ import annotations

from .common import (
    Path,
    argparse,
    os,
    subprocess,
)
from .constants import (
    DOCLING_DATA_RELATIVE_ROOT,
    DOCLING_DEFAULT_GIT_REF,
    DOCLING_REAL_FIXTURE_PATHS,
    DOCLING_REAL_PDF_CORPUS_FIXTURE_PATHS,
)
from .fake_fixtures import write_fake_fixtures


def resolve_fixtures(
    args: argparse.Namespace,
    fixture_dir: Path,
) -> tuple[dict[str, Path], Path | None]:
    if args.fixture_suite == "fake":
        if args.real_docling:
            raise SystemExit(
                "--real-docling requires --fixture-suite docling-real and "
                "--docling-source-root so benchmark inputs are valid documents, "
                "or --fixture-suite explicit with --extra-fixture for an "
                "explicit real input"
            )
        return (
            merge_extra_fixtures(
                write_fake_fixtures(fixture_dir),
                getattr(args, "extra_fixture", []),
            ),
            None,
        )
    if args.fixture_suite == "explicit":
        fixtures = parse_extra_fixtures(getattr(args, "extra_fixture", []))
        if not fixtures:
            raise SystemExit("--fixture-suite explicit requires --extra-fixture")
        return fixtures, None

    if not args.real_docling:
        raise SystemExit("--fixture-suite docling-real requires --real-docling")
    real_fixture_root = resolve_docling_source_root(args.docling_source_root)
    if args.prepare_docling_fixtures:
        prepare_docling_fixtures(
            real_fixture_root,
            repo_url=args.docling_repo_url,
            git_ref=args.docling_git_ref,
        )
    require_docling_source_root(real_fixture_root)
    return (
        merge_extra_fixtures(
            docling_real_fixtures(
                real_fixture_root,
                include_audio=not args.skip_audio,
                include_pdf_corpus=args.include_docling_pdf_corpus,
            ),
            getattr(args, "extra_fixture", []),
        ),
        real_fixture_root,
    )


def select_fixtures(
    fixtures: dict[str, Path],
    fixture_names: list[str],
) -> dict[str, Path]:
    if not fixture_names:
        return fixtures

    missing = sorted(set(fixture_names).difference(fixtures))
    if missing:
        available = ", ".join(sorted(fixtures))
        raise SystemExit(
            "Unknown fixture(s): "
            + ", ".join(missing)
            + f"\nAvailable fixtures: {available}"
        )
    return {fixture_name: fixtures[fixture_name] for fixture_name in fixture_names}


def merge_extra_fixtures(
    fixtures: dict[str, Path],
    fixture_specs: list[str],
) -> dict[str, Path]:
    extra_fixtures = parse_extra_fixtures(fixture_specs)
    collisions = sorted(set(fixtures).intersection(extra_fixtures))
    if collisions:
        raise SystemExit(
            "Extra fixture alias collides with existing fixture(s): "
            + ", ".join(collisions)
        )
    return {**fixtures, **extra_fixtures}


def parse_extra_fixtures(fixture_specs: list[str]) -> dict[str, Path]:
    fixtures: dict[str, Path] = {}
    for fixture_spec in fixture_specs:
        fixture_name, fixture_path = parse_extra_fixture(fixture_spec)
        if fixture_name in fixtures:
            raise SystemExit(f"Duplicate extra fixture alias: {fixture_name}")
        fixtures[fixture_name] = fixture_path
    return fixtures


def parse_extra_fixture(fixture_spec: str) -> tuple[str, Path]:
    if "=" not in fixture_spec:
        raise SystemExit("--extra-fixture must use NAME=PATH syntax: " + fixture_spec)
    fixture_name, raw_path = fixture_spec.split("=", maxsplit=1)
    fixture_name = fixture_name.strip()
    raw_path = raw_path.strip()
    if not fixture_name:
        raise SystemExit("--extra-fixture alias must not be empty")
    if not raw_path:
        raise SystemExit(f"--extra-fixture path must not be empty: {fixture_name}")
    fixture_path = Path(raw_path).expanduser().resolve()
    if not fixture_path.is_file():
        raise SystemExit(f"Extra fixture path does not exist: {fixture_path}")
    return fixture_name, fixture_path


def resolve_docling_source_root(source_root: Path | None) -> Path:
    if source_root is not None:
        return source_root.resolve()
    data_home = Path(os.environ.get("PRJ_DATA_HOME", ".data"))
    return (data_home / "docling-real-fixtures").resolve()


def require_docling_source_root(root: Path) -> None:
    if not (root / DOCLING_DATA_RELATIVE_ROOT).exists():
        raise SystemExit(
            "Docling real fixture root does not contain tests/data: "
            f"{root}\nRun with --prepare-docling-fixtures to sparse clone "
            "Docling's real test attachments into the data directory."
        )


def prepare_docling_fixtures(root: Path, *, repo_url: str, git_ref: str) -> None:
    root.parent.mkdir(parents=True, exist_ok=True)
    if (root / ".git").exists():
        subprocess.run(
            ["git", "-C", str(root), "fetch", "--depth", "1", "origin", git_ref],
            check=True,
        )
        subprocess.run(["git", "-C", str(root), "checkout", "FETCH_HEAD"], check=True)
    else:
        subprocess.run(
            [
                "git",
                "clone",
                "--depth",
                "1",
                "--filter=blob:none",
                "--sparse",
                repo_url,
                str(root),
            ],
            check=True,
        )
        if git_ref != DOCLING_DEFAULT_GIT_REF:
            subprocess.run(
                ["git", "-C", str(root), "fetch", "--depth", "1", "origin", git_ref],
                check=True,
            )
            subprocess.run(
                ["git", "-C", str(root), "checkout", "FETCH_HEAD"], check=True
            )
    subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "sparse-checkout",
            "set",
            "--skip-checks",
            str(DOCLING_DATA_RELATIVE_ROOT),
        ],
        check=True,
    )


def docling_real_fixtures(
    root: Path,
    *,
    include_audio: bool,
    include_pdf_corpus: bool = False,
) -> dict[str, Path]:
    selected_paths = dict(DOCLING_REAL_FIXTURE_PATHS)
    if include_pdf_corpus:
        selected_paths.update(DOCLING_REAL_PDF_CORPUS_FIXTURE_PATHS)
    if not include_audio:
        selected_paths.pop("audio", None)

    fixtures = {
        fixture_name: root / relative_path
        for fixture_name, relative_path in selected_paths.items()
    }
    missing = [
        f"{fixture_name}: {fixture_path}"
        for fixture_name, fixture_path in fixtures.items()
        if not fixture_path.exists()
    ]
    if missing:
        raise SystemExit("Missing Docling real fixtures:\n" + "\n".join(missing))
    return fixtures
