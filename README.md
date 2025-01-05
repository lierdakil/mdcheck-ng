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
