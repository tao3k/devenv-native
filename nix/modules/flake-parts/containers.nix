{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      config,
      system,
      ...
    }:
    let
      nix2container = inputs.nix2container.packages.${system}.nix2container;
      wendaoBin = config.nci.outputs."xiuxian-wendao".packages.release;
      daochangBin = config.nci.outputs."xiuxian-daochang".packages.release;
    in
    {
      packages = {
        "container-wendao" = nix2container.buildImage {
          name = "wendao";
          tag = "latest";
          config = {
            entrypoint = [ "${wendaoBin}/bin/wendao" ];
            env = [
              "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              "GIT_EXEC_PATH=${pkgs.git}/libexec/git-core"
              "PATH=${
                pkgs.lib.makeBinPath [
                  wendaoBin
                  pkgs.git
                ]
              }"
            ];
          };
          layers = [
            (nix2container.buildLayer {
              deps = [
                pkgs.cacert
                pkgs.git
              ];
            })
            (nix2container.buildLayer { deps = [ wendaoBin ]; })
          ];
        };
        "container-daochang" = nix2container.buildImage {
          name = "daochang";
          tag = "latest";
          config = {
            entrypoint = [ "${daochangBin}/bin/xiuxian-daochang" ];
            env = [
              "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
            ];
          };
          layers = [
            (nix2container.buildLayer { deps = [ pkgs.cacert ]; })
            (nix2container.buildLayer { deps = [ daochangBin ]; })
          ];
        };
      };
    };
}
