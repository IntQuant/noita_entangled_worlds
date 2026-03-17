{
  rust-bin,
  mkShell,
  noita-proxy,
  pkgs,
  pkgsCross,
  lib,
}:
let
  # DLLs required for windows.
  dylibs = [
    pkgsCross.mingwW64.windows.pthreads
    pkgsCross.mingwW64.libopus
  ];
  buildInputs = [
    noita-proxy.steamworksRedist
  ];
in
mkShell {
  packages = [
    (rust-bin.stable.latest.minimal.override {
      extensions = [
        "rust-src"
        "rust-docs"
      ];
      targets = [ "x86_64-pc-windows-gnu" ];
    })
    pkgs.pkgsCross.mingwW64.buildPackages.gcc
  ];

  LD_LIBRARY_PATH = lib.makeLibraryPath (dylibs ++ buildInputs);
  RUSTFLAGS = map (a: "-L ${a}/lib") dylibs;
}
