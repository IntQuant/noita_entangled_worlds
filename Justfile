extract_steam_redist:
    python scripts/extract_steam_redist.py

add_dylib_debug: extract_steam_redist
    mkdir noita-proxy/target/debug/ -p
    cp target/tmp/libsteam_api.so noita-proxy/target/debug/

add_dylib_release: extract_steam_redist
    mkdir noita-proxy/target/release/ -p
    cp target/tmp/libsteam_api.so noita-proxy/target/release/

build:
    cd noita-proxy && cargo build
    cd noita-proxy && cargo build --release

run-rel: add_dylib_release
    cd noita-proxy && NP_APPID=480 NP_SKIP_MOD_CHECK=1 cargo run --release

run: add_dylib_debug
    cd noita-proxy && NP_APPID=480 NP_SKIP_MOD_CHECK=1 cargo run

run2: add_dylib_debug
    cd noita-proxy && NP_APPID=480 NP_SKIP_MOD_CHECK=1 NP_NOITA_ADDR=127.0.0.1:21252 cargo run -- --launch-cmd "wine noita.exe -gamemode 0"

run3: add_dylib_debug
    cd noita-proxy && NP_APPID=480 NP_SKIP_MOD_CHECK=1 NP_NOITA_ADDR=127.0.0.1:21253 cargo run -- --launch-cmd "wine noita.exe -gamemode 0"

release: build add_dylib_release
    python scripts/prepare_release.py
