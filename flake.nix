{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };
  outputs = { self, nixpkgs, ... }@inputs:
  let
    pkgs = import nixpkgs {
      system = "x86_64-linux";
    };
  in {
    # https://dev.to/misterio/how-to-package-a-rust-app-using-nix-3lh3
    defaultPackage.x86_64-linux = pkgs.rustPlatform.buildRustPackage rec {
      pname = "hyprprop";
      version = "0.0.1";
      cargoLock.lockFile = ./Cargo.lock;
      src = pkgs.lib.cleanSource ./.;

      preConfigure = ''
      export SLURP_LOCATION=${pkgs.slurp}/bin/slurp
      '';
    };
  
  };
}
