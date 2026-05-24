"""Audio diagnostic candidate report types."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class AudioCandidateSummary:
    """Compact fields used to compare audio diagnostic candidates."""

    summary_path: str
    label: str
    backend: str
    model: str
    precision_gate_passed: bool
    precision_gate_reason: str
    timeline_structure_passed: bool
    timeline_coverage_ratio: float | None
    timeline_gap_seconds: float | None
    timeline_overlap_seconds: float | None
    quality_proxy_passed: bool
    quality_proxy_reason: str
    weak_rows: int
    short_utterance_rows: int
    avg_inaudible_per_minute: float | None
    max_observed_reference_cer: float | None
    reference_coverage_rows: int
    reference_fail_rows: int
    failed_rows: int
    required_term_miss_rows: int
    diagnostic_wall_seconds: float | None
    request_cumulative_seconds: float | None
    latency_p50_seconds: float | None
    latency_p95_seconds: float | None
    transcript_chars: int
    avg_chinese_ratio: float | None
    avg_repeated_ngram_ratio: float | None

    def as_json(self) -> dict[str, object]:
        """Return a stable JSON object for reports."""

        return {
            "summaryPath": self.summary_path,
            "label": self.label,
            "backend": self.backend,
            "model": self.model,
            "precisionGatePassed": self.precision_gate_passed,
            "precisionGateReason": self.precision_gate_reason,
            "timelineStructurePassed": self.timeline_structure_passed,
            "timelineCoverageRatio": self.timeline_coverage_ratio,
            "timelineGapSeconds": self.timeline_gap_seconds,
            "timelineOverlapSeconds": self.timeline_overlap_seconds,
            "qualityProxyPassed": self.quality_proxy_passed,
            "qualityProxyReason": self.quality_proxy_reason,
            "weakRows": self.weak_rows,
            "shortUtteranceRows": self.short_utterance_rows,
            "avgInaudiblePerMinute": self.avg_inaudible_per_minute,
            "maxObservedReferenceCer": self.max_observed_reference_cer,
            "referenceCoverageRows": self.reference_coverage_rows,
            "referenceFailRows": self.reference_fail_rows,
            "failedRows": self.failed_rows,
            "requiredTermMissRows": self.required_term_miss_rows,
            "diagnosticWallSeconds": self.diagnostic_wall_seconds,
            "requestCumulativeSeconds": self.request_cumulative_seconds,
            "latencyP50Seconds": self.latency_p50_seconds,
            "latencyP95Seconds": self.latency_p95_seconds,
            "transcriptChars": self.transcript_chars,
            "avgChineseRatio": self.avg_chinese_ratio,
            "avgRepeatedNgramRatio": self.avg_repeated_ngram_ratio,
        }
