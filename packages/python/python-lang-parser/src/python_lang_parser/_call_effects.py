"""Native-parser call effect classification."""

from __future__ import annotations

import builtins

from .model import PythonCallEffect


def call_effect(function_name: str) -> PythonCallEffect:
    """Return the parser-recognized effect category for one call target."""

    return _BUILTIN_CALL_EFFECTS.get(function_name, PythonCallEffect.UNKNOWN)


_BUILTIN_CALL_EFFECTS = {
    builtins.print.__name__: PythonCallEffect.STANDARD_OUTPUT,
    builtins.breakpoint.__name__: PythonCallEffect.DEBUG_BREAKPOINT,
}
