# This function is imported as `rustPackageOverlay` in `nix/overlays/default.nix`.
#
# Supplies a stable `rustPlatform` from `rust-bin` to `callPackage`.
# The `rust-overlay` must have already been composed onto the `pkgs` set.
#
# This prevents `rust-bin` from being an input of the package, which would
# make it less portable.
{ packageName, sourceRoot }:
final: _prev:
let
  rust-stable = final.rust-bin.stable.latest.minimal;
  rustPlatform = final.makeRustPlatform {
    cargo = rust-stable;
    rustc = rust-stable;
  };
in {
  ${packageName} = final.callPackage "${../packages}/${packageName}.nix" {
    inherit sourceRoot rustPlatform;
  };
}
