{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk.url = "github:nix-community/naersk";
    # TODO: Switch to unstable once copyToRoot is ready
    nickel.url = "github:tweag/nickel";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable-small";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs @ {self, ...}:
    inputs.utils.lib.eachDefaultSystem (system: let
      container-system = "x86_64-unknown-linux-musl";
      pkgs = import inputs.nixpkgs {inherit system;};
      fenix = inputs.fenix.packages.${system};

      toolchain = with fenix; combine [
        minimal.rustc
        minimal.cargo
      ];
      toolchain-cross = with fenix; combine [
        toolchain
        targets."${container-system}".latest.rust-std
      ];

      mkNaerskFor = toolchain: inputs.naersk.lib."${system}".override {
        cargo = toolchain;
        rustc = toolchain;
      };
      naersk = mkNaerskFor toolchain;
      naersk-cross = mkNaerskFor toolchain-cross;
    in rec {
      packages = {
        hydra = pkgs.callPackage (
          { lib, naersk, target ? system, enableTLS ? false }: naersk.buildPackage {
            root = ./.;
            cargoBuildOptions = old: let
              features = builtins.concatStringsSep " "
                ([] ++ lib.optional enableTLS "tls");
            in old ++ ["--features" "\"${features}\""];
            doCheck = true;
          }) { inherit naersk; };
        cross-hydra = (packages.hydra.override { naersk = naersk-cross;}).overrideAttrs
          (old: old // {
            CARGO_BUILD_TARGET = container-system;
            CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER =
              "${pkgs.pkgsCross.musl64.stdenv.cc}/bin/${container-system}-gcc";
          });
        docker-image = pkgs.callPackage (
            { lib
            , dockerTools
            , stdenv
            , copyPathsToStore
            , cacert ? pkgs.pkgsCross.musl64.cacert
            , hydra ? packages.cross-hydra
            , certs ? null # A set containing the cert.pem and key.pem file paths
            }: let
              hydra' = hydra.override {
                enableTLS = certs != null;
              };
            in dockerTools.buildImage {
              name = "hydra";
              tag = "latest";

              copyToRoot = pkgs.buildEnv {
                name = "hydra-root";
                paths = [ pkgs.cacert ];
                pathsToLink = [ "/etc" ];
              };

              config = {
                Cmd = [ "${hydra'}/bin/hydra" ];
                ExposedPorts = {
                  "8080/tcp" = {};
                };
                Env = [
                  "HYDRA_HOST=0.0.0.0"
                  "HYDRA_PORT=8080"
                ] ++ lib.optionals (certs != null) [
                  "HYDRA_CERT=${certs.cert}"
                  "HYDRA_KEY=${certs.key}"
                ];
              };
            }) {};
      };
      defaultPackage = packages.hydra;

      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          (with fenix; combine [
            toolchain
            default.clippy complete.rust-src
            rust-analyzer
          ])
          cargo-make
          act
          inputs.nickel.defaultPackage."${system}"

          nodePackages.vscode-css-languageserver-bin
          nodePackages.vscode-html-languageserver-bin
        ];
      };
    });
}
