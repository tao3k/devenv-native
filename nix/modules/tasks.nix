{
  config,
  lib,
  pkgs,
  ...
}:
let
  mkPath = packages: lib.makeBinPath (lib.filter lib.isDerivation packages);

  pythonBaseEnv = [
    config.languages.python.uv.package
    config.languages.python.package
    pkgs.bash
    pkgs.coreutils
  ];

  pythonScriptEnv = pythonBaseEnv ++ [
    pkgs.just
    pkgs.findutils
    pkgs.gawk
    pkgs.gitMinimal
    pkgs.gnugrep
    pkgs.gnused
  ];

  pythonBenchmarkEnv = pythonScriptEnv ++ [
    pkgs.ripgrep
  ];

  rustBaseEnv = pythonScriptEnv ++ [
    pkgs.ripgrep
    config.languages.rust.toolchainPackage
    pkgs.clang
    pkgs.openssl
    pkgs.pkg-config
    pkgs.protobuf
    pkgs.python3
    pkgs.zlib
  ];

  # Reuse CI-relevant tool packages from global config, but exclude heavy runtime-only tools.
  ciSupportEnv = lib.filter (
    pkg:
    lib.isDerivation pkg
    && !(lib.elem (lib.getName pkg) [
      "ollama"
      "ngrok"
      "secretspec"
      "valkey"
    ])
  ) config.packages;

  hookEnv = pythonBenchmarkEnv ++ ciSupportEnv;
  pythonTaskEnv = pythonBaseEnv;
  pythonScriptTaskEnv = pythonScriptEnv;
  pythonBenchmarkTaskEnv = pythonBenchmarkEnv;
  rustTaskEnv = rustBaseEnv;
  runtimeTaskEnv = rustBaseEnv ++ [ pkgs.valkey ];

  mkTask = envPackages: command: {
    exec = command;
    env = {
      PATH = "${mkPath envPackages}:$PATH";
    };
  };

  mkRustTaskWith = envPackages: command: {
    exec = ''
      export PKG_CONFIG_PATH="${pkgs.zlib.dev}/lib/pkgconfig:${pkgs.zlib.out}/lib/pkgconfig:''${PKG_CONFIG_PATH:-}"
      ${command}
    '';
    env = {
      PATH = "${mkPath envPackages}:$PATH";
      PROTOC = "${pkgs.protobuf}/bin/protoc";
      PYO3_PYTHON = "${config.languages.python.package}/bin/python";
    };
  };

  mkRustTask = command: mkRustTaskWith rustTaskEnv command;

  mkPythonTask = command: mkTask pythonTaskEnv command;
  mkPythonScriptTask = command: mkTask pythonScriptTaskEnv command;
  mkPythonBenchmarkTask = command: mkTask pythonBenchmarkTaskEnv command;
  mkRuntimeTask = command: mkTask runtimeTaskEnv command;
in
{
  tasks = {
    "ci:rust-wendao-performance-quick" = mkRustTask ''
      just rust-wendao-performance-quick
    '';

    "ci:rust-wendao-performance-gateway-formal" = mkRustTask ''
      just rust-wendao-performance-gateway-formal
    '';

    "ci:wendao-gateway-perf-summary" = mkPythonScriptTask ''
      just wendao-gateway-perf-summary
    '';

    "ci:rust-wendao-performance-stress" = mkRustTask ''
      just rust-wendao-performance-stress
    '';

    "ci:rust-wendao-performance-bench-fast" = mkRustTask ''
      just rust-wendao-performance-bench-fast
    '';

    "ci:valkey-live" = mkRuntimeTask ''
      just valkey-live
    '';

    "dev:clean-generated" = mkTask hookEnv ''
      just clean-generated
    '';

    "dev:clean-rust" = mkRustTask ''
      just clean-rust
    '';

    "dev:clean-all" = mkRustTask ''
      just clean-all
    '';
  };
}
