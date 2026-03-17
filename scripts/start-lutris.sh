#!/bin/bash

if not pidof steam >/dev/null
then
	echo "Steam API is not available. If you want to play without port forwarding, Steam needs to be running."
	read -rp "Do you want to start it now? [y/N] " choice
	choice="${choice,,}"
	if [ "$choice" = "y" ] || [ "$choice" = "yes" ]
	then 
		nohup steam >/dev/null 2>&1 &
		read -rp "Started Steam. Press enter when it has fully initialised..."
	else
		echo "Running without Steam..."
	fi
fi

echo "Starting proxy..."

chmod +x noita_proxy.x86_64

exepath=$(grep 'prefix: ' -m1 ~/.local/share/lutris/games/*noita* | tail -c +11)/$(cat ~/.local/share/lutris/games/*noita* | grep 'exe: ' -m1 | tail -c +8)
name=$(grep 'game_slug: ' -m1 ~/.local/share/lutris/games/*noita* | tail -c +12)
LUTRIS_SKIP_INIT=1 ./noita_proxy.x86_64 --exe-path "$exepath" --launch-cmd "lutris lutris:rungame/$name"

