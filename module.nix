self:
{pkgs, lib, config, ...}:
let inherit (lib) types;
    conf = config.services.mdcheck-ng;
in {
  options.services.mdcheck-ng = rec {
    enable = lib.mkEnableOption "Enable mdcheck-ng service";
    runSchedule = lib.mkOption {
      type = types.str;
      description = "When to run the service. Must fall within start and continue";
      default = "daily";
    };
    logLevel = lib.mkOption {
      type = types.enum [ "trace" "debug" "info" "warn" "error" ];
      description = "Log level";
      default = "error";
    };
    maxRunDuration = lib.mkOption {
      type = types.nullOr types.str;
      description = "Maximum run duration";
      default = null;
    };
    global = lib.mkOption {
      description = "Global options";
      default = {};
      type = types.submodule {
        options = {
          start = lib.mkOption {
            type = types.nullOr types.str;
            description = "Cron string to define when a scrub can start";
            default = null;
          };
          continue = lib.mkOption {
            type = types.nullOr types.str;
            description = "Cron string to define when a scrub can continue";
            default = null;
          };
          ionice = lib.mkOption {
            type = types.nullOr types.str;
            description = "ionice CLI arguments specifying ionice class and level";
            default = null;
          };
          nice = lib.mkOption {
            type = types.nullOr types.ints.s8;
            description = "ionice CLI arguments specifying ionice class and level";
            default = null;
          };
        };
      };
    };
    devices = lib.mkOption {
      type = types.attrsOf global.type;
      description = "Per-device overrides";
      default = {};
    };
  };
  config = lib.mkIf conf.enable {
    systemd.timers.mdcheck-ng = {
      wantedBy = [ "timers.target" ];
      timerConfig.OnCalendar = conf.runSchedule;
    };
    systemd.services.mdcheck-ng = {
      path = [ pkgs.util-linux ];
      serviceConfig = {
        ExecStart =
          let filterNull = lib.attrsets.filterAttrs (_: v: v != null);
              config_toml = (pkgs.formats.toml {}).generate "mdcheck-ng.toml"
                (filterNull ({ max_run_duration = conf.maxRunDuration; } // conf.global) //
                  lib.attrsets.mapAttrs (_: filterNull) conf.devices);
          in "${self.packages.${pkgs.system}.default}/bin/mdcheck-ng ${config_toml}";
        WorkingDirectory = "/var/lib/mdcheck-ng";
        Type = "oneshot";
        User = "root";
      };
      environment.RUST_LOG = conf.logLevel;
    };
    systemd.tmpfiles.rules = [
      "d /var/lib/mdcheck-ng 0755 root root -"
    ];
  };
}
