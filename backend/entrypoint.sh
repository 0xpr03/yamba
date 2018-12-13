#!/bin/sh

cd $HOME
rm -rf .pulse/
mkdir .pulse

pulseaudio --kill
pulseaudio --exit-idle-time=-1 -vvvv &
#pulseaudio -D
echo "running cmd"
exec "$@"
