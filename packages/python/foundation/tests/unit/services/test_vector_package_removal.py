"""Removal tests for legacy Python vector package surfaces."""

from __future__ import annotations

import importlib
import importlib.util


def _find_spec_or_none(module_name: str) -> object | None:
    try:
        return importlib.util.find_spec(module_name)
    except ModuleNotFoundError:
        return None


def test_vector_store_modules_removed() -> None:
    importlib.invalidate_caches()
    assert _find_spec_or_none("xiuxian_foundation.services.vector.search") is None
    assert _find_spec_or_none("xiuxian_foundation.services.vector.constants") is None
    assert _find_spec_or_none("xiuxian_foundation.services.vector.models") is None
    assert _find_spec_or_none("xiuxian_foundation.services.vector.store") is None
    assert _find_spec_or_none("xiuxian_foundation.services.vector.knowledge") is None
    assert _find_spec_or_none("xiuxian_foundation.services.vector.hybrid") is None
    assert _find_spec_or_none("xiuxian_foundation.services.vector.crud") is None


def test_vector_package_module_is_absent() -> None:
    importlib.invalidate_caches()
    assert _find_spec_or_none("xiuxian_foundation.services.vector") is None


def test_retired_keyword_backend_eval_module_is_absent() -> None:
    importlib.invalidate_caches()
    assert _find_spec_or_none("xiuxian_foundation.services.keyword_eval") is None
