#!/bin/bash

cd backend

if [ ! -d "ts3client" ]; then
    echo Download teamspeak client...
    chmod +x ts3-dl.sh
    ./ts3-dl.sh
fi

cd ..

sudo docker-compose up --build
