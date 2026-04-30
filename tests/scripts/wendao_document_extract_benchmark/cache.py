"""Project cache and OCR shard cache helpers."""

from __future__ import annotations

from .common import (
    Any,
    Path,
    argparse,
    os,
)
from .constants import DEFAULT_OCR_SHARD_CACHE_MAX_BYTES, OCR_SHARD_CACHE_ROOT_ENV


def resolve_project_cache_home() -> Path:
    cache_home = Path(os.environ.get("PRJ_CACHE_HOME", ".cache"))
    return cache_home.resolve()


def benchmark_ocr_shard_cache_root(args: argparse.Namespace, temp_root: Path) -> Path:
    explicit_root = getattr(args, "ocr_shard_cache_root", None)
    if explicit_root is not None:
        return explicit_root.resolve()
    configured = os.environ.get(OCR_SHARD_CACHE_ROOT_ENV)
    if configured:
        return Path(configured).resolve()
    if getattr(args, "external_endpoint", False):
        return resolve_ocr_shard_cache_root()
    return (temp_root / "ocr-shard-cache").resolve()


def resolve_ocr_shard_cache_root() -> Path:
    configured = os.environ.get(OCR_SHARD_CACHE_ROOT_ENV)
    if configured:
        return Path(configured).resolve()
    return resolve_project_cache_home() / "wendao-document-extract" / "ocr-shards"


def summarize_ocr_shard_cache(root: Path | None = None) -> dict[str, Any]:
    root = root.resolve() if root is not None else resolve_ocr_shard_cache_root()
    file_count = 0
    total_bytes = 0
    if root.exists():
        for path in root.rglob("*.arrow"):
            if not path.is_file():
                continue
            file_count += 1
            total_bytes += path.stat().st_size
    return {
        "root": str(root),
        "fileCount": file_count,
        "totalBytes": total_bytes,
        "maxBytes": optional_positive_int_env(
            "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_BYTES"
        )
        or DEFAULT_OCR_SHARD_CACHE_MAX_BYTES,
        "maxEntries": optional_positive_int_env(
            "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_ENTRIES"
        ),
        "maxAgeSecs": optional_positive_int_env(
            "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_AGE_SECS"
        ),
    }


def optional_positive_int_env(key: str) -> int | None:
    value = os.environ.get(key)
    if value is None:
        return None
    try:
        parsed = int(value)
    except ValueError:
        return None
    return parsed if parsed > 0 else None
