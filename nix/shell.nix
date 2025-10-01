{ rust-bin, mkShell, noita-proxy }:
mkShell {
  strictDeps = true;

  inputsFrom = [ noita-proxy ];

  packages = [
    # Derivations in `rust-stable` provide the toolchain,
    # must be listed first to take precedence over nightly.
    (rust-bin.stable.latest.minimal.override {
      extensions = [ "rust-src" "rust-docs" "clippy" ];
    })

    # Use rustfmt, and other tools that require nightly features.
    (rust-bin.selectLatestNightlyWith (toolchain:
      toolchain.minimal.override {
        extensions = [ "rustfmt" "rust-analyzer" ];
      }))
  ];

  env = {
    inherit (noita-proxy) OPENSSL_DIR OPENSSL_LIB_DIR OPENSSL_NO_VENDOR;

    RUST_BACKTRACE = 1;
  };
}
