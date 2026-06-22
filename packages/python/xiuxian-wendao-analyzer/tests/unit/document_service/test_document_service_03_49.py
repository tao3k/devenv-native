"""document_service test slice 49."""

from __future__ import annotations

from xiuxian_wendao_analyzer.document_service_cli import (
    document_extract_service_main,
)
from xiuxian_wendao_analyzer.document_service_prewarm import (
    DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV,
    DOCUMENT_EXTRACT_PREWARM_PROFILE_ENV,
    DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH_ENV,
    document_extract_prewarm_page_ranges,
    prewarm_document_extract_converter_from_env,
)

from .support import FakeDoclingConverter, Path, pytest


def test_document_extract_prewarm_page_ranges_default_to_first_page() -> None:
    assert document_extract_prewarm_page_ranges(lambda _key: None) == [(1, 1)]


def test_document_extract_prewarm_page_ranges_parse_multiple_ranges() -> None:
    ranges = document_extract_prewarm_page_ranges(
        lambda key: "1,3:5" if key == DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV else None
    )

    assert ranges == [(1, 1), (3, 5)]


def test_document_extract_prewarm_page_ranges_reject_inverted_range() -> None:
    with pytest.raises(ValueError, match="1-based inclusive ranges"):
        document_extract_prewarm_page_ranges(
            lambda key: (
                "5:3" if key == DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV else None
            )
        )


def test_document_extract_prewarm_returns_none_without_source() -> None:
    def converter_factory(profile: str | None) -> FakeDoclingConverter:
        raise AssertionError(f"unexpected converter build for {profile}")

    converter = prewarm_document_extract_converter_from_env(
        converter_factory=converter_factory,
        lookup=lambda _key: None,
    )

    assert converter is None


def test_document_extract_prewarm_builds_profile_converter_and_warms_ranges(
    tmp_path: Path,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"pdf")
    converters: list[FakeDoclingConverter] = []
    profiles: list[str | None] = []

    def converter_factory(profile: str | None) -> FakeDoclingConverter:
        profiles.append(profile)
        converter = FakeDoclingConverter("# Warm\n")
        converters.append(converter)
        return converter

    env = {
        DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH_ENV: str(source),
        DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV: "1,4:6",
        DOCUMENT_EXTRACT_PREWARM_PROFILE_ENV: "structure",
    }

    converter = prewarm_document_extract_converter_from_env(
        converter_factory=converter_factory,
        lookup=env.get,
    )

    assert converter is converters[0]
    assert profiles == ["structure-text"]
    assert converters[0].calls == [source, source]
    assert converters[0].kwargs_calls == [
        {"page_range": (1, 1)},
        {"page_range": (4, 6)},
    ]


def test_document_extract_service_main_prewarms_configured_converter(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"pdf")
    converter = FakeDoclingConverter("# Warm\n")
    captured: dict[str, object] = {}

    class FakeServer:
        def __init__(self, location: str, **kwargs: object) -> None:
            captured["location"] = location
            captured["converter"] = kwargs["converter"]

        def serve(self) -> None:
            captured["served"] = True

    def converter_factory(profile: str | None) -> FakeDoclingConverter:
        captured["profile"] = profile
        return converter

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.document_service_cli.DocumentExtractFlightServer",
        FakeServer,
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.document_service_cli.new_docling_converter_for_profile",
        converter_factory,
    )
    monkeypatch.setattr(
        "sys.argv",
        ["wendao-document-extract", "--host", "127.0.0.1", "--port", "0"],
    )
    monkeypatch.setenv(DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH_ENV, str(source))
    monkeypatch.setenv(DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES_ENV, "2:3")

    assert document_extract_service_main() == 0
    assert captured == {
        "location": "grpc://127.0.0.1:0",
        "converter": converter,
        "profile": "full",
        "served": True,
    }
    assert converter.calls == [source]
    assert converter.kwargs_calls == [{"page_range": (2, 3)}]
