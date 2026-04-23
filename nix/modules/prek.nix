{
  inputs,
  __inputs__,
  pkgs,
  config,
  lib,
}:
let
  initConfigs =
    (inputs.omnibus.units.configs {
      inputs = {
        inputs = {
          nixpkgs = __inputs__.nixpkgs-latest;
          inherit (inputs.omnibus.flake.inputs) nixago;
        };
      };
    }).exports.default;

  /**
    Discover active package scopes so commit validation follows the repository
    layout without requiring a second hand-maintained package list.
  */
  listDirs =
    path:
    if builtins.pathExists path then
      lib.attrNames (lib.filterAttrs (_name: type: type == "directory") (builtins.readDir path))
    else
      [ ];

  packageScopes = listDirs ../../packages/python ++ listDirs ../../packages/rust/crates;

  infraScopes = [
    "nix"
    "docs"
    "cli"
    "deps"
    "ci"
    "data"
    "version"
    "claude"
    "git-ops"
    "git-workflow"
    "mcp"
    "inference"
    "router"
    "orchestrator"
    "skills"
    "rust"
    "core"
    "agent"
    "foundation"
  ];

  activeScopes = lib.unique (infraScopes ++ packageScopes);

  /**
    Generate `cog.toml` with this repository's changelog identity while reusing
    omnibus' shared cocogitto defaults.
  */
  cogNixago = (config.omnibus.ops.mkNixago initConfigs.nixago-cog) initConfigs.cog.default {
    data = {
      scopes = activeScopes;
      changelog = {
        path = "CHANGELOG.md";
        template = "remote";
        remote = "github.com";
        repository = "xiuxian-artisan-workshop";
        owner = "tao3k";
        authors = [
          {
            username = "gtrunsec";
            signature = "Guangtao";
          }
        ];
      };
    };
  };

  generatedHooks = [ cogNixago ];
  allowedScopeWords = lib.concatStringsSep " " activeScopes;

  /**
    `cog check` enforces `scopes`, but `cog verify --file` validates only the
    conventional-commit shape. Add the same scope gate locally for commit-msg.
  */
  cogVerifyWithScopes = pkgs.writeShellScript "cog-verify-with-scopes" ''
    set -eu

    message_file="''${1:?commit message file missing}"
    ${__inputs__.nixpkgs-latest.cocogitto}/bin/cog \
      --config ${lib.escapeShellArg (toString cogNixago.configFile)} \
      verify \
      --ignore-merge-commits \
      --ignore-fixup-commits \
      --file "$message_file"

    scope="$(${pkgs.gnused}/bin/sed -nE '1s/^[[:alnum:]_-]+\(([^()]*)\)!?: .*/\1/p' "$message_file")"
    if [ -z "$scope" ]; then
      exit 0
    fi

    allowed_scopes=${lib.escapeShellArg allowedScopeWords}
    case " $allowed_scopes " in
      *" $scope "*) ;;
      *)
        printf 'Commit scope `%s` is not allowed.\n' "$scope" >&2
        printf 'Allowed scopes: %s\n' "$allowed_scopes" >&2
        exit 1
        ;;
    esac
  '';

  cogHook = initConfigs.prek.cog.git-hooks.hooks.cocogitto-verify // {
    entry = toString cogVerifyWithScopes;
  };
in
{
  config = {
    packages = lib.flatten (map (g: g.__passthru.packages) generatedHooks);

    enterShell = lib.concatMapStringsSep "\n" (g: g.shellHook) generatedHooks;

    git-hooks = {
      hooks = {
        cocogitto-verify = cogHook;
        justfmt = initConfigs.prek.just.git-hooks.hooks.justfmt;
      };
    };
  };
}
