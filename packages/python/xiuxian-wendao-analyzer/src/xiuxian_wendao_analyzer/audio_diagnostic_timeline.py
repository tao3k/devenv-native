"""Audio diagnostic timestamp coverage helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence

    from xiuxian_wendao_analyzer.audio_diagnostic_identity import AudioChunk
    from xiuxian_wendao_analyzer.audio_diagnostic_results import AsrResult


def summarize_result_timeline_coverage(
    chunks: Sequence[AudioChunk],
    results: Sequence[AsrResult],
) -> dict[str, object]:
    """Summarize result coverage against the timestamp shard authority."""

    expected_shard_ids = [chunk.shard_id for chunk in chunks]
    expected_shard_id_set = set(expected_shard_ids)
    expected_audio_seconds = sum(chunk.duration_seconds for chunk in chunks)
    expected_start = min((chunk.start_seconds for chunk in chunks), default=None)
    expected_end = max(
        (chunk.start_seconds + chunk.duration_seconds for chunk in chunks),
        default=None,
    )
    by_backend: dict[str, dict[str, object]] = {}
    for backend in sorted({result.backend for result in results}):
        backend_results = [result for result in results if result.backend == backend]
        ok_results = [result for result in backend_results if result.status == "ok"]
        ok_shard_ids = [result.shard_id for result in ok_results if result.shard_id]
        missing_shard_ids = sorted(expected_shard_id_set - set(ok_shard_ids))
        duplicate_shard_rows = len(ok_shard_ids) - len(set(ok_shard_ids))
        covered_audio_seconds = _covered_audio_seconds(ok_results)
        order_stable = _order_stable(ok_results)
        by_backend[backend] = {
            "expectedShardRows": len(expected_shard_ids),
            "resultRows": len(backend_results),
            "okResultRows": len(ok_results),
            "failedResultRows": len(backend_results) - len(ok_results),
            "missingShardRows": len(missing_shard_ids),
            "duplicateShardRows": duplicate_shard_rows,
            "expectedAudioSeconds": expected_audio_seconds,
            "coveredAudioSeconds": covered_audio_seconds,
            "coverageRatio": (
                covered_audio_seconds / expected_audio_seconds
                if expected_audio_seconds
                else None
            ),
            "expectedStartSeconds": expected_start,
            "expectedEndSeconds": expected_end,
            "orderStable": order_stable,
            "coveragePassed": (
                len(backend_results) == len(expected_shard_ids)
                and len(ok_results) == len(expected_shard_ids)
                and not missing_shard_ids
                and duplicate_shard_rows == 0
                and order_stable
            ),
        }
    return {"timelineResultCoverageByBackend": by_backend}


def _covered_audio_seconds(results: Sequence[AsrResult]) -> float:
    intervals = sorted(
        (
            (result.start_seconds, result.start_seconds + result.duration_seconds)
            for result in results
            if result.duration_seconds > 0
        ),
        key=lambda interval: (interval[0], interval[1]),
    )
    if not intervals:
        return 0.0
    covered = 0.0
    current_start, current_end = intervals[0]
    for start, end in intervals[1:]:
        if start > current_end:
            covered += current_end - current_start
            current_start, current_end = start, end
        else:
            current_end = max(current_end, end)
    covered += current_end - current_start
    return covered


def _order_stable(results: Sequence[AsrResult]) -> bool:
    observed = [
        (result.chunk_index, result.start_seconds)
        for result in results
        if result.shard_id
    ]
    return observed == sorted(observed)
