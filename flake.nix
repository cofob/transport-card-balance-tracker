{
  description = "Transport card balance tracker";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    import-cargo.url = github:edolstra/import-cargo;

    bp.url = "github:serokell/nix-npm-buildpackage";
    bp.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, import-cargo, bp, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = pkgs.rust-bin.stable.latest.default;
        inherit (import-cargo.builders) importCargo;
        bp' = pkgs.callPackage bp {};

        nativeBuildInputs = with pkgs; [
          pkg-config
        ] ++ [
          rustVersion
        ];

        buildInputs = with pkgs; [ ];

        card-tracker-backend = pkgs.stdenv.mkDerivation {
          name = "card-tracker-backend";
          src = "${self}/backend";

          inherit buildInputs;

          nativeBuildInputs = [
            (importCargo { lockFile = ./backend/Cargo.lock; inherit pkgs; }).cargoHome
          ] ++ nativeBuildInputs;

          buildPhase = ''
            cargo build --release --offline
          '';

          installPhase = ''
            install -Dm775 ./target/release/backend $out/bin/backend
          '';
        };

        card-tracker-frontend = bp'.buildNpmPackage {
          pname = "card-tracker-frontend";
          version = "0.1.0";

          src = ./frontend;

          nodejs = pkgs.nodejs;

          buildPhase = ''
            npm run build
          '';

          installPhase = ''
            cp -r build $out
            cp -r node_modules $out/
            cp package.json $out/

            mkdir $out/bin
            cat <<EOF > $out/bin/frontend
              #!${pkgs.bash}/bin/bash
              ${pkgs.nodejs}/bin/node $out/index.js
            EOF
            chmod +x $out/bin/frontend
          '';
        };
      in
      {
        packages = {
          card-tracker-backend = card-tracker-backend;
          card-tracker-frontend = card-tracker-frontend;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = nativeBuildInputs ++ buildInputs ++ [pkgs.nodejs];
        };
      }
    );
}
