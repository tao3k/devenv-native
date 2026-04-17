{
  inputs,
  workspaceRoot,
  self,
  ...
}:

let
  inherit (inputs)
    uv2nix
    nix-filter
    pyproject-nix
    pyproject-build-systems
    ;
in
{
  perSystem =
    {
      pkgs,
      lib,
      system,
      ...
    }:
    let
      workspace = uv2nix.lib.workspace.loadWorkspace { inherit workspaceRoot; };

      overlay = workspace.mkPyprojectOverlay {
        sourcePreference = "wheel";
      };

      pythonSets =
        let

          hacks = pkgs.callPackage pyproject-nix.build.hacks { };
          python = pkgs.python3;
          hack-overlay =
            final: prev:
            {
              torch = hacks.nixpkgsPrebuilt {
                from = pkgs.python3Packages.torchWithoutCuda;
                prev = prev.torch;
              };
              xiuxian-core-rs = hacks.nixpkgsPrebuilt {
                from = self.packages.${system}.xiuxian-core-rs-python-bindings;
                prev = prev.xiuxian-core-rs;
              };
            }
            //
              lib.optionalAttrs
                (pkgs.python3Packages ? nvidia-cufile-cu12 && prev ? nvidia-cufile-cu12)
                {
                  # Some nixpkgs snapshots do not expose this wheel; guard the
                  # override so Linux CI evaluation stays portable.
                  nvidia-cufile-cu12 = hacks.nixpkgsPrebuilt {
                    from = pkgs.python3Packages.nvidia-cufile-cu12;
                    prev = prev.nvidia-cufile-cu12;
                  };
                };
        in
        (pkgs.callPackage pyproject-nix.build.packages {
          inherit python;
        }).overrideScope
          (
            lib.composeManyExtensions [
              pyproject-build-systems.overlays.wheel
              overlay
              hack-overlay
              (final: prev: {
                # Fix pypika build with setuptools
                pypika = prev.pypika.overrideAttrs (old: {
                  nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
                    final.setuptools
                  ];
                });
                antlr4-python3-runtime = prev.antlr4-python3-runtime.overrideAttrs (old: {
                  nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
                    final.setuptools
                  ];
                });
                pylatexenc = prev.pylatexenc.overrideAttrs (old: {
                  nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
                    final.setuptools
                  ];
                });
                hatchling = prev.hatchling.overrideAttrs (old: {
                  propagatedBuildInputs = (old.propagatedBuildInputs or [ ]) ++ [
                    final.editables
                  ];
                });
              })
            ]
          );
    in
    {
      packages.default = self.packages.${system}.xiuxian-artisan-workshop-py;
      packages.xiuxian-artisan-workshop =
        (pythonSets.mkVirtualEnv "xiuxian-artisan-workshop-py" workspace.deps.default)
        .overrideAttrs
          (old: {
            venvIgnoreCollisions = [ "*" ];
            # venvIgnoreCollisions = [
            #   "lib/python${pkgs.python3.pythonVersion}/site-packages/doclayout-yolo-*"
            #   "lib/python${pkgs.python3.pythonVersion}/site-packages/ultralytics-*"
            # ];
          });
    };
}
