from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def _load_module(path: Path, module_name: str):
    spec = importlib.util.spec_from_file_location(module_name, path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def test_session_ids_from_runtime_log_infers_private_chat_chat_only_session(
    tmp_path,
) -> None:
    directory = Path(__file__).resolve().parent
    module = _load_module(
        directory / "config_resolver_runtime.py", "config_resolver_runtime_test"
    )

    log_file = tmp_path / "runtime.log"
    log_file.write_text(
        '2026-03-06 INFO Parsed message, forwarding to agent, session_key: 1304799691, chat_id: Some(1304799691), chat_title: None, chat_type: Some("private"), message_thread_id: None\n'
    )

    assert module.session_ids_from_runtime_log(log_file) == (
        1304799691,
        1304799691,
        None,
    )


def test_session_ids_from_runtime_log_keeps_chat_user_session_shape(tmp_path) -> None:
    directory = Path(__file__).resolve().parent
    module = _load_module(
        directory / "config_resolver_runtime.py",
        "config_resolver_runtime_test_chat_user",
    )

    log_file = tmp_path / "runtime.log"
    log_file.write_text(
        '2026-03-06 INFO Parsed message, forwarding to agent, session_key: 1001:2001, chat_id: Some(1001), chat_title: None, chat_type: Some("private"), message_thread_id: None\n'
    )

    assert module.session_ids_from_runtime_log(log_file) == (1001, 2001, None)
