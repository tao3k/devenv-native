{ workspaceRoot, inputs, ... }:
{
  perSystem =
    {
      pkgs,
      config,
      lib,
      ...
    }:
    let
      # The dumped Metal toolchain
      apple-metal-toolchain = pkgs.callPackage ../../packages/apple-metal-toolchain.nix { };

      # The native Nixpkgs SDK
      apple-sdk = pkgs.apple-sdk_15;

      # Combine them into a single directory that looks like /Applications/Xcode.app/Contents/Developer
      xcode-combined = pkgs.symlinkJoin {
        name = "xcode-combined";
        paths = [
          apple-metal-toolchain
          apple-sdk
        ];
      };
      commonProjectEnv = {
        PYO3_PYTHON = "${pkgs.python3}/bin/python";
        PROTOC = "${pkgs.protobuf}/bin/protoc";
        SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
        NIX_SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
      };
      commonProjectDrvConfig = {
        mkDerivation = {
          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.protobuf
          ];
          buildInputs = [
            pkgs.openssl
            pkgs.cacert
          ];
        };
        env = commonProjectEnv;
      };
    in
    {
      _module.args.apple-metal-toolchain = apple-metal-toolchain;

      nci.projects."cyber-xiuxian-workshop" = {
        path = workspaceRoot;
        export = true;
        drvConfig = commonProjectDrvConfig;
        depsDrvConfig = commonProjectDrvConfig;
      };
      # configure crates
      nci.crates = {
        # "xiuxian-llm" = {
        #   depsDrvConfig = {
        #     mkDerivation.nativeBuildInputs = lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
        #       apple-metal-toolchain
        #       pkgs.xcbuild
        #     ];
        #     mkDerivation.buildInputs = lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
        #       apple-sdk
        #     ];
        #     env = lib.optionalAttrs pkgs.stdenv.hostPlatform.isDarwin {
        #       MISTRALRS_METAL_PRECOMPILE = "1";
        #       # Point DEVELOPER_DIR to the combined symlink forest
        #       DEVELOPER_DIR = "${xcode-combined}";
        #       # Point SDKROOT to the macOS SDK within that forest
        #       SDKROOT = "${xcode-combined}/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk";
        #     };
        #   };
        # };
        "xiuxian-wendao" = {
          drvConfig.mkDerivation.nativeBuildInputs = [ pkgs.protobuf ];
          profiles.release.runTests = false;
        };
        "xiuxian-daochang" = {
          drvConfig.mkDerivation.nativeBuildInputs = [ pkgs.protobuf ];
          profiles.release.runTests = false;
        };
        "xiuxian-zhenfa" = {
          profiles.release.runTests = false;
          drvConfig.mkDerivation = {
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [
              pkgs.libxml2
              pkgs.cacert
            ];
          };
          drvConfig.env = {
            SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            NIX_SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          };
          depsDrvConfig.mkDerivation = {
            buildInputs = [
              pkgs.libxml2
              pkgs.cacert
            ];
          };
          depsDrvConfig.env = {
            SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            NIX_SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          };
        };
        "xiuxian-qianji" = {
          depsDrvConfig = {
            mkDerivation = {
              buildInputs = [
                pkgs.protobuf
                pkgs.libxml2
              ];
            };
          };
        };
        "xiuxian-lance" = {
          drvConfig.mkDerivation.nativeBuildInputs = [ pkgs.protobuf ];
          drvConfig.env.PROTOC = "${pkgs.protobuf}/bin/protoc";
          depsDrvConfig.mkDerivation.nativeBuildInputs = [ pkgs.protobuf ];
          depsDrvConfig.env.PROTOC = "${pkgs.protobuf}/bin/protoc";
        };
        "xiuxian-io" = {
          profiles.release.runTests = false;
        };
        "xiuxian-memory-engine" = {
          profiles.release.runTests = false;
        };
        "xiuxian-skills" = {
          profiles.release.runTests = false;
        };
        "xiuxian-vector" = {
          drvConfig.mkDerivation.nativeBuildInputs = [ pkgs.protobuf ];
          drvConfig.env.PROTOC = "${pkgs.protobuf}/bin/protoc";
          depsDrvConfig = {
            mkDerivation.nativeBuildInputs = [ pkgs.protobuf ];
            env.PROTOC = "${pkgs.protobuf}/bin/protoc";
          };
        };
      };

      packages.xiuxian-core-rs-python-bindings = pkgs.callPackage ../../packages/xiuxian-core-rs.nix {
        inherit workspaceRoot;
        cargoDeps =
          config.nci.outputs."xiuxian-core-rs".packages.release.config.rust-cargo-vendor.vendoredSources;
        version = config.nci.outputs."xiuxian-core-rs".packages.release.config.version;
      };
    };
}
