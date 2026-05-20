{
  pkgs,
  config,
  __inputs__,
  ...
}:
let
  inherit (__inputs__) nixpkgs-latest;
  # Darwin-specific build reliability note:
  # `uv sync --reinstall-package xiuxian-core-rs` triggers maturin -> cargo in a subprocess.
  # With the default Nix-provided cargo script, some subprocess environments hit:
  # `dyld: Symbol not found: _libiconv` (via libidn2/libiconv lookup mismatch).
  # This shim keeps a stable `/usr/bin/env bash` entrypoint and delegates to the
  # toolchain cargo binary, which avoids that dynamic-link failure in practice.
  cargoShim = pkgs.writeTextFile {
    name = "cargo";
    destination = "/bin/cargo";
    executable = true;
    text = ''
      #!/usr/bin/env bash
      # Keep a stable cargo entrypoint for uv/maturin subprocess invocation on Darwin.
      if [[ "$(uname -s)" == "Darwin" && -n "''${DYLD_LIBRARY_PATH:-}" ]]; then
        IFS=':' read -r -a _dyld_parts <<< "''${DYLD_LIBRARY_PATH}"
        _kept_parts=()
        for _p in "''${_dyld_parts[@]}"; do
          [[ -z "''${_p}" ]] && continue
          if [[ "''${_p}" == */.devenv/profile/lib ]]; then
            continue
          fi
          _kept_parts+=("''${_p}")
        done

        if ((''${#_kept_parts[@]} > 0)); then
          export DYLD_LIBRARY_PATH="$(IFS=:; printf '%s' "''${_kept_parts[*]}")"
        else
          unset DYLD_LIBRARY_PATH
        fi
      fi

      # Ensure SDKROOT is available for build tools that rely on macOS SDK discovery.
      if [[ "$(uname -s)" == "Darwin" && -z "''${SDKROOT:-}" ]]; then
        if _sdkroot="$(xcrun --sdk macosx --show-sdk-path 2>/dev/null)"; then
          if [[ -n "''${_sdkroot}" ]]; then
            export SDKROOT="''${_sdkroot}"
          fi
        fi
      fi

      # Prefer the system Clang toolchain on macOS for crates that compile C/C++ code.
      if [[ -z "''${CC:-}" || "''${CC}" == "clang" ]]; then
        export CC="/usr/bin/clang"
      fi
      if [[ -z "''${CXX:-}" || "''${CXX}" == "clang++" ]]; then
        export CXX="/usr/bin/clang++"
      fi
      export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="''${CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER:-/usr/bin/clang}"
      export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER="''${CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER:-/usr/bin/clang}"

      # Prefer precompiled Metal kernels on local macOS builds.
      # In isolated environments where `metal` is unavailable (common in CI/Nix),
      # auto-fallback to runtime kernel compilation for build reliability.
      if [[ "$(uname -s)" == "Darwin" && -z "''${MISTRALRS_METAL_PRECOMPILE:-}" ]]; then
        if ! xcrun -sdk macosx metal -v >/dev/null 2>&1; then
          export MISTRALRS_METAL_PRECOMPILE=0
        fi
      fi

      exec ${config.languages.rust.toolchainPackage}/bin/cargo "$@"
    '';
  };
in
{
  packages = [
    (pkgs.hiPrio cargoShim)
    pkgs.protobuf
    pkgs.openssl
    pkgs.pkg-config
    pkgs.libidn2
    pkgs.cargo-nextest
    pkgs.cargo-audit
    pkgs.cargo-deny
    pkgs.cargo-semver-checks
    pkgs.cargo-machete
    pkgs.cargo-udeps
    pkgs.cargo-sweep
    pkgs.cargo-insta
  ];
  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
    channel = "stable";
    # Ensure rust can link python library
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  env = {
    PYO3_PYTHON = "${config.languages.python.package}/bin/python";
    PROTOC = "${pkgs.protobuf}/bin/protoc";
    # Fix PyO3 extension module linking for cargo test
    # Add Python library path for macOS and Linux
    # PYTHON_LIB_PATH = "${config.languages.python.package}/lib";
    # DYLD_LIBRARY_PATH = "${config.languages.python.package}/lib:${pkgs.openssl.out}/lib:''\${DYLD_LIBRARY_PATH:-}";
    # LD_LIBRARY_PATH = "${config.languages.python.package}/lib:${pkgs.openssl.out}/lib:''\${LD_LIBRARY_PATH:-}";
  };
}
