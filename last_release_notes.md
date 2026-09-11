## Noita Entangled Worlds v1.6.5

Many world sync, entity sync correctness and performance fixes by @rdeaton, including:
 - Kolmi-related bugs
 - Enemies having abnormally high health
 - Wands and spells falling through the floor in HMs
 - and many others

## Accepted pull requests

- Add queueing to the steam reliable network channel and kick clients who are not catching up by @rdeaton in #515
- Version save state directory instead of migration by @rdeaton in #514
- [08/08] Rust world decode by @rdeaton in #513
- [07/08] Enable new tests in CI by @rdeaton in #512
- [06/08] Cleanup dead code by @rdeaton in #511
- [05/08] Assorted entity syncing fixes by @rdeaton in #510
- [04/08] Entity Cache and syncing changes by @rdeaton in #509
- [03/08] Mod side performance fixes around lukki, small correctness issue by @rdeaton in #508
- [02/08] Hot-loop performance improvements from profiling by @rdeaton in #507
- [01/08] Clean up some input handling by @rdeaton in #506
## Installation


Download and unpack `noita_proxy-win.zip` or `noita_proxy-linux.zip`, depending on your OS. After that, launch the proxy.


Proxy is able to download and install the mod automatically. There is no need to download the mod (`quant.ew.zip`) manually.


You'll be prompted for a path to `noita.exe` when launching the proxy for the first time.
It should be detected automatically as long as you use steam version of the game and steam is launched.
        

## Updating


There is a button in bottom-left corner on noita_proxy's main screen that allows to auto-update to a new version when one is available

