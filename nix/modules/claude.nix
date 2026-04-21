{
  lib,
  config,
  pkgs,
  __inputs__,
  ...
}:
let
  system = pkgs.stdenv.hostPlatform.system;
in
{
  packages = [
    __inputs__.llm-agents.packages.${system}.claude-code
  ];
  claude.code.enable = true;
  env = {
    ANTHROPIC_BASE_URL = "https://token-plan-sgp.xiaomimimo.com/anthropic";
    ANTHROPIC_AUTH_TOKEN = config.secretspec.secrets.MIMO_API_KEY;
    # API_TIMEOUT_MS = "2000000";
    alwaysThinkingEnabled = "true";
    ANTHROPIC_MODEL = "mimo-v2-pro";
    ANTHROPIC_SMALL_FAST_MODEL = "mimo-v2-pro";
    ANTHROPIC_DEFAULT_SONNET_MODEL = "mimo-v2-pro";
    ANTHROPIC_DEFAULT_OPUS_MODEL = "mimo-v2-pro";
  };
  claude.code.hooks = {
    # PostToolUse = {
    #   command = ''
    #     bash -c 'cd "$DEVENV_ROOT" && source "$(ls -t .direnv/devenv-profile*.rc 2>/dev/null | head -1)" && lefthook run pre-commit'
    #   '';
    #   matcher = "^(Edit|MultiEdit|Write)$";
    # };
  };
  claude.code.mcpServers = {
    # Local devenv tool-runtime helper
    devenv = {
      type = "stdio";
      command = "devenv";
      args = [ "mcp" ];
      env = {
        DEVENV_ROOT = config.devenv.root;
      };
    };
    # nixos = {
    #   type = "stdio";
    #   command = "nix";
    #   args = [
    #     "run"
    #     "github:utensils/tool-runtime-nixos"
    #     "--"
    #   ];
    # };
    # MiniMax = {
    #   type = "stdio";
    #   command = "uvx";
    #   args = [ "minimax-coding-plan-tool-runtime" ];
    #   env = {
    #     MINIMAX_API_KEY = config.secretspec.secrets.MINIMAX_API_KEY;
    #     MINIMAX_TOOL_BASE_PATH = "${config.devenv.root}/.minimax-output";
    #     MINIMAX_API_HOST = "https://api.minimax.io";
    #     MINIMAX_API_RESOURCE_MODE = "url";
    #   };
    # };
    omniAgent = {
      type = "http";
      url = "http://127.0.0.1:3002/mcp";
      # command = "omni";
      # args = [
      #   "mcp"
      #   "--transport"
      #   "stdio"
      #   # "--port"
      #   # "3002"
      # ];
    };
  };
}
