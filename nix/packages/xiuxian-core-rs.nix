{
  lib,
  stdenv,
  python3Packages,
  rustPlatform,
  maturin,
  pkg-config,
  openssl,
  libiconv,
  python3,
  protobuf,
  runCommand,
  fetchzip,
  workspaceRoot,
  cargoDeps,
  version,
  ...
}:

let
  pname = "xiuxian-core-rs";
  cargoLockText = builtins.readFile (workspaceRoot + "/Cargo.lock");
  cargoLockLines = lib.splitString "\n" cargoLockText;
  cargoLockGitRev =
    repoUrl:
    let
      prefix = "source = \"git+${repoUrl}?rev=";
      matches = lib.filter (line: lib.hasPrefix prefix line) cargoLockLines;
    in
    if matches == [ ] then
      throw "failed to resolve git rev for ${repoUrl} from Cargo.lock"
    else
      builtins.elemAt (lib.splitString "#" (lib.removePrefix prefix (builtins.head matches))) 0;
  lanceRev = cargoLockGitRev "https://github.com/lancedb/lance.git";
  lanceSrc = fetchzip {
    url = "https://github.com/lancedb/lance/archive/${lanceRev}.tar.gz";
    hash = "sha256-Cp93QTsTrTkXizWYoZtFz88R3lX7+MmYN4E9JYBsyps=";
  };
  lanceVendorFixup = import ../lib/lance-vendor-fixup.nix { inherit lanceSrc; };
  # Use Nix native lib.fileset for filtering (no nix-filter dependency)
  filteredSrc = lib.fileset.toSource {
    root = workspaceRoot;
    fileset = lib.fileset.unions [
      (workspaceRoot + "/Cargo.toml")
      (workspaceRoot + "/Cargo.lock")
      (workspaceRoot + "/packages/rust/crates")
      (workspaceRoot + "/packages/rust/bindings/python")
    ];
  };
  cargoDepsWithLock = runCommand "${pname}-cargo-deps" { } ''
    mkdir -p "$out"
    cp -R ${cargoDeps}/. "$out"/
    cp ${workspaceRoot}/Cargo.lock "$out/Cargo.lock"
    ${lanceVendorFixup}
    fix_lance_vendor_dir "$out"
  '';
in
python3Packages.buildPythonPackage {
  inherit pname version;
  name = pname;
  pyproject = true;

  src = filteredSrc;

  # Use maturin to build the Rust extension module
  buildInputs = [
    openssl
    python3Packages.hatchling
    python3Packages.hatch-vcs
  ]
  ++ lib.optionals stdenv.hostPlatform.isDarwin [
    libiconv
  ];

  # Reuse the vendored cargo dependency tree as-is so offline git replacements
  # from rust-cargo-vendor remain intact for isolated Nix builds.
  cargoDeps = cargoDepsWithLock;

  build-system = [ rustPlatform.maturinBuildHook ];

  nativeBuildInputs = [
    pkg-config
    rustPlatform.cargoSetupHook
  ];

  preConfigure = ''
    mkdir -p .cargo
    append_git_sources() {
      repo_url="$1"
      revs="$(
        sed -n "s#^source = \"git+''${repo_url//./\\.}?rev=\\([^#\"]*\\).*#\\1#p" ${workspaceRoot}/Cargo.lock | sort -u
      )"
      for rev in $revs; do
        [ -n "$rev" ] || continue
        cat >> .cargo/git-sources.toml <<EOF
    [source."git+''${repo_url}?rev=''${rev}"]
    git = "''${repo_url}"
    rev = "''${rev}"
    replace-with = "vendored-sources"

    EOF
      done
    }

    : > .cargo/git-sources.toml
    append_git_sources "https://github.com/majiayu000/litellm-rs"
    append_git_sources "https://github.com/DevExzh/litchi.git"
    append_git_sources "https://github.com/J-F-Liu/lopdf"
    append_git_sources "https://github.com/tao3k/orgize"
    append_git_sources "https://github.com/firecrawl/pdf-inspector"
    append_git_sources "https://github.com/tao3k/rust-lang-project-harness"
    append_git_sources "https://github.com/lancedb/lance.git"

    cat > .cargo/config.toml <<EOF
    [source.crates-io]
    replace-with = "vendored-sources"

    EOF
    cat .cargo/git-sources.toml >> .cargo/config.toml
    if [ -n "''${CARGO_HOME:-}" ]; then
      mkdir -p "$CARGO_HOME"
      cat .cargo/git-sources.toml >> "$CARGO_HOME/config.toml"
    fi
    cat >> .cargo/config.toml <<EOF

    [source.vendored-sources]
    directory = "${cargoDepsWithLock}"
    EOF
    cd packages/rust/bindings/python
  '';

  env = {
    PYO3_PYTHON = "${python3}/bin/python3";
    PROTOC = "${protobuf}/bin/protoc";
    OPENSSL_DIR = lib.getDev openssl;
    OPENSSL_LIB_DIR = "${lib.getLib openssl}/lib";
    OPENSSL_NO_VENDOR = 1;
  }
  // lib.optionalAttrs stdenv.hostPlatform.isDarwin {
    # In isolated Nix/macOS builders, `xcrun metal` may be unavailable.
    # Skip build-time precompile and use mistralrs runtime-compilation path.
    MISTRALRS_METAL_PRECOMPILE = "0";
  };

  # Don't run tests during build
  doCheck = false;

  meta = {
    description = "Rust core bindings for Omni DevEnv Fusion";
    longDescription = ''
      High-performance Rust bindings providing core functionality for Omni DevEnv:
      - xiuxian-db-store / Lance vector-store facade: multimodal storage boundary
      - xiuxian-code-intelligence: Code-structure signals for Agent search
      - xiuxian-edit: Structural code editing
      - xiuxian-security: Security scanning
    '';
    homepage = "https://github.com/tao3k/xiuxian-artisan-workshop";
    license = with lib.licenses; [ "apache20" ];
    maintainers = with lib.maintainers; [ "tao3k" ];
    pythonPath = "${python3.sitePackages}";
  };
}
