"""Audio diagnostic quality input facade."""

from __future__ import annotations

from xiuxian_wendao_analyzer.audio_diagnostic_reference_inputs import (
    REFERENCE_STATUS_CANDIDATE_DRAFT,
    REFERENCE_STATUS_CURATED,
    curated_reference_rows_from_draft,
    curated_reference_rows_from_tsv,
    load_reference_transcripts,
    load_term_list,
    prompt_with_domain_terms,
    read_transcript,
    reference_candidate_draft_row_count,
)
from xiuxian_wendao_analyzer.audio_diagnostic_reference_validation import (
    validate_reference_jsonl,
)

__all__ = [
    "REFERENCE_STATUS_CANDIDATE_DRAFT",
    "REFERENCE_STATUS_CURATED",
    "curated_reference_rows_from_draft",
    "curated_reference_rows_from_tsv",
    "load_reference_transcripts",
    "load_term_list",
    "prompt_with_domain_terms",
    "read_transcript",
    "reference_candidate_draft_row_count",
    "validate_reference_jsonl",
]
