{
  description = "Controller for mdraid check process";

  outputs = { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
    in
    {
      devShells.${system}.default = with pkgs; mkShell {
        buildInputs = [
          rustc
          cargo
          clippy
          rust-analyzer
          rustfmt
        ];
        # Environment variables
        RUST_SRC_PATH = rustPlatform.rustLibSrc;
      };
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = manifest.name;
        version = manifest.version;
        src = pkgs.lib.cleanSource ./.;
        cargoLock.lockFile = ./Cargo.lock;
        nativeBuildInputs = with pkgs; [ pkg-config ];
      };
    };
}
