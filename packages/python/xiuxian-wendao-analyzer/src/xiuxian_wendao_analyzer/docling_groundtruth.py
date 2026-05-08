"""Docling upstream groundtruth comparison helpers."""

from __future__ import annotations

import difflib
import json
from pathlib import Path
from typing import Any

STRUCTURE_ARROW_NAME = "_structure.arrow"


def resolve_docling_groundtruth_root(
    *,
    explicit_root: Path | None,
    compare_enabled: bool,
    real_fixture_root: Path | None,
) -> Path | None:
    if explicit_root is not None:
        return explicit_root.resolve()
    if not compare_enabled or real_fixture_root is None:
        return None
    return (
        real_fixture_root / "tests" / "data" / "groundtruth" / "docling_v2"
    ).resolve()


def compare_report_artifacts_to_docling_groundtruth(
    *,
    enabled: bool,
    groundtruth_root: Path | None,
    report: dict[str, Any],
    min_char_coverage: float = 0.98,
    min_similarity: float = 0.98,
) -> list[dict[str, Any]]:
    if not enabled or groundtruth_root is None:
        return []
    return [
        compare_artifact_to_docling_groundtruth(
            source=Path(artifact.get("source", "")),
            output_dir=Path(artifact.get("outputDir", "")),
            groundtruth_root=groundtruth_root,
            min_char_coverage=min_char_coverage,
            min_similarity=min_similarity,
        )
        for artifact in report.get("artifactReports", [])
    ]


def compare_artifact_to_docling_groundtruth(
    *,
    source: Path,
    output_dir: Path,
    groundtruth_root: Path,
    min_char_coverage: float = 0.98,
    min_similarity: float = 0.98,
) -> dict[str, Any]:
    stem = source.stem
    markdown_path = groundtruth_root / f"{stem}.md"
    json_path = groundtruth_root / f"{stem}.json"
    if not markdown_path.exists() and not json_path.exists():
        return {
            "checked": False,
            "passed": None,
            "source": str(source),
            "outputDir": str(output_dir),
            "groundtruthRoot": str(groundtruth_root),
            "groundtruthStem": stem,
            "missingReason": f"missing Docling groundtruth for `{stem}`",
        }

    candidate_markdown = _read_candidate_markdown(source, output_dir)
    candidate_json_path = output_dir / f"{stem}.docling.json"
    groundtruth_markdown = (
        markdown_path.read_text(encoding="utf-8") if markdown_path.exists() else ""
    )
    markdown_exact = (
        _normalize_text(candidate_markdown) == _normalize_text(groundtruth_markdown)
        if markdown_path.exists() and candidate_markdown is not None
        else None
    )
    char_coverage = _ratio(
        len(candidate_markdown or ""),
        len(groundtruth_markdown),
    )
    similarity = (
        _text_similarity(groundtruth_markdown, candidate_markdown or "")
        if markdown_path.exists() and candidate_markdown is not None
        else None
    )
    json_exact = (
        _normalized_json(candidate_json_path) == _normalized_json(json_path)
        if candidate_json_path.exists() and json_path.exists()
        else None
    )
    failures = []
    if candidate_markdown is None:
        failures.append("missing candidate markdown or structure text")
    if markdown_exact is False and char_coverage < min_char_coverage:
        failures.append(
            f"markdown char coverage {char_coverage:.4f} below {min_char_coverage:.4f}"
        )
    if (
        markdown_exact is False
        and similarity is not None
        and similarity < min_similarity
    ):
        failures.append(
            f"markdown similarity {similarity:.4f} below {min_similarity:.4f}"
        )
    return {
        "checked": True,
        "passed": not failures,
        "source": str(source),
        "outputDir": str(output_dir),
        "groundtruthRoot": str(groundtruth_root),
        "groundtruthStem": stem,
        "groundtruthMarkdownPath": (
            str(markdown_path) if markdown_path.exists() else None
        ),
        "groundtruthJsonPath": str(json_path) if json_path.exists() else None,
        "candidateTextSource": _candidate_text_source(source, output_dir),
        "candidateTextChars": len(candidate_markdown or ""),
        "groundtruthMarkdownChars": len(groundtruth_markdown),
        "charCoverageRatio": char_coverage,
        "markdownSimilarity": similarity,
        "markdownExactMatch": markdown_exact,
        "jsonExactMatch": json_exact,
        "failures": failures,
    }


def summarize_docling_groundtruth_reports(
    reports: list[dict[str, Any]],
) -> dict[str, Any]:
    checked = [report for report in reports if report.get("checked")]
    missing = [report for report in reports if not report.get("checked")]
    failed = [report for report in checked if report.get("passed") is False]
    similarities = [
        value
        for report in checked
        if isinstance((value := report.get("markdownSimilarity")), int | float)
    ]
    coverages = [
        value
        for report in checked
        if isinstance((value := report.get("charCoverageRatio")), int | float)
    ]
    return {
        "checked": bool(checked),
        "checkedCount": len(checked),
        "missingCount": len(missing),
        "failureCount": len(failed),
        "passed": (not failed if checked else None),
        "minMarkdownSimilarity": min(similarities) if similarities else None,
        "minCharCoverageRatio": min(coverages) if coverages else None,
        "failures": [
            {
                "groundtruthStem": report.get("groundtruthStem"),
                "failures": report.get("failures", []),
            }
            for report in failed
        ],
    }


def _read_candidate_markdown(source: Path, output_dir: Path) -> str | None:
    markdown_path = output_dir / f"{source.stem}.md"
    if markdown_path.exists():
        return markdown_path.read_text(encoding="utf-8")
    return _read_structure_text(output_dir / STRUCTURE_ARROW_NAME)


def _candidate_text_source(source: Path, output_dir: Path) -> str | None:
    markdown_path = output_dir / f"{source.stem}.md"
    if markdown_path.exists():
        return str(markdown_path)
    structure_path = output_dir / STRUCTURE_ARROW_NAME
    if structure_path.exists():
        return str(structure_path)
    return None


def _read_structure_text(path: Path) -> str | None:
    if not path.exists():
        return None
    try:
        import pyarrow.ipc as arrow_ipc
    except ImportError:
        return None

    fragments: list[tuple[str, str]] = []
    with path.open("rb") as handle:
        reader = arrow_ipc.open_file(handle)
        for batch_index in range(reader.num_record_batches):
            batch = reader.get_batch(batch_index)
            names = set(batch.schema.names)
            if "content" not in names:
                continue
            content = batch.column(batch.schema.get_field_index("content")).to_pylist()
            if "readingOrderKey" in names:
                order = batch.column(
                    batch.schema.get_field_index("readingOrderKey")
                ).to_pylist()
            else:
                order = [
                    f"{batch_index:06d}|{row:06d}" for row in range(batch.num_rows)
                ]
            for row_order, row_content in zip(order, content, strict=True):
                if isinstance(row_content, str) and row_content.strip():
                    fragments.append((str(row_order or ""), row_content))
    if not fragments:
        return None
    return "\n\n".join(content for _, content in sorted(fragments))


def _normalized_json(path: Path) -> str | None:
    if not path.exists():
        return None
    return json.dumps(json.loads(path.read_text(encoding="utf-8")), sort_keys=True)


def _normalize_text(value: str) -> str:
    return "\n".join(
        line.rstrip() for line in value.replace("\r\n", "\n").split("\n")
    ).strip()


def _ratio(numerator: int, denominator: int) -> float:
    if denominator <= 0:
        return 1.0 if numerator == 0 else 0.0
    return numerator / denominator


def _text_similarity(expected: str, candidate: str) -> float:
    expected = _normalize_text(expected)
    candidate = _normalize_text(candidate)
    if not expected and not candidate:
        return 1.0
    return difflib.SequenceMatcher(None, expected, candidate, autojunk=False).ratio()
