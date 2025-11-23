set shell := ["pwsh.exe", "-c"]
export CMAKE_POLICY_VERSION_MINIMUM := "3.5"

extract_steam_redist:
    python scripts/extract_steam_redist.py

add_dylib_debug: extract_steam_redist
    mkdir -Force "noita-proxy/target/debug/"
    cp "redist/libsteam_api.so" "noita-proxy/target/debug/"

add_dylib_release: extract_steam_redist
    mkdir noita-proxy/target/release/ > nul
    cp redist/libsteam_api.so noita-proxy/target/release/

build:
    cd noita-proxy && cargo build
    cd noita-proxy && cargo build --release

build_noita_api_example:
    cd noita_api_example && cargo build --release --target=i686-pc-windows-gnu
    cp noita_api_example/target/i686-pc-windows-gnu/release/noita_api_example.dll noita_api_example/material_converter/material_converter.dll

# # ewext stuff
build_luajit:
    mkdir -Force "target/"
    cd target && git clone https://luajit.org/git/luajit.git || $true
    cd target/luajit && git checkout v2.0.4 && make HOST_CC="gcc -m32" CROSS=i686-w64-mingw32- TARGET_SYS=Windows
    cp target/luajit/src/ -erroraction 'silentlycontinue' || $true
    bindgen target/luajit/src/lua.h -o src/lua_bindings.rs --dynamic-loading Lua51 --no-layout-tests

# `rustup target add i686-pc-windows-gnu` first
build_ext:
    cd ewext && cargo build --release --target=i686-pc-windows-gnu
    cp ewext/target/i686-pc-windows-gnu/release/ewext.dll quant.ew/ewext.dll

build_ext_debug:
    cd ewext && cargo build --target=i686-pc-windows-gnu
    cp ewext/target/i686-pc-windows-gnu/debug/ewext.dll quant.ew/ewext.dll

# # mod movin
# # $env:RM_KEEP_MOD="1"; for 1 run without rm mod
move_mod:
    $env:RM_KEEP_MOD ? $true : (rm -Recurse -Force -Path "E:/SteamLibrary/steamapps/common/Noita/mods/quant.ew" || $true)
    $env:RM_KEEP_MOD ? $true : (cp "quant.ew" "E:/SteamLibrary/steamapps/common/Noita/mods/" -Recurse -Force || $true)

# #run commands
build_blob:
    cd blob_guy && cargo build --release --target=i686-pc-windows-gnu
    cp blob_guy/target/i686-pc-windows-gnu/release/blob_guy.dll blob_guy/blob_guy/blob_guy.dll

build_blob_debug:
    cd blob_guy && cargo build --target=i686-pc-windows-gnu
    cp blob_guy/target/i686-pc-windows-gnu/debug/blob_guy.dll blob_guy/blob_guy/blob_guy.dll

run-rel $NP_SKIP_MOD_CHECK="1": add_dylib_release move_mod
    cd noita-proxy && cargo run --release

flamegraph: add_dylib_debug
    cd noita-proxy && cargo flamegraph

run $NP_SKIP_MOD_CHECK="1": add_dylib_debug build_ext move_mod
    cd noita-proxy && cargo run

run-w-gdb $NP_SKIP_MOD_CHECK="1": add_dylib_debug build_ext
    cd noita-proxy && cargo run -- --run-noita-with-gdb


run2 PARAMS="" $NP_SKIP_MOD_CHECK="1": add_dylib_debug build_ext move_mod
    cd noita-proxy && cargo run -- --launch-cmd "E:/SteamLibrary/steamapps/common/Noita/noita_dev.exe -gamemode 0" {{PARAMS}}

run2-alt $NP_SKIP_MOD_CHECK="1": add_dylib_debug build_ext move_mod
    cd noita-proxy && cargo run -- --launch-cmd "'E:\Archivos Programas\x64dbg\release\x32' E:/SteamLibrary/steamapps/common/Noita/noita_dev.exe -gamemode 0"

run3 $NP_SKIP_MOD_CHECK="1": add_dylib_debug build_ext move_mod
    cd noita-proxy && NP_NOITA_ADDR=127.0.0.1:21253 cargo run -- --launch-cmd "'E:\Archivos Programas\x64dbg\release\x32' E:/SteamLibrary/steamapps/common/Noita/noita_dev.exe -gamemode 0"

release_old: build_ext move_mod
    python scripts/prepare_release.py

release:
    cd noita-proxy && cargo check
    cd ewext && cargo check
    python scripts/check_pre_ci.py

clean:
    cd noita-proxy && cargo clean
    cd ewext && cargo clean

make_release_assets:
    python scripts/make_release_assets.py
