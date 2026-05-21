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


def test_test_config_resolver_shim_re_exports_config_resolver() -> None:
    directory = Path(__file__).resolve().parent
    impl = _load_module(directory / "config_resolver.py", "config_resolver_impl_test")
    shim = _load_module(
        directory / "test_config_resolver.py", "test_config_resolver_shim_test"
    )

    required_names = (
        "telegram_webhook_secret_token",
        "telegram_webhook_bind",
        "telegram_webhook_port",
        "default_telegram_webhook_url",
        "normalize_telegram_session_partition_mode",
        "session_ids_from_runtime_log",
    )
    for name in required_names:
        assert hasattr(impl, name)
        assert hasattr(shim, name)

    assert shim.normalize_telegram_session_partition_mode("chat-user") == "chat_user"
    assert (
        shim.normalize_telegram_session_partition_mode("topic_user")
        == "chat_thread_user"
    )
    assert shim.default_telegram_webhook_url() == impl.default_telegram_webhook_url()
