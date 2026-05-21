from __future__ import annotations

from pathlib import Path

from wendao_core_lib.episteme_contracts.paths import episteme_root, ontology_root
from wendao_core_lib.episteme_contracts.wendao_audio_claim_rdf_pipeline_receipt.__main__ import (
    build_pipeline_receipt,
)
from wendao_core_lib.episteme_contracts.wendao_dataset_ontology.__main__ import (
    build_validation_report,
)
from wendao_core_lib.episteme_contracts.wendao_ontology_registry.__main__ import (
    build_registry,
    registry_text,
)


def repository_root() -> Path:
    return Path(__file__).resolve().parents[4]


def test_episteme_path_resolution_from_parent_package(monkeypatch):
    root = repository_root() / "wendao-episteme"
    monkeypatch.setenv("WENDAO_EPISTEME_ROOT", str(root))

    assert episteme_root() == root
    assert ontology_root() == root / "ontology"


def test_episteme_registry_compiles_from_parent_package(monkeypatch):
    monkeypatch.setenv(
        "WENDAO_EPISTEME_ROOT", str(repository_root() / "wendao-episteme")
    )

    registry = build_registry()
    rendered = registry_text(registry)

    assert registry["ontology"] == "wendao"
    assert registry["source_contract"]["manifest"] == "manifest.toml"
    assert registry["rdf_terms"]["classes"]
    assert '"source_contract"' in rendered


def test_episteme_dataset_and_pipeline_contracts_validate(monkeypatch):
    monkeypatch.setenv(
        "WENDAO_EPISTEME_ROOT", str(repository_root() / "wendao-episteme")
    )

    dataset_report = build_validation_report()
    pipeline_receipt = build_pipeline_receipt()

    assert dataset_report["passed"] is True
    assert pipeline_receipt["passed"] is True
