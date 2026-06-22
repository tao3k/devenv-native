{
  __inputs__,
  __nixpkgs__,
  inputs,
  pkgs,
  lib,
  ...
}:
let
  system = pkgs.stdenv.hostPlatform.system;
in
{
  packages = [
    # inputs.worktrunk.packages.${system}.worktrunk
    __inputs__.packages.backmark
    # ``__inputs__.llm-agents.packages.${system}.claudebox
    __nixpkgs__.repomix
    __nixpkgs__.ast-grep
    __nixpkgs__.spec-kit
    # __nixpkgs__.claude-code
    # __inputs__.llm-agents.packages.${system}.claude-code
    # __inputs__.llm-agents.packages.${system}.cursor-agent
    # __nixpkgs__.playwright-driver.browsers
    # (__inputs__.llm-agents.packages.${system}.codex.overrideAttrs {
    #   # src = pkgs.fetchFromGitHub {
    #   #   owner = "openai";
    #   #   repo = "codex";
    #   #   rev = "cc417c39a00f81b9c30d26ab45b0726a4887cb5e";
    #   #   sha256 = "sha256-ONSOVvDLfs8IDq4hI+XYAcMwjXSTBDIJlHB5Xwq107Q=";
    #   # };
    # })
    (__inputs__.llm-agents.packages.${system}.reasonix.overrideAttrs rec {
      # version = "0.53.0";
      # src = pkgs.fetchFromGitHub {
      #    owner = "esengine";
      #    repo = "DeepSeek-Reasonix";
      #   rev = "984e830df42ad065774df2c2a0ce06deb09dfa78";
      #   sha256 = "sha256-DzTb9V8z9R+NTjvhuLq19jWZCHEYkhyQRH8DoQOBgLo=";
      #   };
      #  npmDeps = pkgs.importNpmLock {
      #    npmRoot = src;
      #  };
      })        
    __inputs__.llm-agents.packages.${system}.gemini-cli
    __inputs__.llm-agents.packages.${system}.rtk
  ];

  env = {
    # PLAYWRIGHT_BROWSERS_PATH = "${__nixpkgs__.playwright-driver.browsers}";
    # PLAYWRIGHT_LAUNCH_OPTIONS_EXECUTABLE_PATH  = "${__nixpkgs__.playwright-driver.browsers}/chromium-1194/chrome-mac/Chromium.app/Contents/MacOS/Chromium";
  };
}
