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
            doc =
              let
                eval = lib.evalModules {
                  modules = [
                    {
                      options._module.args = lib.mkOption { internal = true; };
                      config._module.args = { inherit pkgs; };
                      config._module.check = false;
                    }
                    self.nixosModules.default
                  ];
                };
              in
              (pkgs.nixosOptionsDoc { inherit (eval) options; }).optionsCommonMark;
            units =
              let
                eval = lib.nixosSystem {
                  inherit pkgs;
                  modules = [
                    {
                      _module.check = false;
                      services.mdcheck-ng.enable = true;
                      systemd.globalEnvironment = lib.mkForce { };
                      systemd.services.mdcheck-ng.path = lib.mkForce [ ];
                      system.stateVersion = "25.11";
                    }
                    self.nixosModules.default
                  ];
                };
              in
              pkgs.symlinkJoin {
                name = "units";
                paths = [
                  eval.config.systemd.units."mdcheck-ng.service".unit
                  eval.config.systemd.units."mdcheck-ng.timer".unit
                ];
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
