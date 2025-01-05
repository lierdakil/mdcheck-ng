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

No nixos configuration in the flake at the time of writing, because I was being
lazy. PRs welcome.

## Config

Config is in TOML format. The fields are:

- `start`: Crontab string defining when a new check will be started, and how
  long it will run. The crontab spec MUST include seconds (likely as `*`). If
  unspecified, no checks are ever started. For example, `"* * 1-6 * * Sun#1"`
  will start check on the first Sunday of the month, at 1 AM, and will run until
  6:59:59 AM. Using croner to parse crontab specs, see [the
  docs](https://docs.rs/croner/latest/croner/#pattern) for more information on
  the allowed syntax.
- `continue`: Same as `start`, but for continuing checks. This should generally
  overlap with `start`, but include other time intervals. If unspecified,
  defaults to `start`. For example, `"* * 1-6 * * Sun"` will resume checks every
  Sunday, and will pause them again after 6:59:59 AM.
- `ionice`: part of `ionice` command line to set ionice level on the check
  process. Does nothing if unspecified. For example, "-c3" will set to `idle`
  ionice level.
- `nice`: what nice level to set the check process to. Does nothing if
  unspecified. For example `15` will set nice level to `15`. Nice level can be
  negative (but generally you don't want that).

One can specify any of these per md device. For example:

```toml
start = "* * 1-6 * * Sun#1"
continue = "* * 1-6 * * Sun"
ionice = "-c3"
nice = 15

[md127]
start = "* * 1-6 * * Sat#1"
continue = "* * 1-6 * * Sat"
ionice = "-c2 -n7"
```

will run checks on `md127` on Saturdays instead of Sundays, and with slightly
higher ionice priority (literally one step above `idle`).

Any fields unspecified in per-device overloads are taken from the root config
instead.
