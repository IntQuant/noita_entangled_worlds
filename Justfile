run:
    cd noita-proxy && NP_APPID=480 NP_SKIP_MOD_CHECK=1 cargo run

build:
    cd noita-proxy && cargo build --release

run-rel:
    cd noita-proxy && NP_APPID=480 NP_SKIP_MOD_CHECK=1 cargo run --release

run-rel-n:
    cd noita-proxy && NP_SKIP_MOD_CHECK=1 cargo run --release

run-rel-n-2:
    cd noita-proxy && NP_NOITA_ADDR=127.0.0.1:21252 NP_SKIP_MOD_CHECK=1 cargo run --release

run2:
    cd noita-proxy && NP_APPID=480 NP_SKIP_MOD_CHECK=1 NP_NOITA_ADDR=127.0.0.1:21252 cargo run -- --launch-cmd "wine noita.exe -gamemode 0"

run3:
    cd noita-proxy && NP_APPID=480 NP_SKIP_MOD_CHECK=1 NP_NOITA_ADDR=127.0.0.1:21253 cargo run -- --launch-cmd "wine noita.exe -gamemode 0"

release:
    python prepare_release.py

noita:
    cd /home/quant/.local/share/Steam/steamapps/common/Noita/ && NP_NOITA_ADDR=127.0.0.1:21252 wine noita.exe -gamemode 0

noita1:
    cd /home/quant/.local/share/Steam/steamapps/common/Noita/ && wine noita.exe
    