#!/bin/sh

pulseaudio -D
echo "running cmd"
exec "$@"
