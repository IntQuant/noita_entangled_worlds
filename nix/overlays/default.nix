{ self, lib, rust-overlay }: {
  default = lib.composeManyExtensions [
    # This is to ensure that other overlays and invocations of `callPackage`
    # receive `rust-bin`, but without hard-coding a specific derivation.
    # This can be overridden by consumers.
    self.overlays.rust-overlay
  ];

  # This flake exposes `overlays.rust-overlay` which is automatically applied
  # by `overlays.default`. This overlay is intended to provide `rust-bin` for
  # the package overlay, in the event it is not already present.
  rust-overlay = final: prev:
    if prev ? rust-bin then { } else rust-overlay.overlays.default final prev;
}
