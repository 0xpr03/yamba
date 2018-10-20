#!/bin/bash

display="10";

echo "Killing previous Xvfb instances... ";
killall -9 Xvfb;

echo "Starting Xvfb with ts3 on display #$display... ";
/usr/bin/Xvfb :$display &
export DISPLAY=:$display.0
./ts3client/ts3client_runscript.sh
