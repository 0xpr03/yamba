#!/bin/sh
set -e

cd $HOME
rm -rf .pulse/
mkdir .pulse

pulseaudio --kill || true
pulseaudio --exit-idle-time=-1 &

# report warnings
export GST_DEBUG=2
exec "$@"
