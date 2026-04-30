from __future__ import annotations

import python_lang_parser as parser_api
import xiuxian_harness_python_lang_project as harness_api


def test_root_package_reexports_parser_fact_models() -> None:
    assert harness_api.PythonCallEffect is parser_api.PythonCallEffect
    assert harness_api.PythonExportContract is parser_api.PythonExportContract
    assert harness_api.PythonExportContractKind is parser_api.PythonExportContractKind
    assert harness_api.PythonModuleShape is parser_api.PythonModuleShape
