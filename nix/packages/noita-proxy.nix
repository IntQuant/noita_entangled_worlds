{ sourceRoot, lib, runCommandNoCC, rustPlatform, copyDesktopItems
, makeDesktopItem, pkg-config, cmake, patchelf, imagemagick, openssl, libjack2
, alsa-lib, libopus, wayland, libxkbcommon, libGL }:

rustPlatform.buildRustPackage (finalAttrs:
  let
    inherit (finalAttrs) src pname version meta buildInputs steamworksRedist;
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
    nativeBuildInputs =
      [ copyDesktopItems pkg-config cmake patchelf imagemagick ];

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

    # TODO: Research which icon sizes are most important. These are what I found on my system.
    postInstall = ''
      for size in 16 20 22 24 32 48 64 96 128 144 180 192 256 512 1024; do
        icon_dir=$out/share/icons/hicolor/''${size}x''${size}/apps
        mkdir -p $icon_dir
        magick assets/icon.png \
          -strip -filter Point -resize ''${size}x''${size} \
          $icon_dir/noita-proxy.png
      done
    '';

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

    desktopItems = [
      (makeDesktopItem {
        name = "noita-proxy";
        desktopName = "Noita Entangled Worlds";
        comment = meta.description;
        exec = "noita-proxy";
        icon = "noita-proxy";
        categories = [ "Game" "Utility" ];
        keywords = [ "noita" "proxy" "server" "steam" "game" ];
        terminal = false;
        singleMainWindow = true;
      })
    ];

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
