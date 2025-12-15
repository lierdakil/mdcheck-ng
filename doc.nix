{
  lib,
  nixosOptionsDoc,
  pkgs,
  module,
}:
let
  eval = lib.evalModules {
    modules = [
      {
        options._module.args = lib.mkOption { internal = true; };
        config._module.args = { inherit pkgs; };
        config._module.check = false;
      }
      module
    ];
  };
in
(nixosOptionsDoc { inherit (eval) options; }).optionsCommonMark
