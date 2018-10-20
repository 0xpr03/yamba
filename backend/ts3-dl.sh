#!/bin/bash

rm TeamSpeak3*;
wget http://dl.4players.de/ts/releases/3.2.2/TeamSpeak3-Client-linux_amd64-3.2.2.run;
chmod +x TeamSpeak3-Client-linux_amd64-3.2.2.run;
./TeamSpeak3-Client-linux_amd64-3.2.2.run;
rm TeamSpeak3-Client-linux_amd64-3.2.2.run;
mv TeamSpeak3-Client-linux_amd64 ts3client;
