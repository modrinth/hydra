{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs @ {self, ...}:
    inputs.utils.lib.eachDefaultSystem (system: let
      pkgs = import inputs.nixpkgs {inherit system;};
      fenix = inputs.fenix.packages.${system};
    in {
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          (with fenix; combine [
            minimal.rustc minimal.cargo
            default.clippy complete.rust-src
            rust-analyzer
          ])
          cargo-make
          act

          nodePackages.vscode-css-languageserver-bin
          nodePackages.vscode-html-languageserver-bin
        ];
      };
    });
}
