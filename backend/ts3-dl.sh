#!/bin/bash
if [ -d "ts3client" ]; then
    rm -Rf ts3client
fi
rm TeamSpeak3*;
wget http://dl.4players.de/ts/releases/3.2.3/TeamSpeak3-Client-linux_amd64-3.2.3.run;
chmod +x TeamSpeak3-Client-linux_amd64-3.2.3.run;
./TeamSpeak3-Client-linux_amd64-3.2.3.run;
rm TeamSpeak3-Client-linux_amd64-3.2.3.run;
mv TeamSpeak3-Client-linux_amd64 ts3client;
