from __future__ import annotations

import pytest

from xiuxian_wendao_analyzer import AnalyzerConfig


def test_analyzer_config_defaults_to_score_rank() -> None:
    config = AnalyzerConfig()

    assert config.strategy == "score_rank"


def test_analyzer_config_accepts_score_rank_strategy() -> None:
    config = AnalyzerConfig(strategy="score_rank")

    assert config.strategy == "score_rank"


def test_analyzer_config_rejects_removed_strategies() -> None:
    with pytest.raises(ValueError, match="unsupported analyzer strategy"):
        AnalyzerConfig(strategy="linear_blend")  # type: ignore[arg-type]
