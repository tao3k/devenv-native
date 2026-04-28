"""Baseline analyzer strategies built on the wendao-core-lib substrate."""

from __future__ import annotations

import math
from dataclasses import dataclass
from typing import TYPE_CHECKING, Protocol

if TYPE_CHECKING:
    from .config import AnalyzerConfig


class AnalyzerStrategyProtocol(Protocol):
    """Protocol for one analyzer strategy implementation."""

    def analyze_rows(self, rows: list[dict[str, object]]) -> list[dict[str, object]]: ...


def _coerce_float(value: object, field_name: str) -> float:
    if isinstance(value, bool) or not isinstance(value, int | float):
        raise TypeError(f"{field_name} must be numeric")
    number = float(value)
    if not math.isfinite(number):
        raise ValueError(f"{field_name} must be finite")
    return number


@dataclass(frozen=True, slots=True)
class ScoreRankAnalyzer:
    """Deterministic analyzer that re-ranks rows by an existing score field."""

    config: AnalyzerConfig

    def analyze_rows(self, rows: list[dict[str, object]]) -> list[dict[str, object]]:
        ranked_rows: list[dict[str, object]] = []
        for row in rows:
            score = _coerce_float(row.get("score"), "score")
            ranked_rows.append({**row, "score": score})

        ranked_rows.sort(
            key=lambda row: (-float(row["score"]), str(row.get("doc_id") or row.get("path") or "")),
        )
        for index, row in enumerate(ranked_rows, start=1):
            row["rank"] = index
        return ranked_rows


def build_analyzer(config: AnalyzerConfig) -> AnalyzerStrategyProtocol:
    """Build the configured analyzer strategy."""

    return ScoreRankAnalyzer(config=config)


__all__ = [
    "AnalyzerStrategyProtocol",
    "ScoreRankAnalyzer",
    "build_analyzer",
]
