self:
{pkgs, lib, config, ...}:
let inherit (lib) types;
    conf = config.services.mdcheck-ng;
in {
  options.services.mdcheck-ng = rec {
    enable = lib.mkEnableOption "Enable mdcheck-ng service";
    runSchedule = lib.mkOption {
      type = types.str;
      description = "When to run the service. Must fall within start and continue. Systemd OnCalendar format.";
      default = "daily";
      example = "Sun *-*-* 01:00";
    };
    logLevel = lib.mkOption {
      type = types.enum [ "trace" "debug" "info" "warn" "error" ];
      description = "Log level";
      default = "error";
      example = "info";
    };
    maxRunDuration = lib.mkOption {
      type = types.nullOr types.str;
      description =
        "Maximum duration for a single run. Can be used to limit scrub time,
        instead of specifying ranges in `start` and `continue`. Accepts
        `humantime` strings, see <https://docs.rs/humantime/latest/humantime/fn.parse_duration.html>
        for exact syntax.";
      default = null;
      example = "6h 30m";
    };
    global = lib.mkOption {
      description = "Global options";
      default = {};
      type = types.submodule {
        options = {
          start = lib.mkOption {
            type = types.nullOr types.str;
            description = "Cron string to define when a scrub can start, in the croner format.
              See <https://docs.rs/croner/latest/croner/#pattern> for exact syntax, but note that
              seconds are NOT optional.";
            default = null;
            example = "* * 1-15 * * Sun#1";
          };
          continue = lib.mkOption {
            type = types.nullOr types.str;
            description = "Cron string to define when a scrub can continue, in the same format as `start`.";
            default = null;
            example = "* * 1-15 * * Sun";
          };
          ionice = lib.mkOption {
            type = types.nullOr types.str;
            description =
              "ionice CLI arguments specifying ionice class and level for the scrub process";
            default = null;
            example = "-c2 -n7";
          };
          nice = lib.mkOption {
            type = types.nullOr types.ints.s8;
            description = "Nice level for the scrub process";
            default = null;
            example = 15;
          };
        };
      };
    };
    devices = lib.mkOption {
      type = types.attrsOf global.type;
      description = "Per-device overrides";
      default = {};
      example = {
        md127.start = "* * 1-15 * * Sat";
      };
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
