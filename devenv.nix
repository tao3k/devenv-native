{ pkgs
, lib
, config
, inputs
, ...
}:

let
  processModuleStamp = builtins.readFile ./nix/modules/process.nix;
  processModuleStampHash = builtins.hashString "sha256" processModuleStamp;
  prekhookModuleStamp = builtins.readFile ./nix/modules/prek.nix;
  prekhookModuleStampHash = builtins.hashString "sha256" prekhookModuleStamp;
  nixpkgs-latest = import inputs.nixpkgs-latest {
    system = pkgs.stdenv.hostPlatform.system;
    config = {
      allowUnfree = true;
    };
  };
  nixosModules =
    (inputs.omnibus.pops.nixosProfiles.addLoadExtender {
      load = {
        src = ./nix/modules;
        inputs = {
          __nixpkgs__ = nixpkgs-latest;
          __inputs__ = {
            inherit (inputs) llm-agents;
            inherit nixpkgs-latest packages;
          };
          inputs = {
            nixpkgs = nixpkgs-latest;
          };
        };
      };
    }).exports.default;

  packages =
    (inputs.omnibus.pops.packages.addLoadExtender {
      load = {
        src = ./nix/packages;
        inputs = {
          inputs = {
            nixpkgs = nixpkgs-latest;
          };
        };
      };
    }).exports.packages;
in
{
  imports = [
    nixosModules.claude
    nixosModules.flake-parts.omnibus
    nixosModules.files
    nixosModules.prek
    nixosModules.python
    nixosModules.llm
    nixosModules.rust
    nixosModules.packages
    nixosModules.tasks
    nixosModules.process
    #./modules/flake-parts/omnibus-hive.nix
    ({
      config = lib.mkMerge [
        {
          omnibus = {
            inputs = {
              inputs = {
                nixpkgs = pkgs;
                inherit nixpkgs-latest;
                inherit (inputs.omnibus.flake.inputs) nixago;
              };
            };
          };
        }
      ];
    })
  ];

  devcontainer.enable = true;
  # https://devenv.sh/basics/
  env.GREET = "devenv";
  # devenv.warnOnNewVersion = false;
  # https://devenv.sh/packages/
  packages = [
    pkgs.ollama
    pkgs.valkey
    pkgs.ngrok
    pkgs.nodejs
    pkgs.tree
    pkgs.duckdb
    pkgs.asciinema
    pkgs.ffmpeg
    nixpkgs-latest.jujutsu
  ];

  dotenv.enable = true;
  dotenv.filename = [ ".env" ];
  # https://devenv.sh/processes/
  # processes.cargo-watch.exec = "cargo-watch";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
  scripts.hello.exec = ''
    echo hello from $GREET
  '';
  scripts.wendao-client.exec = ''
    set -euo pipefail

    root="''${PRJ_ROOT:-''${DEVENV_ROOT:-$(pwd)}}"
    installed="$root/.devenv/state/cargo-install/bin/wendao-client"

    if [ ! -x "$installed" ]; then
      echo "wendao-client is not installed. Run: direnv exec . just install-wendao-client" >&2
      exit 127
    fi

    exec "$installed" "$@"
  '';

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  enterShell = ''
    # process-module-stamp: ${processModuleStampHash}
    # prekhook-module-stamp: ${prekhookModuleStampHash}
    export PATH="$DEVENV_ROOT/.devenv/state/cargo-install/bin:$DEVENV_ROOT/.devenv/profile/bin:$DEVENV_ROOT/.venv/bin:$PATH"
    export OLLAMA_MODELS="''${OLLAMA_MODELS:-''${PRJ_DATA_HOME:-$DEVENV_ROOT/.data}/models}"
    ${lib.optionalString (pkgs.stdenv.hostPlatform.isDarwin) ''
      unset SDKROOT
      export PATH="/usr/bin:/usr/sbin:$PATH"
      export SDKROOT="/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"
      export DEVELOPER_DIR="/Applications/Xcode.app/Contents/Developer"
    ''}
  '';

  cachix.pull = [ "tao3k" ];
  # cachix.push = "tao3k";
  # https://devenv.sh/tests/
  enterTest = "";

  # https://devenv.sh/pre-commit-hooks/
  git-hooks.hooks = {
    ruff.enable = true;
    rustfmt.enable = true;
    clippy.enable = true;
    prettier.enable = true;
    clippy.packageOverrides.cargo = config.languages.rust.toolchainPackage;
    clippy.packageOverrides.clippy = config.languages.rust.toolchainPackage;
    clippy.settings.allFeatures = true;
    oxlint.enable = true;
    oxfmt.enable = true;
  };
  # git-hooks.hooks.nixfmt.enable = true;
  # See full reference at https://devenv.sh/reference/options/
}
