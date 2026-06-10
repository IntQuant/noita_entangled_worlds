## Noita Entangled Worlds v1.6.3

1.7 changes were stashed away for now in order to release fixes we've had.

Special thanks to Multirious for going through hundreds of commits and cherry-picking ones that are actually useful.

Notable changes in this release:
 - Local health alternate fixes and changes by @k-Knight and @Multirious. Dead players now use a special entity instead of a potion mimic.
 - Reimplementation of internally used polymorph effect by @k-Knight. Polymorph immunity shouldn't be able to break it anymore.
 - Updated French and Spanish translations by @leBourreau and @Icey-the-dragon.
 - Many internal improvements (see list of accepted pull requests).

## Accepted pull requests

- add darwin support to nix package by @Random-Scientist in #490
- Made heart statue walk by @k-Knight in #485
- Fix for the failed polymorph in local health logic by @k-Knight in #484
- Alternate local health tweaks and fixes by @Multirious in #483
- Fix cli not reading settings by @Multirious in #482
- Fix AudioSettings::disabled still trying to interact with audio library by @Multirious in #481
- Fix paths not set in cli by @Multirious in #479
- Path handling rewrite by @Multirious in #478
- Overridable settings path by @Multirious in #474
- Refactor noita-proxy UI code by @Multirious in #473
- Update Nix and cross-compilation by @Multirious in #471
- Refactor nix with `nixfmt` by @Multirious in #470
- ci: add matrix build for macOS (Intel & ARM64) by @artemkloko in #467
- Added Transtlation to spanish by @Icey-the-dragon in #451
- Fixed github release macos and lint workflow by @Icey-the-dragon in #450
- Complete French translation by @leBourreau in #446
- Create Linux (Lutris) install guide and automatic startup script by @LeoMerlino in #434
- nix+flake: init with noita-proxy package, overlay, and devshell  by @spikespaz in #432
- Add instructions to use proxy on MacOs by @RoenaltAstrophore in #426

## Installation


Download and unpack `noita_proxy-win.zip` or `noita_proxy-linux.zip`, depending on your OS. After that, launch the proxy.


Proxy is able to download and install the mod automatically. There is no need to download the mod (`quant.ew.zip`) manually.


You'll be prompted for a path to `noita.exe` when launching the proxy for the first time.
It should be detected automatically as long as you use steam version of the game and steam is launched.
        

## Updating


There is a button in bottom-left corner on noita_proxy's main screen that allows to auto-update to a new version when one is available

