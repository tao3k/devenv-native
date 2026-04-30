"""Compact project namespace index for agent-oriented rules."""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path
from typing import TYPE_CHECKING

from python_lang_parser import PythonSymbolKind

if TYPE_CHECKING:
    from collections.abc import Sequence

    from python_lang_parser import PythonModuleReport, SourceLocation


class AgentNamespaceSurface(StrEnum):
    """Public namespace surface categories used by agent policy rules."""

    CALLABLE = "callable"
    TYPE = "type"
    VALUE = "value"


@dataclass(frozen=True, slots=True)
class AgentNamespaceItem:
    """One parser-backed public namespace item."""

    name: str
    surface: AgentNamespaceSurface
    module_path: str | None
    module_name: str
    location: SourceLocation


def collect_agent_namespace_items(
    modules: Sequence[PythonModuleReport],
) -> tuple[AgentNamespaceItem, ...]:
    """Return project-level namespace items from parser report facts."""

    items: list[AgentNamespaceItem] = []
    for report in modules:
        if not report.is_valid:
            continue
        items.extend(_symbol_namespace_items(report))
        items.extend(_assignment_namespace_items(report))
    return tuple(items)


def _symbol_namespace_items(
    report: PythonModuleReport,
) -> tuple[AgentNamespaceItem, ...]:
    items: list[AgentNamespaceItem] = []
    for symbol in report.symbols:
        if not symbol.is_top_level or not symbol.is_public:
            continue
        surface = _symbol_surface(symbol.kind)
        if surface is None:
            continue
        items.append(
            AgentNamespaceItem(
                name=symbol.name,
                surface=surface,
                module_path=report.path,
                module_name=_module_name(report),
                location=symbol.location,
            )
        )
    return tuple(items)


def _assignment_namespace_items(
    report: PythonModuleReport,
) -> tuple[AgentNamespaceItem, ...]:
    items: list[AgentNamespaceItem] = []
    for assignment in report.assignments:
        if not assignment.is_top_level or not assignment.is_public:
            continue
        items.append(
            AgentNamespaceItem(
                name=assignment.name,
                surface=AgentNamespaceSurface.VALUE,
                module_path=report.path,
                module_name=_module_name(report),
                location=assignment.location,
            )
        )
    return tuple(items)


def _symbol_surface(kind: PythonSymbolKind) -> AgentNamespaceSurface | None:
    if kind in {PythonSymbolKind.FUNCTION, PythonSymbolKind.ASYNC_FUNCTION}:
        return AgentNamespaceSurface.CALLABLE
    if kind == PythonSymbolKind.CLASS:
        return AgentNamespaceSurface.TYPE
    return None


def _module_name(report: PythonModuleReport) -> str:
    if report.path is None:
        return "<memory>"
    return Path(report.path).stem
