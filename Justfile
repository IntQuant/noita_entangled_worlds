run:
    cd noita-proxy && NP_APPID=480 NP_SKIP_MOD_CHECK=1 cargo run

run-rel:
    cd noita-proxy && NP_APPID=480 NP_SKIP_MOD_CHECK=1 cargo run --release

run-rel-n:
    cd noita-proxy && NP_SKIP_MOD_CHECK=1 cargo run --release

run2:
    cd noita-proxy && NP_APPID=480 NP_SKIP_MOD_CHECK=1 NP_NOITA_ADDR=127.0.0.1:21252 cargo run

release:
    python prepare_release.py
