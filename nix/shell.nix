{
  rust-bin,
  mkShell,
  noita_proxy,
  pkgs,
  lib,
}:
let
  buildInputs = with pkgs; [
    openssl
    libjack2
    alsa-lib
    libopus
    wayland
    libxkbcommon
    libGL
    noita_proxy.steamworksRedist
  ];
  DearImGui = pkgs.fetchzip {
    url = "https://github.com/dextercd/Noita-Dear-ImGui/releases/download/release-1.26.0/NoitaDearImGui-1.26.0.zip";
    hash = "sha256-StvMK9udG+gbf1EOUO9O2nOORtxYEls3bh+qSdh4Qrs=";
    stripRoot = false;
  };
  ComponentExplorer = pkgs.fetchzip {
    url = "https://github.com/dextercd/Noita-Component-Explorer/releases/download/release-1.60.5/ComponentExplorer-1.60.5.zip";
    hash = "sha256-KooMsP588WPjruBcgf0tnbOMckFQFtuai/aWUt4nDDk=";
  };
  MiniDump = pkgs.fetchzip {
    url = "https://github.com/dextercd/Noita-Minidump/releases/download/release-1.2.1/NoitaMinidump-1.2.1.zip";
    hash = "sha256-MIqiHdND+d3lPI66SH3I1DcZeIP/sU37fu/MjNlgEu4=";
  };
in
mkShell {
  strictDeps = true;

  inputsFrom = [ noita_proxy ];

  packages = [
    # Derivations in `rust-stable` provide the toolchain,
    # must be listed first to take precedence over nightly.
    (rust-bin.stable.latest.minimal.override {
      extensions = [
        "rust-src"
        "rust-docs"
        "clippy"
      ];
    })

    # Use rustfmt, and other tools that require nightly features.
    (rust-bin.selectLatestNightlyWith (
      toolchain:
      toolchain.minimal.override {
        extensions = [
          "rustfmt"
          "rust-analyzer"
        ];
      }
    ))

    pkgs.lua-language-server
  ];

  NOITA_MOD_COMPONENTEXPLORER = "${ComponentExplorer}";
  NOITA_MOD_NOITADEARIMGUI = "${DearImGui}/NoitaDearImGui";
  NOITA_MOD_MINIDUMP = "${MiniDump}";

  env = {
    inherit (noita_proxy) OPENSSL_DIR OPENSSL_LIB_DIR OPENSSL_NO_VENDOR;

    RUST_BACKTRACE = 1;
    LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
  };
}
