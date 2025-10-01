{ sourceRoot, lib, runCommandNoCC, rustPlatform, pkg-config, cmake, patchelf
, openssl, libjack2, alsa-lib, libopus, wayland, libxkbcommon, libGL }:

rustPlatform.buildRustPackage (finalAttrs:
  let
    inherit (finalAttrs) src pname version buildInputs steamworksRedist;
    manifest = lib.importTOML "${src}/noita-proxy/Cargo.toml";
  in {
    pname = "noita-entangled-worlds-proxy";
    inherit (manifest.package) version;

    # The real root of the entire project.
    # This is the only place `sourceRoot` is used.
    src = sourceRoot;
    # The root of this particular binary crate to build.
    sourceRoot = "source/noita-proxy";

    cargoLock.lockFile = "${src}/noita-proxy/Cargo.lock";

    strictDeps = true;
    nativeBuildInputs = [ pkg-config cmake patchelf ];

    # TODO: Add dependencies for X11 desktop environments.
    buildInputs = [
      openssl
      libjack2
      alsa-lib
      libopus
      wayland
      libxkbcommon
      libGL
      steamworksRedist
    ];

    env = {
      OPENSSL_DIR = "${lib.getDev openssl}";
      OPENSSL_LIB_DIR = "${lib.getLib openssl}/lib";
      OPENSSL_NO_VENDOR = 1;
    };

    checkFlags = [
      # Disable networked tests
      "--skip bookkeeping::releases::test::release_assets"
    ];

    postFixup = ''
      patchelf $out/bin/noita-proxy \
        --set-rpath ${lib.makeLibraryPath buildInputs}
    '';

    # This attribute is defined here instead of a `let` block, because in this position,
    # it can be overridden with `overrideAttrs`, and shares a `src` with the top-level.
    steamworksRedist =
      runCommandNoCC "${pname}-steamworks-redist" { inherit src; } ''
        install -Dm555 $src/redist/libsteam_api.so -t $out/lib
      '';

    meta = {
      description = "Noita Entangled Worlds proxy application.";
      homepage = "https://github.com/IntQuant/noita_entangled_worlds";
      changelog =
        "https://github.com/IntQuant/noita_entangled_worlds/releases/tag/v${version}";
      license = with lib.licenses; [ mit asl20 ];
      platforms = [ "x86_64-linux" ];
      maintainers = with lib.maintainers; [ spikespaz ];
      mainProgram = "noita-proxy";
    };
  })
