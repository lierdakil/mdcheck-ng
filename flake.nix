{
  description = "Controller for mdraid check process";

  outputs = { self, nixpkgs, ... }:
    let systems = ["x86_64-linux" "aarch64-linux"];
        base = { nixosModules.default = import ./module.nix self; };
        per-system = system:
          let pkgs = nixpkgs.legacyPackages.${system};
          in builtins.mapAttrs (_: v: { ${system} = v; }) {
            devShells.default = with pkgs; mkShell {
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
            packages = let manifest = (pkgs.lib.importTOML ./Cargo.toml).package; in {
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
                in (pkgs.nixosOptionsDoc { inherit (eval) options;}).optionsCommonMark;
            };
            checks = {
              module = pkgs.testers.runNixOSTest (import ./test.nix self);
            };
          };
          inherit (nixpkgs.lib) recursiveUpdate map foldr;
    in foldr recursiveUpdate base (map per-system systems);
}
