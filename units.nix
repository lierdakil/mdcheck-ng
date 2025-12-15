{
  lib,
  pkgs,
  symlinkJoin,
  module,
  nixosSystem,
}:
let
  eval = nixosSystem {
    inherit pkgs;
    modules = [
      {
        _module.check = false;
        services.mdcheck-ng.enable = true;
        systemd.globalEnvironment = lib.mkForce { };
        systemd.services.mdcheck-ng.path = lib.mkForce [ ];
        system.stateVersion = "25.11";
      }
      module
    ];
  };
in
symlinkJoin {
  name = "units";
  paths = [
    eval.config.systemd.units."mdcheck-ng.service".unit
    eval.config.systemd.units."mdcheck-ng.timer".unit
  ];
}
