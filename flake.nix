{
  description = "Controller for mdraid check process";

  outputs = { self, nixpkgs, ... }:
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
      packages.${system} = {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;
          src = pkgs.lib.sourceFilesBySuffices (pkgs.lib.cleanSource ./.)
            [ "Cargo.lock" "Cargo.toml" ".rs" ];
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [ pkg-config ];
        };
        doc =
          let eval = pkgs.lib.evalModules {
                modules = [
                  {
                    options._module.args = pkgs.lib.mkOption { internal = true; };
                    config._module.args = { inherit pkgs; };
                    config._module.check = false;
                  }
                  self.nixosModules.default
                ];
              };
          in
            (pkgs.nixosOptionsDoc {
              inherit (eval) options;
            }).optionsCommonMark;
      };
      nixosModules.default = import ./module.nix self;
      checks.${system} = {
        module = pkgs.testers.runNixOSTest (import ./test.nix self);
      };
    };
}
