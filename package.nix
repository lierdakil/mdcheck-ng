{
  lib,
  rustPlatform,
  pkg-config,
}:
let
  manifest = (lib.importTOML ./Cargo.toml).package;
in
rustPlatform.buildRustPackage {
  pname = manifest.name;
  version = manifest.version;
  src = lib.sourceFilesBySuffices (lib.cleanSource ./.) [
    "Cargo.lock"
    "Cargo.toml"
    ".rs"
  ];
  cargoLock.lockFile = ./Cargo.lock;
  nativeBuildInputs = [ pkg-config ];
}
