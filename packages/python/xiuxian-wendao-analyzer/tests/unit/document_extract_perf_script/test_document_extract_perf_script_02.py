"""document_extract_perf_script test slice 2."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
    tomllib,
)


def test_parse_extra_fixtures_rejects_missing_files(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()

    try:
        benchmark.parse_extra_fixtures([f"arxiv-2604-17337={tmp_path / 'missing.pdf'}"])
    except SystemExit as error:
        assert "Extra fixture path does not exist" in str(error)
    else:
        raise AssertionError("missing extra fixture should fail")


def test_merge_extra_fixtures_rejects_alias_collision(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    pdf_fixture = tmp_path / "sample.pdf"
    pdf_fixture.write_bytes(b"%PDF")

    try:
        benchmark.merge_extra_fixtures(
            {"pdf": tmp_path / "base.pdf"},
            [f"pdf={pdf_fixture}"],
        )
    except SystemExit as error:
        assert "collides with existing fixture" in str(error)
    else:
        raise AssertionError("colliding extra fixture should fail")


def test_prepare_distinct_miss_fixtures_writes_unique_fake_inputs(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        distinct_miss_concurrency=4,
        duplicate_miss_concurrency=0,
        fixture_suite="fake",
        flight_mode="async",
    )

    fixtures = benchmark.prepare_distinct_miss_fixtures(
        args,
        {},
        tmp_path / "distinct-fixtures",
    )

    assert list(fixtures) == [
        "distinct-01-markdown",
        "distinct-02-docx",
        "distinct-03-image",
        "distinct-04-audio",
    ]
    assert len({path.read_bytes() for path in fixtures.values()}) == 4


def test_document_extras_cover_xbrl_and_audio_asr() -> None:
    package_root = Path(__file__).resolve().parents[3]
    pyproject = tomllib.loads((package_root / "pyproject.toml").read_text())
    optional_dependencies = pyproject["project"]["optional-dependencies"]

    assert "docling[xbrl]>=2.70.0" in optional_dependencies["documents"]
    assert "docling[xbrl]>=2.70.0" in optional_dependencies["documents-audio"]
    assert "openai-whisper>=20250625" in optional_dependencies["documents-audio"]
    assert "imageio-ffmpeg>=0.6.0" in optional_dependencies["documents-audio"]


def test_docling_real_fixture_root_defaults_to_prj_data_home(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    monkeypatch.setenv("PRJ_DATA_HOME", str(tmp_path))

    assert (
        benchmark.resolve_docling_source_root(None)
        == (tmp_path / "docling-real-fixtures").resolve()
    )


def test_prepare_docling_fixtures_uses_sparse_checkout(
    monkeypatch, tmp_path: Path
) -> None:
    benchmark = _load_benchmark_module()
    commands: list[list[str]] = []

    def fake_run(command: list[str], *, check: bool) -> None:
        commands.append(command)
        assert check

    monkeypatch.setattr(benchmark.subprocess, "run", fake_run)

    benchmark.prepare_docling_fixtures(
        tmp_path / "docling-real-fixtures",
        repo_url="https://example.test/docling.git",
        git_ref=benchmark.DOCLING_DEFAULT_GIT_REF,
    )

    assert commands[0][:5] == ["git", "clone", "--depth", "1", "--filter=blob:none"]
    assert "--sparse" in commands[0]
    assert commands[1][-2:] == ["--skip-checks", "tests/data"]
