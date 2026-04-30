"""PDFium runtime helpers for explicit raster benchmark lanes."""

from __future__ import annotations

from .cache import resolve_project_cache_home
from .common import (
    Path,
    platform,
    sys,
    tarfile,
    urllib,
)
from .constants import PDFIUM_BINARIES_BASE_URL, PDFIUM_BINARIES_RELEASE


def validate_pdfium_library_path(path: Path) -> Path:
    resolved = path.resolve()
    if not resolved.is_file():
        raise SystemExit(f"PDFium library path does not exist: {resolved}")
    return resolved


def prepare_pdfium_runtime() -> Path:
    asset_name = pdfium_asset_name()
    expected_library_name = pdfium_library_filename()
    cache_root = resolve_project_cache_home()
    release_dir = (
        cache_root
        / "wendao-document-extract"
        / "pdfium"
        / ("chromium-" + PDFIUM_BINARIES_RELEASE.split("/", maxsplit=1)[1])
    )
    target_dir = release_dir / asset_name.removesuffix(".tgz")
    existing_library = find_pdfium_library(target_dir, expected_library_name)
    if existing_library is not None:
        return existing_library

    target_dir.mkdir(parents=True, exist_ok=True)
    archive_path = release_dir / asset_name
    if not archive_path.is_file():
        download_pdfium_archive(asset_name, archive_path)
    safe_extract_tgz(archive_path, target_dir)
    library_path = find_pdfium_library(target_dir, expected_library_name)
    if library_path is None:
        raise SystemExit(
            "Downloaded PDFium archive did not contain "
            f"{expected_library_name}: {archive_path}"
        )
    return library_path


def pdfium_asset_name(
    *,
    sys_platform: str | None = None,
    machine: str | None = None,
) -> str:
    sys_platform = sys_platform or sys.platform
    machine = normalize_machine(machine or platform.machine())
    if sys_platform == "darwin":
        if machine in {"arm64", "aarch64"}:
            return "pdfium-mac-arm64.tgz"
        if machine in {"x86_64", "amd64"}:
            return "pdfium-mac-x64.tgz"
    if sys_platform.startswith("linux"):
        if machine in {"arm64", "aarch64"}:
            return "pdfium-linux-arm64.tgz"
        if machine in {"x86_64", "amd64"}:
            return "pdfium-linux-x64.tgz"
    if sys_platform.startswith("win"):
        if machine in {"arm64", "aarch64"}:
            return "pdfium-win-arm64.tgz"
        if machine in {"x86_64", "amd64"}:
            return "pdfium-win-x64.tgz"
        if machine in {"x86", "i386", "i686"}:
            return "pdfium-win-x86.tgz"
    raise SystemExit(
        "No pinned PDFium binary is configured for "
        f"platform={sys_platform} machine={machine}"
    )


def normalize_machine(machine: str) -> str:
    return machine.strip().lower().replace("-", "_")


def pdfium_library_filename(*, sys_platform: str | None = None) -> str:
    sys_platform = sys_platform or sys.platform
    if sys_platform == "darwin":
        return "libpdfium.dylib"
    if sys_platform.startswith("win"):
        return "pdfium.dll"
    return "libpdfium.so"


def download_pdfium_archive(asset_name: str, archive_path: Path) -> None:
    release = PDFIUM_BINARIES_RELEASE.replace("/", "%2F")
    url = f"{PDFIUM_BINARIES_BASE_URL}/{release}/{asset_name}"
    archive_path.parent.mkdir(parents=True, exist_ok=True)
    temporary_path = archive_path.with_suffix(archive_path.suffix + ".download")
    with (
        urllib.request.urlopen(url, timeout=60.0) as response,
        temporary_path.open("wb") as output,
    ):
        while True:
            chunk = response.read(1024 * 1024)
            if not chunk:
                break
            output.write(chunk)
    temporary_path.replace(archive_path)


def safe_extract_tgz(archive_path: Path, target_dir: Path) -> None:
    root = target_dir.resolve()
    with tarfile.open(archive_path, "r:gz") as archive:
        members = archive.getmembers()
        for member in members:
            member_target = (root / member.name).resolve()
            if root != member_target and root not in member_target.parents:
                raise SystemExit(
                    f"PDFium archive member escapes target directory: {member.name}"
                )
        try:
            archive.extractall(root, members=members, filter="data")
        except TypeError:
            archive.extractall(root, members=members)


def find_pdfium_library(root: Path, library_name: str) -> Path | None:
    if not root.exists():
        return None
    preferred = root / "lib" / library_name
    if preferred.is_file():
        return preferred.resolve()
    matches = sorted(path for path in root.rglob(library_name) if path.is_file())
    if not matches:
        return None
    return matches[0].resolve()
