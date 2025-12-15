self:
{
  pkgs,
  lib,
  config,
  ...
}:
let
  inherit (lib) types;
  conf = config.services.mdcheck-ng;
in
{
  options.services.mdcheck-ng = rec {
    enable = lib.mkEnableOption "mdcheck-ng service";
    runSchedule = lib.mkOption {
      type = types.str;
      description = "When to run the service. Must fall within start and continue. Systemd OnCalendar format.";
      default = "daily";
      example = "Sun *-*-* 01:00";
    };
    logLevel = lib.mkOption {
      type = types.enum [
        "trace"
        "debug"
        "info"
        "warn"
        "error"
      ];
      description = "Log level";
      default = "error";
      example = "info";
    };
    global = lib.mkOption {
      description = "Global options";
      default = { };
      type = types.submodule {
        options = {
          start = lib.mkOption {
            type = types.nullOr types.str;
            description = "Cron string to define when a scrub can start, in the croner format.
              See <https://docs.rs/croner/latest/croner/#pattern> for exact syntax. Any fields not
              specified will be assumed to be `*`, so you could specify just `Sun#1` to run on the
              first Sunday of the month.";
            default = null;
            example = "Sun#1";
          };
          continue = lib.mkOption {
            type = types.nullOr types.str;
            description = "Cron string to define when a scrub can continue, in the same format as `start`.";
            default = null;
            example = "Sun";
          };
          ionice = lib.mkOption {
            type =
              let
                idle = types.enum [ "idle" ];
                besteffort = types.submodule {
                  options = {
                    best_effort = lib.mkOption {
                      type = types.ints.between 0 7;
                    };
                  };
                };
                realtime = types.submodule {
                  options = {
                    realtime = lib.mkOption {
                      type = types.ints.between 0 7;
                    };
                  };
                };
              in
              types.nullOr (
                types.oneOf [
                  idle
                  besteffort
                  realtime
                ]
              );
            description = ''Either `"idle"`, or `{ best_effort = lvl; }`, or `{ realtime = lvl; }`, where lvl is between 0 and 7'';
            default = null;
            example = "-c2 -n7";
          };
          nice = lib.mkOption {
            type = types.nullOr (types.ints.between (-20) 19);
            description = "Nice level for the scrub process";
            default = null;
            example = 15;
          };
          max_run_duration = lib.mkOption {
            type = types.nullOr types.str;
            description = "Maximum duration for a single run. Used to limit scrub time.
              Unspecified means unlimited. Accepts `humantime` strings, see
              <https://docs.rs/humantime/latest/humantime/fn.parse_duration.html>
              for the exact syntax.";
            default = null;
            example = "6h 30m";
          };
        };
      };
    };
    devices = lib.mkOption {
      type = types.attrsOf global.type;
      description = "Per-device overrides";
      default = { };
      example = {
        md127 = {
          start = "Sat#1";
          continue = "Sat";
          max_run_duration = "6h";
        };
      };
    };
  };
  config = lib.mkIf conf.enable {
    systemd.timers.mdcheck-ng = {
      description = "Periodically runs mdcheck-ng";
      wantedBy = [ "timers.target" ];
      timerConfig.OnCalendar = conf.runSchedule;
    };
    systemd.services.mdcheck-ng = {
      description = "Controller for mdraid check process";
      restartIfChanged = false;
      serviceConfig = {
        ExecStart =
          let
            filterNull = lib.attrsets.filterAttrs (_: v: v != null);
            config_toml = (pkgs.formats.toml { }).generate "mdcheck-ng.toml" (
              filterNull conf.global // lib.attrsets.mapAttrs (_: filterNull) conf.devices
            );
          in
          "${self.packages.${pkgs.stdenv.hostPlatform.system}.default}/bin/mdcheck-ng ${config_toml}";
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
