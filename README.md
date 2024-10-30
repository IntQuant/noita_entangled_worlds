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

## Connect using Steam

In the Proxy window, click on "Create Lobby". Then, "Save lobby ID to clipboard". Send that ID to your friends, who can then *copy* it and press "Connect to lobby in clipboard".

![image](https://github.com/user-attachments/assets/45cf2be6-090c-4d83-aa6b-516d94748cc5)

After that, just start a new Noita game on everyone's PCs, and you should be in multiplayer mode :)

## Mods support

[The mods listed here](https://docs.google.com/spreadsheets/d/1nMdqzrLCav_diXbNPB9RgxPcCQzDPgXdEv-klKWJyS0) have been tested by the community, it is publically editable so please add any untested mod with your findings


## Cli connect

can also connect via cli, just run `noita_proxy --lobby [steam_code/ip and port]`


## Cli host

can also host via cli, just run `noita_proxy --host [steam/port]`, "--host steam" will host a steam game and "--host 5123" or any port will host via ip at that port

## Connecting via steam without steam version of game

to connect via steam without the steam version of game, since its more stable you can do the following

on all clients run the proxy with the NP_APPID=480 environemental variable, you can do that by making a bat/bash file to set that before running the executable

## Thanks

Special thanks to:
 - Contributors.
 - @EvaisaDev for allowing to use code from Noita Arena mod.
 - @dextercd for NoitaPatcher.
 - Creators of other libraries used in this project.