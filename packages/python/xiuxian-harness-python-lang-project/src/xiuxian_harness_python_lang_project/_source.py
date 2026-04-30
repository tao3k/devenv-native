"""Shared source-file helpers for deterministic harness rules."""

from __future__ import annotations

import io
import tokenize
from pathlib import Path

from python_lang_parser import SourceLocation


def path_location(path: Path) -> SourceLocation:
    """Return the first-token location for a file-level finding."""

    return SourceLocation(path=str(path), line=1, column=0)


def read_text(path: Path) -> str | None:
    """Read UTF-8 Python source when available."""

    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return None


def source_line(path: str | None, line: int) -> str | None:
    """Return one source line for compact diagnostic rendering."""

    if path is None or line < 1:
        return None
    try:
        return Path(path).read_text(encoding="utf-8").splitlines()[line - 1]
    except (OSError, IndexError, UnicodeDecodeError):
        return None


def count_effective_python_code_lines(content: str) -> int:
    """Count Python source lines with native tokenize."""

    lines: set[int] = set()
    try:
        for token in tokenize.generate_tokens(io.StringIO(content).readline):
            if token.type in _NON_CODE_TOKEN_TYPES:
                continue
            lines.add(token.start[0])
    except tokenize.TokenError:
        return 0
    return len(lines)


_NON_CODE_TOKEN_TYPES = frozenset(
    {
        tokenize.COMMENT,
        tokenize.DEDENT,
        tokenize.ENCODING,
        tokenize.ENDMARKER,
        tokenize.INDENT,
        tokenize.NEWLINE,
        tokenize.NL,
    }
)
