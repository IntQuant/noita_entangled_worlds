{
  mkShell,
  rust-bin,
  pkgsCross,
}:
let
  # DLLs required for windows.
  dylibs = [
    pkgsCross.mingw32.windows.pthreads
    pkgsCross.mingw32.windows.mcfgthreads
  ];
in
mkShell {
  packages = [
    (rust-bin.selectLatestNightlyWith (
      toolchain:
      toolchain.minimal.override {
        extensions = [
          "rustfmt"
          "rust-analyzer"
          "rust-src"
          "rust-docs"
          "clippy"
        ];
        targets = [ "i686-pc-windows-gnu" ];
      }
    ))
    pkgsCross.mingw32.buildPackages.gcc
  ];
  RUSTFLAGS =
    let
      searchDirs = map (a: "-L ${a}/lib") dylibs;
    in
    [
      "-C"
      "link-arg=-lmcfgthread"
    ]
    ++ searchDirs;
}

