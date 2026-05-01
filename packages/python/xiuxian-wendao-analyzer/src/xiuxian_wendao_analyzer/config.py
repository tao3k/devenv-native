"""Analyzer-local configuration for xiuxian-wendao-analyzer."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

AnalyzerStrategy = Literal["score_rank"]


@dataclass(frozen=True, slots=True)
class AnalyzerConfig:
    """Configuration for one analyzer strategy instance."""

    strategy: AnalyzerStrategy = "score_rank"

    def __post_init__(self) -> None:
        if self.strategy != "score_rank":
            raise ValueError(f"unsupported analyzer strategy: {self.strategy}")


__all__ = ["AnalyzerConfig", "AnalyzerStrategy"]
