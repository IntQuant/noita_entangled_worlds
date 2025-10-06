#!/bin/bash

if not pidof steam >/dev/null
then
	echo "Steam API is not available. If you want to play without port forwarding, Steam needs to be running."
	read -rp "Do you want to start it now? [y/n] " choice
	choice="${choice,,}"
	if [ "$choice" = "y" ] || [ "$choice" = "yes" ]
	then 
		nohup steam >/dev/null 2>&1 &
		read -rp "Started Steam. Press enter when it has fully initialised..."
	else
		echo "Running without Steam..."
	fi
fi
ID=$(lutris --list-games 2>&1 | grep Noita | awk -F'|' '{ if (length($1) >= 2) print substr($1, length($1)-1, 1) }')

echo "Noita Lutris instance found with ID: $ID"
echo "Starting proxy..."

chmod +x noita_proxy.x86_64

LUTRIS_SKIP_INIT=1 ./noita_proxy.x86_64 --exe-path "/home/$USER/Games/gog/noita/drive_c/GOG Games/Noita/noita.exe" --launch-cmd "lutris lutris:rungameid/$ID"

