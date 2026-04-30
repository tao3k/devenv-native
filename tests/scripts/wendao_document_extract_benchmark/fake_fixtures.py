"""Fake fixture generation and distinct-miss fixture selection."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .common import Path, argparse


def write_fake_fixtures(fixture_dir: Path) -> dict[str, Path]:
    fixtures = {
        "small-md": ("sample.md", b"# Sample\n\nHello\n"),
        "docx-like": ("sample.docx", b"docx fixture"),
        "image": ("scan.png", b"\x89PNG\r\n\x1a\n"),
        "audio": ("lecture.mp3", b"ID3 fixture"),
    }
    paths = {}
    for name, (filename, content) in fixtures.items():
        path = fixture_dir / filename
        path.write_bytes(content)
        paths[name] = path
    return paths


def write_distinct_fake_fixtures(fixture_dir: Path, count: int) -> dict[str, Path]:
    fixture_dir.mkdir(parents=True, exist_ok=True)
    templates = [
        ("markdown", ".md", b"# Distinct fixture\n\n"),
        ("docx", ".docx", b"distinct docx-like fixture\n"),
        ("image", ".png", b"\x89PNG\r\n\x1a\ndistinct image fixture\n"),
        ("audio", ".mp3", b"ID3 distinct audio fixture\n"),
        ("webvtt", ".vtt", b"WEBVTT\n\n00:00.000 --> 00:01.000\nfixture\n"),
        ("xml", ".xml", b'<?xml version="1.0"?><fixture/>'),
    ]
    paths = {}
    for index in range(count):
        kind, suffix, content = templates[index % len(templates)]
        name = f"distinct-{index + 1:02d}-{kind}"
        path = fixture_dir / f"{name}{suffix}"
        path.write_bytes(content + f"\ninstance={index + 1}\n".encode())
        paths[name] = path
    return paths


def prepare_distinct_miss_fixtures(
    args: argparse.Namespace,
    fixtures: dict[str, Path],
    fixture_dir: Path,
) -> dict[str, Path]:
    count = args.distinct_miss_concurrency
    if count <= 0:
        return {}
    if args.flight_mode != "async":
        raise SystemExit("--distinct-miss-concurrency requires --flight-mode async")
    if args.fixture_suite == "fake":
        return write_distinct_fake_fixtures(fixture_dir, count)
    if args.duplicate_miss_concurrency > 0:
        raise SystemExit(
            "--distinct-miss-concurrency and --duplicate-miss-concurrency should "
            "be run separately with real Docling fixtures so both remain true "
            "cold-miss probes"
        )
    if count > len(fixtures):
        raise SystemExit(
            f"--distinct-miss-concurrency requested {count} real fixtures, "
            f"but only {len(fixtures)} selected fixtures are available"
        )
    return dict(list(fixtures.items())[:count])


def distinct_miss_wait_ms(args: argparse.Namespace) -> int:
    if args.distinct_miss_wait_ms is not None:
        return max(args.distinct_miss_wait_ms, 0)
    return max(args.wait_ms, 60_000)
