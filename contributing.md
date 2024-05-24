# Building Noita Proxy

Rust toolchain can be acquired from https://rustup.rs

Another option is to use your distributions rust toolchain, but it might be too old. Proxy is currently built with rustc 1.77.1.

Next, run `cargo build` (or `cargo build --release` for a faster release version) in noita-proxy directory.

You also might need to add steam dynamic library. The easiest way to get one is from the latest mod release.

# Running several instances locally

Address 127.0.0.1:21251 is used to communicate between proxy and local Noita instance by default.
However, because generally ports can't be used by several apps at once, only one proxy instance can use the default address:port pair.

This address can be changed using enviromental variable NP_NOITA_ADDR. Example (on linux):
```bash
NP_DISABLE_STEAM=1 NP_NOITA_ADDR=127.0.0.1:21252 cargo run --release # To start the proxy
NP_NOITA_ADDR=127.0.0.1:21252 wine noita.exe # To start Noita
```

You'll probably want to add NP_SKIP_MOD_CHECK=1 to disable automatic mod installation as well.
