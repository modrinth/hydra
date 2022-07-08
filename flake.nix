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
      pkgs = import inputs.nixpkgs {inherit system;};
      fenix = inputs.fenix.packages.${system};
      toolchain = with fenix; combine [
        minimal.rustc
        minimal.cargo
      ];

      naersk = inputs.naersk.lib."${system}".override {
        cargo = toolchain;
        rustc = toolchain;
      };
    in rec {
      packages = {
        hydra = pkgs.callPackage (
          { lib, enableTLS ? true }: naersk.buildPackage {
            root = ./.;
            cargoBuildOptions = old: let
              features = builtins.concatStringsSep " "
                ([] ++ lib.optional enableTLS "tls");
            in old ++ ["--features" "\"${features}\""];
            doCheck = true;
          }) {  };
        # TODO: Set up with cross toolchain to build on OSX
        docker-image = pkgs.callPackage (
          { lib
          , dockerTools
          , stdenv
          , copyPathsToStore
          , hydra ? packages.hydra
          , certs ? null # A set containing the cert.pem and key.pem file paths
          }: let
            hydra' = hydra.override {
              enableTLS = certs != null;
            };
          in dockerTools.buildImage {
            # To not be confused with Nix's Hydra CI
            name = "hydra-auth";
            tag = hydra.version;

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
