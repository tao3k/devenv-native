from __future__ import annotations

from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[2]
FLAKE_PARTS = PROJECT_ROOT / "nix/modules/flake-parts/xiuxian-artisan-workshop.nix"
CORE_RS = PROJECT_ROOT / "nix/packages/xiuxian-core-rs.nix"
FIXUP = PROJECT_ROOT / "nix/lib/lance-vendor-fixup.nix"


def test_lance_vendor_fixup_removes_workspace_lints_and_restores_protos() -> None:
    fixup = FIXUP.read_text(encoding="utf-8")

    assert "fix_lance_vendor_dir()" in fixup
    assert "materialize_lance_vendor_crate()" in fixup
    assert "restore_lance_vendor_protos()" in fixup
    assert 'realpath "$crate_dir"' in fixup
    assert '$0 == "[lints]"' in fixup
    assert '$0 == "workspace = true"' in fixup
    assert (
        '[ -e "$crate_dir/protos" ] || [ -L "$crate_dir/protos" ] || return 0' in fixup
    )
    assert 'chmod -R u+w "$crate_dir/protos"' in fixup
    assert 'rm -rf "$crate_dir/protos"' in fixup
    assert "cp -R ${lanceSrc}/protos" in fixup
    assert '"$vendor_dir"/fsst-* "$vendor_dir"/lance-*' in fixup
    assert "for crate_name in" not in fixup
    assert "\"''${crate_name}\"-*" not in fixup


def test_nci_deps_drv_applies_lance_vendor_fixup_from_cargo_lock_rev() -> None:
    module = FLAKE_PARTS.read_text(encoding="utf-8")

    assert 'cargoLockGitRev "https://github.com/lancedb/lance.git"' in module
    assert "pkgs.fetchzip" in module
    assert "fetchFromGitHub" not in module
    assert "https://github.com/lancedb/lance/archive/${lanceRev}.tar.gz" in module
    assert "lanceVendorFixup" in module
    assert "fix_lance_vendor_dir \"''${cargoVendorDir:-$TMPDIR/nix-vendor}\"" in module
    assert "depsDrvConfig = commonProjectDepsDrvConfig;" in module


def test_python_binding_cargo_deps_applies_lance_vendor_fixup() -> None:
    package = CORE_RS.read_text(encoding="utf-8")

    assert ", fetchzip" in package
    assert "fetchFromGitHub" not in package
    assert 'cargoLockGitRev "https://github.com/lancedb/lance.git"' in package
    assert "https://github.com/lancedb/lance/archive/${lanceRev}.tar.gz" in package
    assert 'fix_lance_vendor_dir "$out"' in package
