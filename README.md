## Noita Entangled Worlds - Play Noita not alone

[<img src="https://img.shields.io/liberapay/patrons/bgkillas.svg?logo=liberapay">](https://liberapay.com/bgkillas/) for one of the 2 devs, the other dev does not have the means for donations due to his residence

Noita Entangled Worlds is an online true coop multiplayer mod.

What is synced as of now:
 - Players, their positions, health, perks and inventories
 - Wand usage
 - Item usage
 - Items in world (One player can throw a wand/potion/spell/item, another can pick it up)
 - Enemies and their attacks
 - Pixels of the grid world
 - Fungal shifts
 - Polymorphing

There is a video by nichelessone that showcases a recent version: https://www.youtube.com/watch?v=mMP-93-RTs0

Discord server: [https://discord.gg/uAK7utvVWN](https://discord.gg/uAK7utvVWN)

## Installation (provided by @stefnotch)

Go to [releases](https://github.com/IntQuant/noita_entangled_worlds/releases) and download the latest `noita-proxy-win.zip` or `noita-proxy-linux.zip`, depending on your OS.

Unpack it, and launch the proxy. Proxy is able to download and install the mod automatically.

![image](https://github.com/user-attachments/assets/817cd204-1815-4834-803b-58761b21dc51)

Then, start Noita, and enable the mod.
1. In the "Mods" menu, enable unsafe mods.
2. Then, enable the "Quant's Entangled Worlds" mod.

![image](https://github.com/IntQuant/noita_entangled_worlds/assets/10220080/3a45f0ad-2ef1-4896-805c-1c1266e039c4)

Now you're ready to start a server and have fun!

### Installation on MacOS (provided by @Ownezx and @Roenalt)

1. Install a GOG copy of Noita using [portingkit](https://www.portingkit.com/) by following the guide given directly on the Noita entry page on portingkit with a few specific options in the "Advance Settings" step:
   1. Set the Engine to "WS12WineKegworks10.0-battle.net"
   2. Set the Operating System to "Windows 11". 
2. After confirming that the game launch, open the folder where the game is installed and navigate to where the `noita.exe` is located (usually in "/Users/{User}/Applications/Noita.app/Contents/SharedSupport/prefix/drive_c/GOG Games/Noita") and add a shortcut to it in the sidebar of the Finder. 
3. Go to [releases](https://github.com/IntQuant/noita_entangled_worlds/releases), download the latest `noita-proxy-macos.zip`.
4. Unpack it and launch the proxy, it will ask to give the path to the `noita.exe` (that we save a shortcut to!). Once the path is given, the proxy will be able to download and install the mod automatically.
5. Close the proxy, then launch it again via a terminal with the following command: `~/Applications/noita-proxy-macos/noita_proxy --launch-cmd '"/Users/{User}/Applications/Noita.app/Contents/MacOS/wineskinlauncher" --run "C:\GOG Games\Noita\noita.exe"'`
6. Then you can enjoy the mod as usual, by enabling it in the "Mods" menu of Noita.

Note: The proxy must be launched via terminal with the command above every time you want to play multiplayer.

## Installation on Linux with Lutris (provided by @merll002)

1. Install the GOG version of Noita through the lutris game installer:
   <img width="596" height="64" alt="image" src="https://github.com/user-attachments/assets/dfc2f415-1557-4716-b3e2-c62aae941344" />
2. Navigate to the directory where the proxy was downloaded
3. Run the proxy by typing `./start.sh`
4. Enable the mod (refer to main installation instructions)
5. Done!

## Connect using Steam

In the Proxy window, click on "Create Lobby". Then, "Save lobby ID to clipboard". Send that ID to your friends, who can then *copy* it and press "Connect to lobby in clipboard".

![image](https://github.com/user-attachments/assets/45cf2be6-090c-4d83-aa6b-516d94748cc5)

After that, just start a new Noita game on everyone's PCs, and you should be in multiplayer mode :)

## When to press "New Game" and when to press "Continue"

 - "New Game" - you're joining a multiplayer run you haven't joined before.
 - "Continue" - you're reconnecting to a multiplayer run that you've joined before and hasn't ended yet.

Using the same save file for multiplayer and singleplayer isn't something that should be done.

## Global perks

Some perks are perks and affect the entire world, and thus are shown for every player.

There are 11 global perks:
 - No More Shuffle
 - Unlimited Spells
 - Trick Blood Money
 - Gold is Forever
 - Greed
 - Trick Greed
 - Peace with Gods
 - Extra Item in Holy Mountain
 - More Love
 - More Hatred
 - More Blood

## Mods support

[The mods listed here](https://docs.google.com/spreadsheets/d/1nMdqzrLCav_diXbNPB9RgxPcCQzDPgXdEv-klKWJyS0) have been tested by the community, it is publically editable so please add any untested mod with your findings


## CLI connect

You can also connect via cli, just run `noita_proxy --lobby [steam_code/ip and port]`


## CLI host

You can also host via cli, just run `noita_proxy --host [steam/port]`, "--host steam" will host a steam game and "--host 5123" or any port will host via ip at that port

## Connecting via steam without steam version of game

There is a "Allow using steam networking even if you don't have the game on steam" checkbox in top left on main screen of proxy.

## Thanks

Special thanks to:
 - Contributors.
 - @EvaisaDev for allowing to use code from Noita Arena mod.
 - @dextercd for NoitaPatcher.
 - Creators of other libraries used in this project.
