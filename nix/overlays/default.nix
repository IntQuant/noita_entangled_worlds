{ self, lib, rust-overlay }:
let rustPackageOverlay = import ./rust-package.nix;
in {
  default = lib.composeManyExtensions [
    # This is to ensure that other overlays and invocations of `callPackage`
    # receive `rust-bin`, but without hard-coding a specific derivation.
    # This can be overridden by consumers.
    self.overlays.rust-overlay

    # Packages provided by this flake:
    self.overlays.noita-proxy
  ];

  # This flake exposes `overlays.rust-overlay` which is automatically applied
  # by `overlays.default`. This overlay is intended to provide `rust-bin` for
  # the package overlay, in the event it is not already present.
  rust-overlay = final: prev:
    if prev ? rust-bin then { } else rust-overlay.overlays.default final prev;

  # The overlay definition uses `rust-bin` to construct a `rustPlatform`,
  # and `rust-bin` is not provided by this particular overlay.
  # Prefer using `overlays.default`, or composing with `rust-overlay` manually.
  noita-proxy = rustPackageOverlay {
    packageName = "noita-proxy";
    sourceRoot = self;
  };
}
