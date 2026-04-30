from __future__ import annotations

import python_lang_parser as parser_api
import xiuxian_harness_python_lang_project as harness_api
import xiuxian_harness_python_lang_project.harness as harness_facade


def test_root_package_reexports_parser_fact_models() -> None:
    assert harness_api.PythonCallEffect is parser_api.PythonCallEffect
    assert harness_api.PythonExportContract is parser_api.PythonExportContract
    assert harness_api.PythonExportContractKind is parser_api.PythonExportContractKind
    assert harness_api.PythonModuleShape is parser_api.PythonModuleShape


def test_root_package_reexports_embedding_harness_surface() -> None:
    assert harness_api.PythonHarnessConfig is harness_facade.PythonHarnessConfig
    assert harness_api.PythonHarnessReport is harness_facade.PythonHarnessReport
    assert (
        harness_api.PythonProjectPolicyRulePack
        is harness_facade.PythonProjectPolicyRulePack
    )
    assert (
        harness_api.default_python_harness_config
        is harness_facade.default_python_harness_config
    )
    assert (
        harness_api.python_project_harness_test
        is harness_facade.python_project_harness_test
    )
    assert (
        harness_api.python_project_policy_rules
        is harness_facade.python_project_policy_rules
    )
    assert (
        harness_api.render_python_lang_harness
        is harness_facade.render_python_lang_harness
    )
    assert (
        harness_api.render_python_lang_harness_advice
        is harness_facade.render_python_lang_harness_advice
    )
    assert "render_python_lang_harness_advice" in harness_api.__all__
