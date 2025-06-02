## Noita Entangled Worlds v1.5.4

Changed config and save_state location to system-specific locations (%APPDATA%\Roaming\quant\entangledworlds on windows, ~/.config/entangledworlds and ~/.local/share/entangledworlds on Linux)
Old locations (right next to the executable) will be used if proxy.ron already exists here.

Should fix issues related to saving when location next to executable isn't writeable.

## Accepted pull requests

- translated more strings for russian by @goluboch in #378

## Installation


Download and unpack `noita-proxy-win.zip` or `noita-proxy-linux.zip`, depending on your OS. After that, launch the proxy.


Proxy is able to download and install the mod automatically. There is no need to download the mod (`quant.ew.zip`) manually.


You'll be prompted for a path to `noita.exe` when launching the proxy for the first time.
It should be detected automatically as long as you use steam version of the game and steam is launched.
        

## Updating


There is a button in bottom-left corner on noita-proxy's main screen that allows to auto-update to a new version when one is available

