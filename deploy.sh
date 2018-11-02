#!/bin/bash

cd backend

echo Build backend...
cd daemon
RUST_BACKTRACE=1 cargo build --release
cd ..
cd ts3plugin
RUST_BACKTRACE=1 cargo build --release
cd ..

if [ ! -d "ts3client" ]; then
    echo Download teamspeak client...
    chmod +x ts3-dl.sh
    ./ts3-dl.sh
fi

cd ..

sudo docker-compose build
