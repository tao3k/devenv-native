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

  rustQualityEnv = rustBaseEnv ++ [
    pkgs.cargo-audit
    pkgs.cargo-deny
    pkgs.cargo-nextest
  ];

  rustSecurityEnv = rustBaseEnv ++ [
    pkgs.cargo-audit
    pkgs.cargo-deny
  ];

  rustGovernanceEnv = rustBaseEnv ++ [
    pkgs.cargo-semver-checks
    pkgs.cargo-machete
    pkgs.cargo-udeps
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
  rustQualityTaskEnv = rustQualityEnv;
  rustSecurityTaskEnv = rustSecurityEnv;
  rustGovernanceTaskEnv = rustGovernanceEnv;
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
  mkRustQualityTask = command: mkRustTaskWith rustQualityTaskEnv command;
  mkRustSecurityTask = command: mkRustTaskWith rustSecurityTaskEnv command;
  mkRustGovernanceTask = command: mkRustTaskWith rustGovernanceTaskEnv command;

  mkPythonTask = command: mkTask pythonTaskEnv command;
  mkPythonScriptTask = command: mkTask pythonScriptTaskEnv command;
  mkPythonBenchmarkTask = command: mkTask pythonBenchmarkTaskEnv command;
  mkRuntimeTask = command: mkTask runtimeTaskEnv command;
in
{
  tasks = {
    "ci:lint" = mkTask hookEnv ''
      just lint
    '';

    "ci:check-format" = mkTask hookEnv ''
      just check-format
    '';

    "ci:check-commits" = mkTask hookEnv ''
      just check-commits
    '';

    "ci:rust-quality-gate" = mkRustQualityTask ''
      just rust-quality-gate-ci "''${RUST_CHECK_TIMEOUT_SECS:-3600}"
    '';

    "ci:rust-security-gate" = mkRustSecurityTask ''
      just rust-security-gate
    '';

    "ci:rust-contract-dependency-governance" = mkRustGovernanceTask ''
      just rust-contract-dependency-governance
    '';

    "ci:rust-xiuxian-core-rs-lib" = mkRustTask ''
      just rust-xiuxian-core-rs-lib
    '';

    "ci:rust-xiuxian-daochang-profiles" = mkRustTask ''
      just rust-xiuxian-daochang-profiles
    '';

    "ci:rust-xiuxian-daochang-dependency-assertions" = mkRustTask ''
      just rust-xiuxian-daochang-dependency-assertions
    '';

    "ci:rust-xiuxian-daochang-backend-role-contracts" = mkRustTask ''
      just rust-xiuxian-daochang-backend-role-contracts
    '';

    "ci:rust-xiuxian-qianji-scenario-audit-contracts" = mkRustTask ''
      just rust-xiuxian-qianji-scenario-audit-contracts
    '';

    "ci:rust-xiuxian-wendao-contract-feedback-consumer" = mkRustTask ''
      just rust-xiuxian-wendao-contract-feedback-consumer
    '';

    "ci:rust-xiuxian-daochang-embedding-role-perf-medium-gate" = mkRustTask ''
      just rust-xiuxian-daochang-embedding-role-perf-medium-gate
    '';

    "ci:rust-xiuxian-daochang-embedding-role-perf-heavy-gate" = mkRustTask ''
      just rust-xiuxian-daochang-embedding-role-perf-heavy-gate
    '';

    "ci:rust-fusion-snapshots" = mkRustTask ''
      just rust-fusion-snapshots
    '';

    "ci:rust-search-perf-guard" = mkRustTask ''
      just rust-search-perf-guard
    '';

    "ci:rust-retrieval-audits" = mkRustTask ''
      just rust-retrieval-audits
    '';

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

    "ci:contract-e2e-route-test-json" = mkPythonScriptTask ''
      just contract-e2e-route-test-json
    '';

    "ci:contract-freeze" = mkPythonScriptTask ''
      just test-contract-freeze
    '';

    "ci:test-quick" = mkPythonScriptTask ''
      just test-quick
    '';

    "ci:no-inline-python-guard" = mkPythonScriptTask ''
      just no-inline-python-guard
    '';

    "ci:wendao-ppr-gate" = mkPythonScriptTask ''
      just gate-wendao-ppr
    '';

    "ci:wendao-ppr-report" = mkPythonScriptTask ''
      just gate-wendao-ppr-report
    '';

    "ci:wendao-ppr-mixed-canary" = mkPythonScriptTask ''
      just gate-wendao-ppr-mixed-canary
    '';

    "ci:wendao-ppr-report-validate" = mkPythonScriptTask ''
      just validate-wendao-ppr-reports
    '';

    "ci:wendao-ppr-gate-summary" = mkPythonScriptTask ''
      just wendao-ppr-gate-summary
    '';

    "ci:wendao-ppr-rollout-status" = mkPythonScriptTask ''
      just wendao-ppr-rollout-status
    '';

    "ci:valkey-live" = mkRuntimeTask ''
      just valkey-live
    '';

    "ci:telegram-session-isolation-rust" = mkRustTask ''
      just telegram-session-isolation-rust
    '';

    "ci:telegram-session-isolation-python" = mkPythonScriptTask ''
      just telegram-session-isolation-python
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
