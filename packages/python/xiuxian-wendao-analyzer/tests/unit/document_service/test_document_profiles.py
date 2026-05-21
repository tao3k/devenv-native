from xiuxian_wendao_analyzer.document_profiles import (
    DOCUMENT_EXTRACT_FULL_THREADS_ENV,
    DOCUMENT_EXTRACT_STRUCTURE_TEXT_PROFILE,
    document_extract_full_threads_from_env,
    normalize_document_extract_profile,
)


def test_document_extract_full_threads_accepts_only_positive_integers() -> None:
    assert document_extract_full_threads_from_env({}) is None
    assert (
        document_extract_full_threads_from_env({DOCUMENT_EXTRACT_FULL_THREADS_ENV: ""})
        is None
    )
    assert (
        document_extract_full_threads_from_env(
            {DOCUMENT_EXTRACT_FULL_THREADS_ENV: "invalid"}
        )
        is None
    )
    assert (
        document_extract_full_threads_from_env({DOCUMENT_EXTRACT_FULL_THREADS_ENV: "0"})
        is None
    )
    assert (
        document_extract_full_threads_from_env({DOCUMENT_EXTRACT_FULL_THREADS_ENV: "4"})
        == 4
    )


def test_document_extract_profile_normalizes_structure_text_aliases() -> None:
    assert (
        normalize_document_extract_profile("docling-structure-text")
        == DOCUMENT_EXTRACT_STRUCTURE_TEXT_PROFILE
    )
    assert (
        normalize_document_extract_profile("structure_text")
        == DOCUMENT_EXTRACT_STRUCTURE_TEXT_PROFILE
    )
