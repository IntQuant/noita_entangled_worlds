extract_steam_redist:
    python scripts/extract_steam_redist.py

add_dylib_debug: extract_steam_redist
    mkdir noita-proxy/target/debug/ -p
    cp redist/libsteam_api.so noita-proxy/target/debug/

add_dylib_release: extract_steam_redist
    mkdir noita-proxy/target/release/ -p
    cp redist/libsteam_api.so noita-proxy/target/release/

build:
    cd noita-proxy && cargo build
    cd noita-proxy && cargo build --release

build_noita_api_example:
    cd noita_api_example && cargo build --release --target=i686-pc-windows-gnu
    cp noita_api_example/target/i686-pc-windows-gnu/release/noita_api_example.dll noita_api_example/material_converter/material_converter.dll

## ewext stuff
build_luajit:
    mkdir target/ -p
    cd target && git clone https://luajit.org/git/luajit.git || true
    cd target/luajit && git checkout v2.0.4 && make HOST_CC="gcc -m32" CROSS=i686-w64-mingw32- TARGET_SYS=Windows
    cp target/luajit/src/
    bindgen ../target/luajit/src/lua.h -o src/lua_bindings.rs --dynamic-loading Lua51 --no-layout-tests

# `rustup target add i686-pc-windows-gnu` first
build_ext:
    cd ewext && cargo build --release --target=i686-pc-windows-gnu
    cp ewext/target/i686-pc-windows-gnu/release/ewext.dll quant.ew/ewext.dll

build_ext_debug:
    cd ewext && cargo build --target=i686-pc-windows-gnu
    cp ewext/target/i686-pc-windows-gnu/debug/ewext.dll quant.ew/ewext.dll

##
build_blob:
    cd blob_guy && cargo build --release --target=i686-pc-windows-gnu
    cp blob_guy/target/i686-pc-windows-gnu/release/blob_guy.dll blob_guy/blob_guy/blob_guy.dll
build_blob_debug:
    cd blob_guy && cargo build --target=i686-pc-windows-gnu
    cp blob_guy/target/i686-pc-windows-gnu/debug/blob_guy.dll blob_guy/blob_guy/blob_guy.dll

run-rel: add_dylib_release build_ext
    cd noita-proxy && NP_SKIP_MOD_CHECK=1 cargo run --release

flamegraph: add_dylib_debug
    cd noita-proxy && NP_APPID=480 NP_SKIP_MOD_CHECK=1 cargo flamegraph

run: add_dylib_debug build_ext
    cd noita-proxy && NP_SKIP_MOD_CHECK=1 cargo run

run-w-gdb: add_dylib_debug build_ext
    cd noita-proxy && NP_SKIP_MOD_CHECK=1 cargo run -- --run-noita-with-gdb --launch-cmd "wine noita.exe"

run2: add_dylib_debug build_ext
    cd noita-proxy && NP_SKIP_MOD_CHECK=1 cargo run -- --launch-cmd "wine noita.exe -gamemode 0"

run2-alt: add_dylib_debug build_ext
    cd noita-proxy && NP_SKIP_MOD_CHECK=1 cargo run -- --launch-cmd "strace wine noita.exe -gamemode 0"

run3: add_dylib_debug build_ext
    cd noita-proxy && NP_SKIP_MOD_CHECK=1 NP_NOITA_ADDR=127.0.0.1:21253 cargo run -- --launch-cmd "strace wine noita.exe -gamemode 0"

release_old: build_ext
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
