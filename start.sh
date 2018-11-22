#!/bin/bash

cd backend

if [ ! -d "ts3client" ]; then
    echo Download teamspeak client...
    chmod +x ts3-dl.sh
    ./ts3-dl.sh
fi

cd ..

set -a
source config/database.env
source config/database_root.env
set +a
sudo -E docker-compose up --build
