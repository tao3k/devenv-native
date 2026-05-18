"""Path confinement helpers for source-contract analyzer adapters."""

from __future__ import annotations

from pathlib import Path


def resolve_corpus_relative_path(
    *,
    corpus_root: Path,
    relative_path: str,
    task_label: str,
) -> Path:
    """Resolve a task path while keeping it confined to the corpus root.

    Raises:
        ValueError: If the task path is absolute or escapes the corpus root.
    """

    task_path = Path(relative_path)
    if task_path.is_absolute():
        raise ValueError(f"{task_label} source path must be relative to corpus root")
    resolved_root = corpus_root.resolve()
    resolved_path = (resolved_root / task_path).resolve()
    try:
        resolved_path.relative_to(resolved_root)
    except ValueError as exc:
        raise ValueError(f"{task_label} source path escapes corpus root") from exc
    return resolved_path


__all__ = ["resolve_corpus_relative_path"]
