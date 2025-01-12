# mdcheck-ng

This project is born from mild frustration with [mdcheck] and [checkarray]
scripts, and the fact that neither quite works on NixOS out of the box (well,
rather, the latter is not available, and the former is shipped by default, but
extremely broken, and not very easy to fix).

[mdcheck]: https://github.com/md-raid-utilities/mdadm/blob/main/misc/mdcheck
[checkarray]: https://salsa.debian.org/debian/mdadm/-/blob/debian/master/debian/checkarray

Basically, this takes the same basic approach as mdcheck, steals a few useful
titbits, namely ionice and renice, from checkarray, and packages this all as a
Rust flake.

## Why would I want this?

If you're running mdraid arrays on Linux and they're not periodically scrubbed,
you should be worried. You'll only find out bitrot happened when you try to read
the data that's not there anymore. Pray that it's still recoverable by that
point.

So, that establishes that we want periodic scrubbing. What are our options? A
few:

- Run `echo check > /sys/block/md*/md/sync_action` in a cron job. This is
  simple, but not particularly robust and potentially dangerous. For one, for
  large HDD arrays, this will take a long time, all the while your system will
  be under heavy I/O load -- so iowait times will skyrocket. Not really ideal.
  This should not cancel resync if one's happening when the cron job runs,
  `resync` should take priority over `check`, but I can't guarantee it'll
  actually work that way every time.

- The aforementioned `checkarray` in a cron job. This script will tune the check
  process to reduce the overall system impact. You still can observe very high
  iowait, especially if your machine experiences mildly heavy load during check
  (e.g. because some search spider decided to crawl your website). Not available
  in NixOS, but one could certainly hack it in if they were so inclined.

- `mdcheck` script with its associated systemd timers and services. This will
  pause the check after a configurable delay and restart it according to a
  systemd timer schedule. The problem is, it makes a lot of assumptions and ends
  up being rather brittle. This is shipped with `mdadm`, and hence available on
  NixOS... kind of. The script itself isn't installed, only its units, and the
  units expect the script to be in `/usr/share`. Also the script itself
  starts/stops timers which doesn't quite work as expected on NixOS. Also, the
  script is not particularly robust Bash, shellcheck complains quite a bit.

Overall, my recommendation would be to go with `checkarray` if it's available, and all your arrays are small enough (or fast enough) to check in a few hours.

If you're on NixOS, however, only the first option is really available without
extra steps (and those steps end up being quite complicated). Hence, this
project.

The principle aims of this project are thus:

- Run periodic mdraid scrubs.
- Adjust io-/nice level of the scrub process like `checkarray`.
- Allow for pause-and-resume like `mdcheck`.
- Avoid complicated and brittle systemd/cron wrangling.
- Be as robust as practically feasible.

## Could this just have been a shell script?

Yes. Shell is easier to mess up though, requires more external dependencies, and
has less options for nice things like TOML configs. I see no strong reason to
prefer shell, all things considered.

## Basic setup

This is still early days, so I'll just provide a very barebones setup
instructions here. We'll see about improving these once I run this for a while
and decide it's worth the trouble.

The basic principle is this: set up the binary to run whenever you might
potentially want to start/continue scrubbing. Could be via cron or via systemd
timers. I'll go with the latter option.

Thus, the service:

```ini
# /etc/systemd/system/mdcheck-ng.service
[Unit]

[Service]
Type=oneshot
User=root
WorkingDirectory=/var/lib/mdcheck-ng
ExecStart=/path/to/mdcheck-ng /path/to/mdcheck-ng.toml
```

The timer:

```ini
# /etc/systemd/system/mdcheck-ng.timer
[Unit]

[Timer]
OnCalendar=daily

[Install]
WantedBy=timers.target
```

The example units are available in the repo in `./systemd/`.

This will run the service every day at 1 AM. Whether any scrubbing actually
takes place or not is decided by the config. If no scrubbing is supposed to take
place, the service will exit immediately, so _make sure_ it's started during `start` and/or `continue` intervals defined in the config.

Beware, however, that the service will happily restart a scrub it may have just
finished. So avoid starting it more than once within the same activity period
defined in `start`. This is a rather exotic edge case, mind.

The flake provides a NixOS module that does all this, so you can do something like this:

```nix
{ config, lib, ... }:
{
  # NB: this assumes you pass the flake through to the module somehow, there
  # are many options, the easiest is via specialArgs. You could also just load
  # the module directly while setting up `nixosConfigurations` instead.
  imports = [ mdcheck-ng-flake.nixosModules.default ];
  config = lib.mkIf config.boot.swraid.enable {
    services.mdcheck-ng = {
      enable = true;
      runSchedule = "01:00";
      logLevel = "info";
      global = {
        start = "Sun#1";
        continue = "Sun";
        ionice = "-c3";
        nice = 15;
        max_run_duration = "6h";
      };
    };
    # optionally disable the default borkerd timers; they don't work anyway, so
    # might as well.
    systemd.timers = {
      mdcheck_start.enable = false;
      mdcheck_continue.enable = false;
    };
  };
}
```

Please see [options.md](./options.md) for NixOS module options reference.

## Config

Config is in TOML format. The fields are:

- `start`: Crontab string defining when a new scrub will be started. Any fields
  not specified are assumed to be `*`. If `start` is unspecified entirely, no
  scrubs are ever started. For example, `"Sun#1"` will start a scrub on the
  first Sunday of the month. This uses croner to parse crontab specs, see [the
  docs](https://docs.rs/croner/latest/croner/#pattern) for more information on
  the exact syntax.
- `continue`: Same as `start`, but for continuing checks. This should generally
  overlap with `start`, but include other time intervals. If unspecified,
  defaults to `start`. For example, `"Sun"` will resume checks every Sunday.
- `max_run_duration`: Maximum duration for a single run, in [humantime format].
  Used to limit scrub time per run.
- `ionice`: part of `ionice` command line to set ionice level on the check
  process. Does nothing if unspecified. For example, "-c3" will set to `idle`
  ionice level.
- `nice`: what nice level to set the check process to. Does nothing if
  unspecified. For example `15` will set nice level to `15`. Nice level can be
  negative (but generally you don't want that).

[humantime format]: https://docs.rs/humantime/latest/humantime/fn.parse_duration.html

One can specify any of these per md device. For example:

```toml
start = "Sun#1"
continue = "Sun"
ionice = "-c3"
nice = 15

[md127]
start = "Sat#1"
continue = "Sat"
ionice = "-c2 -n7"
max_run_duration = "6h"
```

will run checks on `md127` on Saturdays instead of Sundays, and with slightly
higher ionice priority (literally one step above `idle`).

Any fields unspecified in per-device overloads are taken from the root config
instead.
