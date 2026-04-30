"""Native parser diagnostic helpers."""

from __future__ import annotations

from .model import PythonDiagnostic, PythonDiagnosticSeverity, SourceLocation


def compile_diagnostic(
    source: str,
    *,
    path_text: str | None,
    filename: str,
) -> PythonDiagnostic | None:
    """Return a diagnostic for compile-time syntax errors."""

    try:
        compile(source, filename, "exec", dont_inherit=True, optimize=0)
    except SyntaxError as exc:
        return diagnostic_from_parse_exception(
            exc,
            code="python.compile.invalid",
            path_text=path_text,
        )
    return None


def diagnostic_from_parse_exception(
    exc: SyntaxError | ValueError,
    *,
    code: str,
    path_text: str | None,
) -> PythonDiagnostic:
    """Convert a CPython parser exception into a compact diagnostic."""

    if isinstance(exc, SyntaxError):
        location = SourceLocation(
            path=exc.filename,
            line=exc.lineno or 1,
            column=max((exc.offset or 1) - 1, 0),
        )
        return PythonDiagnostic(
            code=code,
            severity=PythonDiagnosticSeverity.ERROR,
            message=exc.msg,
            location=location,
            source_line=(exc.text or "").rstrip("\n") or None,
        )
    return PythonDiagnostic(
        code=code,
        severity=PythonDiagnosticSeverity.ERROR,
        message=str(exc),
        location=SourceLocation(path=path_text, line=1, column=0),
    )
