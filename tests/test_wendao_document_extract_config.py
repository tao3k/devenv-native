from __future__ import annotations

from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore[no-redef]


def test_root_wendao_toml_declares_document_extract_endpoint() -> None:
    config_path = Path(__file__).resolve().parents[1] / "wendao.toml"
    with config_path.open("rb") as handle:
        config = tomllib.load(handle)

    assert config["document_extract"]["endpoint"] == "http://127.0.0.1:50051"
