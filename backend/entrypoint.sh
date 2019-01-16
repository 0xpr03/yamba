#!/bin/sh
set -e

rm -rf $HOME/.pulse/
mkdir $HOME/.pulse

pulseaudio --kill || true
pulseaudio --exit-idle-time=-1 &

# report warnings
export GST_DEBUG=2
exec "$@"
