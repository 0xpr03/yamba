#!/bin/sh

cd $HOME
rm -rf .pulse/
mkdir .pulse

pulseaudio --kill
pulseaudio --exit-idle-time=-1 -vvvv &

# report warnings
export GST_DEBUG=2
echo "running cmd"
exec "$@"
