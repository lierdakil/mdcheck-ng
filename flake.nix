{
  description = "Controller for mdraid check process";

  outputs =
    { self, nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      inherit (nixpkgs) lib;
      per-system =
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ self.overlays.default ];
          };
        in
        {
          devShells.default =
            with pkgs;
            mkShell {
              inputsFrom = [ pkgs.mdcheck-ng ];
              buildInputs = [
                clippy
                rust-analyzer
                rustfmt
              ];
              # Environment variables
              RUST_SRC_PATH = rustPlatform.rustLibSrc;
            };
          legacyPackages.pkgsStatic.mdcheck-ng = pkgs.pkgsStatic.mdcheck-ng;
          legacyPackages.pkgsCross = lib.mapAttrs (_: val: {
            inherit (val) mdcheck-ng;
            pkgsStatic = {
              # a horrible hack to avoid the whole pkgsCross.*.pkgsStatic mess
              mdcheck-ng = val.mdcheck-ng.overrideAttrs (_: {
                env.RUSTFLAGS = "-C target-feature=+crt-static";
              });
            };
          }) pkgs.pkgsCross;
          packages = rec {
            default = mdcheck-ng;
            mdcheck-ng = pkgs.mdcheck-ng;
            doc = pkgs.callPackage ./doc.nix { module = self.nixosModules.default; };
            units = pkgs.callPackage ./units.nix {
              module = self.nixosModules.default;
              nixosSystem = lib.nixosSystem;
            };
          };
          checks = {
            module = pkgs.testers.runNixOSTest (import ./test.nix self);
          };
        };
    in
    lib.foldr lib.recursiveUpdate { } (
      lib.map (sys: lib.mapAttrs (_: v: { ${sys} = v; }) (per-system sys)) systems
    )
    // {
      nixosModules.default = import ./module.nix self;
      overlays.default = final: prev: { mdcheck-ng = final.callPackage ./package.nix { }; };
    };
}
