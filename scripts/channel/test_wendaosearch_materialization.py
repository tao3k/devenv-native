from __future__ import annotations

import os
import subprocess
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[2]
PROCESS_RUNTIME = PROJECT_ROOT / "scripts/channel/process-runtime.sh"
WENDAOSEARCH_COMMON = PROJECT_ROOT / "scripts/channel/wendaosearch-common.sh"


def _git(*args: str, cwd: Path) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=cwd,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def _create_origin_repo(tmp_path: Path) -> tuple[Path, str]:
    worktree = tmp_path / "worktree"
    origin = tmp_path / "origin.git"
    (worktree / "config" / "live").mkdir(parents=True, exist_ok=True)
    (worktree / "scripts").mkdir(parents=True, exist_ok=True)
    (worktree / "Project.toml").write_text('name = "WendaoSearch"\n', encoding="utf-8")
    (worktree / "config" / "live" / "parser_summary.toml").write_text(
        '[interface]\nhost = "127.0.0.1"\nport = 41081\n',
        encoding="utf-8",
    )
    (worktree / "scripts" / "run_parser_summary_service.jl").write_text(
        'println("parser-summary stub")\n',
        encoding="utf-8",
    )
    _git("init", cwd=worktree)
    _git("config", "user.name", "Codex", cwd=worktree)
    _git("config", "user.email", "codex@example.com", cwd=worktree)
    _git("add", ".", cwd=worktree)
    _git("commit", "-m", "seed", cwd=worktree)
    _git("branch", "-M", "main", cwd=worktree)
    rev = _git("rev-parse", "HEAD", cwd=worktree)
    _git("clone", "--bare", str(worktree), str(origin), cwd=tmp_path)
    return origin, rev


def _run_materialize(
    runtime_root: Path, repo_url: str
) -> subprocess.CompletedProcess[str]:
    command = f"""
set -euo pipefail
source "{PROCESS_RUNTIME}"
source "{WENDAOSEARCH_COMMON}"
export WENDAOSEARCH_PACKAGE_REPO_URL="{repo_url}"
wendaosearch_materialize_package_repo "{runtime_root}"
git -C "{runtime_root}/.data/WendaoSearch.jl" rev-parse HEAD
"""
    return subprocess.run(
        ["bash", "-lc", command],
        cwd=PROJECT_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


def test_wendaosearch_materialize_package_repo_clones_missing_checkout_at_default_head(
    tmp_path: Path,
) -> None:
    origin, rev = _create_origin_repo(tmp_path)
    runtime_root = tmp_path / "runtime-root"
    runtime_root.mkdir()

    result = _run_materialize(runtime_root, origin.as_uri())

    assert result.returncode == 0, result.stderr
    assert result.stdout.strip().splitlines()[-1] == rev
    assert (
        runtime_root
        / ".data"
        / "WendaoSearch.jl"
        / "config"
        / "live"
        / "parser_summary.toml"
    ).is_file()


def test_wendaosearch_materialize_package_repo_leaves_existing_git_checkout_untouched(
    tmp_path: Path,
) -> None:
    origin, rev = _create_origin_repo(tmp_path)
    runtime_root = tmp_path / "runtime-root"
    existing = runtime_root / ".data" / "WendaoSearch.jl"
    existing.mkdir(parents=True, exist_ok=True)
    _git("init", cwd=existing)
    _git("config", "user.name", "Codex", cwd=existing)
    _git("config", "user.email", "codex@example.com", cwd=existing)
    (existing / "README.md").write_text("existing checkout\n", encoding="utf-8")
    _git("add", ".", cwd=existing)
    _git("commit", "-m", "existing", cwd=existing)
    existing_rev = _git("rev-parse", "HEAD", cwd=existing)

    result = _run_materialize(runtime_root, origin.as_uri())

    assert result.returncode == 0, result.stderr
    assert result.stdout.strip().splitlines()[-1] == existing_rev
    assert _git("rev-parse", "HEAD", cwd=existing) == existing_rev


def test_wendaosearch_materialize_package_repo_rejects_existing_nongit_directory(
    tmp_path: Path,
) -> None:
    origin, rev = _create_origin_repo(tmp_path)
    runtime_root = tmp_path / "runtime-root"
    existing = runtime_root / ".data" / "WendaoSearch.jl"
    existing.mkdir(parents=True, exist_ok=True)
    (existing / "README.md").write_text("plain directory\n", encoding="utf-8")

    result = _run_materialize(runtime_root, origin.as_uri())

    assert result.returncode == 1
    assert "is not a git checkout" in result.stderr


def test_wendaosearch_launch_materializes_missing_package_checkout(
    tmp_path: Path,
) -> None:
    origin, rev = _create_origin_repo(tmp_path)
    runtime_root = tmp_path / "runtime-root"
    runtime_root.mkdir()
    fake_bin = tmp_path / "bin"
    fake_bin.mkdir()
    fake_julia = fake_bin / "julia"
    argv_log = tmp_path / "julia-argv.txt"
    fake_julia.write_text(
        '#!/usr/bin/env bash\nset -euo pipefail\nprintf \'%s\\n\' "$@" > "$FAKE_JULIA_ARGV_LOG"\n',
        encoding="utf-8",
    )
    fake_julia.chmod(0o755)

    env = dict(os.environ)
    env["PATH"] = f"{fake_bin}:/usr/bin:/bin"
    env["PRJ_ROOT"] = str(runtime_root)
    env["WENDAOSEARCH_PACKAGE_REPO_URL"] = origin.as_uri()
    env["WENDAOSEARCH_RUNTIME_DIR"] = ".run/wendaosearch"
    env["WENDAOSEARCH_CONFIG"] = ".data/WendaoSearch.jl/config/live/parser_summary.toml"
    env["WENDAOSEARCH_SCRIPT"] = "run_parser_summary_service.jl"
    env["FAKE_JULIA_ARGV_LOG"] = str(argv_log)

    result = subprocess.run(
        ["bash", str(PROJECT_ROOT / "scripts/channel/wendaosearch-launch.sh")],
        cwd=PROJECT_ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    assert (runtime_root / ".data" / "WendaoSearch.jl" / "Project.toml").is_file()
    assert (
        _git("rev-parse", "HEAD", cwd=runtime_root / ".data" / "WendaoSearch.jl") == rev
    )
    argv = argv_log.read_text(encoding="utf-8").splitlines()
    assert f"--project={runtime_root / '.data' / 'WendaoSearch.jl'}" in argv
    assert (
        str(
            runtime_root
            / ".data"
            / "WendaoSearch.jl"
            / "scripts"
            / "run_parser_summary_service.jl"
        )
        in argv
    )


def test_wendaosearch_launch_allows_explicit_julia_project(tmp_path: Path) -> None:
    origin, _rev = _create_origin_repo(tmp_path)
    runtime_root = tmp_path / "runtime-root"
    runtime_root.mkdir()
    bootstrap_env = tmp_path / "wendaosearch-env"
    bootstrap_env.mkdir()
    fake_bin = tmp_path / "bin"
    fake_bin.mkdir()
    fake_julia = fake_bin / "julia"
    argv_log = tmp_path / "julia-argv.txt"
    env_log = tmp_path / "julia-env.txt"
    fake_julia.write_text(
        "#!/usr/bin/env bash\n"
        "set -euo pipefail\n"
        'printf \'%s\\n\' "$@" > "$FAKE_JULIA_ARGV_LOG"\n'
        'printf \'%s\\n\' "${WENDAO_SEARCH_USE_ACTIVE_PROJECT:-}" > "$FAKE_JULIA_ENV_LOG"\n',
        encoding="utf-8",
    )
    fake_julia.chmod(0o755)

    env = dict(os.environ)
    env["PATH"] = f"{fake_bin}:/usr/bin:/bin"
    env["PRJ_ROOT"] = str(runtime_root)
    env["WENDAOSEARCH_PACKAGE_REPO_URL"] = origin.as_uri()
    env["WENDAOSEARCH_RUNTIME_DIR"] = ".run/wendaosearch"
    env["WENDAOSEARCH_CONFIG"] = ".data/WendaoSearch.jl/config/live/parser_summary.toml"
    env["WENDAOSEARCH_SCRIPT"] = "run_parser_summary_service.jl"
    env["WENDAOSEARCH_JULIA_PROJECT"] = str(bootstrap_env)
    env["FAKE_JULIA_ARGV_LOG"] = str(argv_log)
    env["FAKE_JULIA_ENV_LOG"] = str(env_log)

    result = subprocess.run(
        ["bash", str(PROJECT_ROOT / "scripts/channel/wendaosearch-launch.sh")],
        cwd=PROJECT_ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    argv = argv_log.read_text(encoding="utf-8").splitlines()
    assert f"--project={bootstrap_env}" in argv
    assert env_log.read_text(encoding="utf-8").strip() == "1"
