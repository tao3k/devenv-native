"""Removal guards for deleted command registration helper surfaces."""

from __future__ import annotations

import importlib
import importlib.util


def test_decorators_module_no_longer_exports_registry_helpers() -> None:
    decorators = importlib.import_module("xiuxian_foundation.api.decorators")

    assert not hasattr(decorators, "skill_command")
    assert not hasattr(decorators, "tool_command")
    assert not hasattr(decorators, "is_skill_command")
    assert not hasattr(decorators, "get_script_config")
    assert not hasattr(decorators, "get_command_metadata")
    assert not hasattr(decorators, "get_tool_annotations")
    assert not hasattr(decorators, "_skill_command_registry")
    assert not hasattr(decorators, "skill_resource")
    assert not hasattr(decorators, "is_skill_resource")
    assert not hasattr(decorators, "get_resource_config")


def test_utils_skills_no_longer_exports_skill_command_path_helper() -> None:
    skills = importlib.import_module("xiuxian_foundation.utils.skills")

    assert not hasattr(skills, "skill_command")
    assert not hasattr(skills, "current_skill_dir")
    assert not hasattr(skills, "skill_path")
    assert hasattr(skills, "skill_script")


def test_tracer_package_is_absent() -> None:
    assert importlib.util.find_spec("xiuxian_tracer") is None


def test_rag_package_is_absent() -> None:
    assert importlib.util.find_spec("xiuxian_rag") is None
