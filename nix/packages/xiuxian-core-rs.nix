{ lib
, stdenv
, python3Packages
, rustPlatform
, maturin
, pkg-config
, openssl
, libiconv
, python3
, protobuf
, runCommand
, workspaceRoot
, cargoDeps
, version
, ...
}:

let
  pname = "xiuxian-core-rs";
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
    cat > .cargo/git-sources.toml <<EOF
    [source."git+https://github.com/tao3k/litellm-rs?branch=xiuxian"]
    git = "https://github.com/tao3k/litellm-rs"
    branch = "xiuxian"
    replace-with = "vendored-sources"

    [source."git+https://github.com/J-F-Liu/lopdf?rev=7a05512d831415b1f2b1ce522391d6beab8a1284"]
    git = "https://github.com/J-F-Liu/lopdf"
    rev = "7a05512d831415b1f2b1ce522391d6beab8a1284"
    replace-with = "vendored-sources"

    [source."git+https://github.com/tao3k/orgize?rev=b663a07fc9697ee82bac6c4995de1bc92b88ba05"]
    git = "https://github.com/tao3k/orgize"
    rev = "b663a07fc9697ee82bac6c4995de1bc92b88ba05"
    replace-with = "vendored-sources"

    [source."git+https://github.com/firecrawl/pdf-inspector?rev=63b55731337c18baf23319b73cc9780bb23ac61b"]
    git = "https://github.com/firecrawl/pdf-inspector"
    rev = "63b55731337c18baf23319b73cc9780bb23ac61b"
    replace-with = "vendored-sources"
    EOF

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
      - xiuxian-tags: Tag extraction
      - xiuxian-edit: Structural code editing
      - xiuxian-security: Security scanning
    '';
    homepage = "https://github.com/tao3k/xiuxian-artisan-workshop";
    license = with lib.licenses; [ "apache20" ];
    maintainers = with lib.maintainers; [ "tao3k" ];
    pythonPath = "${python3.sitePackages}";
  };
}
